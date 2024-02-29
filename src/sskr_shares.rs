use crate::bytewords::*;
use anyhow::{bail, Error};
use sskr::METADATA_SIZE_BYTES;

pub fn share_metadata(source: &[u8], minimal: &bool) -> Result<(u16, [usize; 5]), Error> {
    if source.len() < METADATA_SIZE_BYTES {
        bail!(
            "Share is too short: \"{}\"",
            byteword_string_no_checksum(&source, minimal)
        );
    }

    let group_threshold = ((source[2] >> 4) + 1) as usize;
    let group_count = ((source[2] & 0xf) + 1) as usize;

    if group_threshold > group_count {
        bail!(
            "Share has invalid group threshold: \"{}\"",
            byteword_string_no_checksum(&source, minimal)
        );
    }

    let identifier = ((source[0] as u16) << 8) | source[1] as u16;
    let group_index = (source[3] >> 4) as usize;
    let member_threshold = ((source[3] & 0xf) + 1) as usize;
    let reserved = source[4] >> 4;
    if reserved != 0 {
        bail!(
            "Share has invalid reserved bits: \"{}\"",
            byteword_string_no_checksum(&source, minimal)
        );
    }
    let member_index = (source[4] & 0xf) as usize;

    Ok((
        identifier,
        [
            group_index,
            group_threshold,
            group_count,
            member_index,
            member_threshold,
        ],
    ))
}
