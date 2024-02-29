use anyhow::{anyhow, Error};
use crc::{Crc, CRC_32_ISO_HDLC};
use lazy_static::lazy_static;
use std::collections::HashMap;

#[rustfmt::skip]
static WORDS: &'static str =
    "ableacidalsoapexaquaarchatomauntawayaxisbackbaldbarnbeltbetabias\
     bluebodybragbrewbulbbuzzcalmcashcatschefcityclawcodecolacookcost\
     cruxcurlcuspcyandarkdatadaysdelidicedietdoordowndrawdropdrumdull\
     dutyeacheasyechoedgeepicevenexamexiteyesfactfairfernfigsfilmfish\
     fizzflapflewfluxfoxyfreefrogfuelfundgalagamegeargemsgiftgirlglow\
     goodgraygrimgurugushgyrohalfhanghardhawkheathelphighhillholyhope\
     hornhutsicedideaidleinchinkyintoirisironitemjadejazzjoinjoltjowl\
     judojugsjumpjunkjurykeepkenokeptkeyskickkilnkingkitekiwiknoblamb\
     lavalazyleaflegsliarlimplionlistlogoloudloveluaulucklungmainmany\
     mathmazememomenumeowmildmintmissmonknailnavyneednewsnextnoonnote\
     numbobeyoboeomitonyxopenovalowlspaidpartpeckplaypluspoempoolpose\
     puffpumapurrquadquizraceramprealredorichroadrockroofrubyruinruns\
     rustsafesagascarsetssilkskewslotsoapsolosongstubsurfswantacotask\
     taxitenttiedtimetinytoiltombtoystriptunatwinuglyundouniturgeuser\
     vastveryvetovialvibeviewvisavoidvowswallwandwarmwaspwavewaxywebs\
     whatwhenwhizwolfworkyankyawnyellyogayurtzapszerozestzinczonezoom";

lazy_static! {
    static ref WORD_LOOKUP: HashMap<&'static str, u8> = {
        let mut lookup = HashMap::new();
        for i in 0..=255 {
            lookup.insert(index_to_byteword(i), i);
        }
        lookup
    };
}

fn index_to_byteword(i: u8) -> &'static str {
    let begin: usize = (i as u16 * 4) as usize;
    let end: usize = (begin + 4) as usize;
    &WORDS[begin..end]
}

fn byteword_to_index(word: &str) -> u8 {
    WORD_LOOKUP[word]
}

fn byteword_checksum(bytes: &[u8]) -> [u8; 4] {
    Crc::<u32>::new(&CRC_32_ISO_HDLC)
        .checksum(bytes)
        .to_be_bytes()
}

pub fn byteword_string(bytes: &[u8], minimal: &bool) -> String {
    let checksum = byteword_checksum(bytes);
    let data_with_checksum = [bytes, &checksum].concat();
    byteword_string_no_checksum(&data_with_checksum, minimal)
}

pub fn byteword_string_no_checksum(bytes: &[u8], minimal: &bool) -> String {
    bytes
        .iter()
        .map(|i| {
            let btw = index_to_byteword(*i);
            if *minimal {
                // take the first and the last letter of the byteword
                let first = btw.chars().next().unwrap();
                let last = btw.chars().last().unwrap();
                format!("{}{}", first, last)
            } else {
                btw.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn byteword_string_to_bytes(input: String, minimal: &bool) -> Result<Vec<u8>, Error> {
    let keys: Vec<_> = WORD_LOOKUP.keys().cloned().collect();
    let parts = input.split(" ");
    let words: Vec<_> = if *minimal {
        parts.map(|x| {
            keys.iter().find(|&&w| {
                let first = w.chars().next().unwrap();
                let last = w.chars().last().unwrap();
                format!("{}{}", first, last) == x
            }).unwrap().clone()
        }).collect()
    } else {
        parts.collect()
    };

    for word in words.clone().into_iter() {
        if !WORD_LOOKUP.contains_key(word) {
            return Err(anyhow!("Not a valid byteword: \"{}\"", word));
        }
    }
    let all_bytes = words.into_iter().map(byteword_to_index).collect::<Vec<u8>>();
    if all_bytes.len() < 5 {
        return Err(anyhow!(
            "Byteword string too short (must include checksum): \"{}\"",
            input
        ));
    }
    let (bytes, checksum) = all_bytes.split_at(all_bytes.len() - 4);
    if checksum != byteword_checksum(bytes) {
        return Err(anyhow!(
            "Invalid checksum (last 4 words) for byteword string \"{}\"",
            input
        ));
    }
    Ok(bytes.to_vec())
}