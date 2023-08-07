use crate::bytewords::*;
use crate::sskr_shares::*;
use anyhow::{anyhow, bail, Error};
use bip39::{Language, Mnemonic};
use dcbor::CBOR;
use sskr::sskr_combine;
use std::collections::HashMap;

pub fn recover(lines: Vec<String>) -> Result<Mnemonic, Error> {
    let mut shares: Vec<Vec<u8>> = vec![];

    // Get shares from raw strings
    for line in lines {
        // Parse bytewords and strip byteword-level checksum
        let bytes = byteword_string_to_bytes(line)?;

        // Unwrap data from CBOR container
        let cbor = CBOR::from_data(bytes.as_slice())?;
        let cbor_bytes = cbor.expect_tagged_value(309)?;
        let share = cbor_bytes.expect_byte_string()?;

        // Retain share data
        shares.push(share.to_vec());
    }

    // Parse out metadata from each share
    let mut share_ids: Vec<u16> = vec![];
    let mut share_meta: Vec<[usize; 5]> = vec![];
    for share in shares.clone() {
        let (id, meta) = share_metadata(&share)?;
        share_ids.push(id);
        share_meta.push(meta);
    }

    let identifier = share_ids[0];

    // Make sure identifier is the same for all shares
    if share_ids.iter().any(|id| id != &identifier) {
        bail!("Mismatched identifiers, shares don't go together");
    }

    let group_threshold = share_meta[0][1];
    let group_count = share_meta[0][2];

    // Make sure group threshold and group count are the same for all shares
    if share_meta
        .iter()
        .any(|meta| meta[1] != group_threshold || meta[2] != group_count)
    {
        bail!("Mismatched group threshold or count, shares don't go together");
    }

    // Group shares by group in the form { group_num => Vec<(share_index, share)> }
    let mut shares_by_group: HashMap<usize, Vec<(usize, Vec<u8>)>> = HashMap::new();
    for (i, share) in shares.clone().iter().enumerate() {
        let share_group_num = share_meta[i][0];

        if !shares_by_group.contains_key(&share_group_num) {
            shares_by_group.insert(share_group_num, vec![]);
        }

        shares_by_group
            .get_mut(&share_group_num)
            .unwrap()
            .push((i, share.to_vec()));
    }

    // See how many groups are recoverable
    let mut recoverable_groups: Vec<usize> = vec![];

    // Look through the groups one-by-one to validate and gather information
    for (group_num, shares) in &shares_by_group {
        // Make sure the member threshold is the same for all shares in the group
        let member_threshold = share_meta[shares[0].0][4];
        if shares
            .iter()
            .any(|(i, _share)| share_meta[*i][4] != member_threshold)
        {
            bail!(
                "Mismatched share member thresholds in group {}, shares don't go together",
                group_num + 1
            );
        }

        // See whether this group is recoverable
        if shares.len() >= member_threshold {
            recoverable_groups.push(*group_num);
        }
    }

    // Make sure there are enough groups to recover the secret
    if recoverable_groups.len() < group_threshold {
        bail!(
            "Not enough groups, need to satisfy at least {} but only {} are satisfied ({})",
            group_threshold,
            recoverable_groups.len(),
            recoverable_groups
                .iter()
                .map(|g| (g + 1).to_string())
                .collect::<Vec<String>>()
                .join(" and ")
        )
    }

    let mut shares_for_recovery: Vec<Vec<u8>> = vec![];

    // Gather shares from enough theoretically-recoverable groups
    for group_num in recoverable_groups.iter().take(group_threshold) {
        // Get just the shares without the group number
        let group_shares = shares_by_group[&group_num]
            .iter()
            .map(|(_i, share)| share.to_vec())
            .collect::<Vec<Vec<u8>>>();

        for share in group_shares {
            shares_for_recovery.push(share);
        }
    }

    let secret = sskr_combine(&shares_for_recovery)
        .map_err(|e| anyhow!("Error during SSKR combination: {}", e))?;

    Mnemonic::from_entropy(secret.data(), Language::English).map_err(|e| {
        anyhow!(
            "Recovered entropy 0x{} but unable to make mnemonic: {}",
            hex::encode(secret.data()),
            e
        )
    })
}
