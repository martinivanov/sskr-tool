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

pub fn byteword_string(bytes: &[u8]) -> String {
    let checksum = byteword_checksum(bytes);
    let data_with_checksum = [bytes, &checksum].concat();
    byteword_string_no_checksum(&data_with_checksum)
}

pub fn byteword_string_no_checksum(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|i| index_to_byteword(*i))
        .collect::<Vec<&str>>()
        .join(" ")
}

pub fn byteword_string_to_bytes(input: String) -> Result<Vec<u8>, Error> {
    let words = input.split(" ");
    for word in words.clone() {
        if !WORD_LOOKUP.contains_key(word) {
            return Err(anyhow!("Not a valid byteword: \"{}\"", word));
        }
    }
    let all_bytes = words.map(byteword_to_index).collect::<Vec<u8>>();
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
