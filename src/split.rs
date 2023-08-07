use crate::bytewords::byteword_string;
use anyhow::{anyhow, bail, Error};
use bip39::{Language, Mnemonic, MnemonicType};
use dcbor::{CBOREncodable, CBOR};
use lazy_static::lazy_static;
use regex::Regex;
use sskr::{sskr_generate, GroupSpec, Secret, Spec};

lazy_static! {
    static ref SPEC_REGEX: Regex = Regex::new(r"^((\d+of\d+),)*\d+of\d+$").unwrap();
    static ref SPEC_GROUP_REGEX: Regex = Regex::new(r"(?<m>\d+)of(?<n>\d+)").unwrap();
}

pub fn split(
    spec: &String,
    group_threshold: usize,
    phrase: &String,
) -> Result<(Mnemonic, Vec<Vec<String>>), Error> {
    let sskr_spec = parse_spec(spec, group_threshold)?;
    let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;
    let entropy = mnemonic.entropy();
    let secret = Secret::new(entropy)?;
    let groups = sskr_generate(&sskr_spec, &secret)?;
    let byteword_groups = to_bytewords(&groups);
    Ok((mnemonic, byteword_groups))
}

pub fn split_random_phrase(
    spec: &String,
    group_threshold: usize,
) -> Result<(Mnemonic, Vec<Vec<String>>), Error> {
    let mnemonic = Mnemonic::new(MnemonicType::Words12, Language::English);
    split(spec, group_threshold, &mnemonic.phrase().to_string())
}

fn to_bytewords(groups: &Vec<Vec<Vec<u8>>>) -> Vec<Vec<String>> {
    groups
        .iter()
        .map(|shares| {
            shares
                .iter()
                .map(|share| {
                    let cbor = CBOR::tagged_value(309, CBOR::byte_string(share));
                    byteword_string(cbor.cbor_data().as_slice())
                })
                .collect()
        })
        .collect()
}

fn parse_spec(spec: &String, group_threshold: usize) -> Result<Spec, Error> {
    if !SPEC_REGEX.is_match(spec) {
        bail!("Invalid group spec");
    }

    let mut group_specs: Vec<GroupSpec> = vec![];

    for part in spec.split(",") {
        let Some(group_match) = SPEC_GROUP_REGEX.captures(&part) else {
            bail!("Invalid group \"{}\" in spec", &part);
        };

        let m = group_match["m"].parse()?;
        let n = group_match["n"].parse()?;

        if m > n {
            bail!(
                "Invalid group \"{}\" in spec ({} is greater than {})",
                &part,
                m,
                n
            );
        }

        if m == 1 && n > 1 {
            bail!(
                "Invalid group \"{}\" in spec: 1 of N groups (where N > 1) not supported",
                &part
            );
        }

        group_specs.push(
            GroupSpec::new(m, n)
                .map_err(|e| anyhow!("Error making group spec for group \"{}\": {}", &part, e))?,
        );
    }

    Ok(Spec::new(group_threshold, group_specs)?)
}
