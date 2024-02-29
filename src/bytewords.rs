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
    static ref WORD_TO_INDEX_LOOKUP: HashMap<&'static str, u8> = {
        let mut lookup = HashMap::new();
        for i in 0..=255 {
            lookup.insert(index_to_byteword(i), i);
        }
        lookup
    };

    static ref MINIMAL_WORD_TO_WORD_LOOKUP: HashMap<String, &'static str> = {
        let mut lookup = HashMap::new();
        for i in 0..=255 {
            let word = index_to_byteword(i);
            let minimal = byteword_to_minimal_string(word);
            lookup.insert(minimal, word);
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
    WORD_TO_INDEX_LOOKUP[word]
}

fn byteword_checksum(bytes: &[u8]) -> [u8; 4] {
    Crc::<u32>::new(&CRC_32_ISO_HDLC)
        .checksum(bytes)
        .to_be_bytes()
}

fn byteword_minimal_string_to_byteword(input: &str) -> Result<Vec<&str>, Error> {
    let chars = input.chars().collect::<Vec<char>>();
    let chunks= chars
        .chunks(2)
        .map(|x| x.iter().collect::<String>());

    let words = chunks.map(|x| {
        match MINIMAL_WORD_TO_WORD_LOOKUP.get(&x) {
            Some(word) => Ok(*word),
            None => return Err(anyhow!("Not a valid byteword: \"{}\"", x)),
        }
    }).collect();

    words
}

fn byteword_to_minimal_string(word: &str) -> String {
    let first = word.chars().next().unwrap();
    let last = word.chars().last().unwrap();
    format!("{}{}", first, last)
}

pub fn byteword_string(bytes: &[u8], minimal: &bool) -> String {
    let checksum = byteword_checksum(bytes);
    let data_with_checksum = [bytes, &checksum].concat();
    byteword_string_no_checksum(&data_with_checksum, minimal)
}

pub fn byteword_string_no_checksum(bytes: &[u8], minimal: &bool) -> String {
    let words = bytes
        .iter()
        .map(|i| {
            let btw = index_to_byteword(*i);
            if *minimal {
                byteword_to_minimal_string(btw)
            } else {
                btw.to_string()
            }
        })
        .collect::<Vec<String>>();

    if *minimal {
        words.join("")
    } else {
        words.join(" ")
    }
}

pub fn byteword_string_to_bytes(input: &str, minimal: &bool) -> Result<Vec<u8>, Error> {
    let words: Vec<&str> = if *minimal {
        byteword_minimal_string_to_byteword(input)?
    } else {
        input.split(" ").collect()
    };

    for word in words.clone().into_iter() {
        if !WORD_TO_INDEX_LOOKUP.contains_key(word) {
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