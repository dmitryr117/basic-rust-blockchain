use crate::{config::{GENESIS_DATA, GENESIS_HASH, GENESIS_LAST_HASH, GENESIS_TS}, crypto_hash::cryptohash};
use chrono::{Utc};

pub trait ChainBlock<T> {
    fn genesis() -> T;
    fn mine_block(data: Vec<String>, last_block: &T) -> T;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub timestamp: i64,
    pub last_hash: Vec<u8>,
    pub hash: Vec<u8>,
    pub data: Vec<String>,
}

impl Block {
    pub fn new(timestamp: i64, last_hash: Vec<u8>, hash: Vec<u8>, data: Vec<String>) -> Self {
        Self {
            timestamp,
            last_hash,
            hash,
            data
        }
    }
}

impl ChainBlock<Block> for Block {
    fn genesis() -> Self {
        let data = GENESIS_DATA.iter().map(|item| {item.to_string()}).collect();
        Self::new(GENESIS_TS, GENESIS_LAST_HASH.to_vec(), GENESIS_HASH.to_vec(), data)        
    }

    fn mine_block(data: Vec<String>, last_block: &Block) -> Block {
        let nano_time = Utc::now().timestamp_nanos_opt().unwrap();
        let last_hash = hex::encode(&last_block.hash);
        let new_hash = cryptohash(&data, &last_hash, nano_time);
        Self::new(nano_time, last_block.hash.clone(), new_hash, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_new_block() {
        let timestamp = 1234;
        let last_hash = vec![1, 2, 3, 4];
        let hash = vec![1, 2, 3, 4];
        let data = vec![String::from("data")];

        let new_block = Block::new(timestamp, last_hash.clone(), hash.clone(), data.clone());

        let comp_block = Block {
            timestamp,
            last_hash,
            hash, 
            data
        };

        assert_eq!(new_block, comp_block);
    }

    #[test]
    fn test_genesis() {
        let genesis_block = Block::genesis();

        let genesis_data = GENESIS_DATA.iter().map(|item| item.to_string()).collect();
        let comp_block = Block {
            timestamp: GENESIS_TS,
            last_hash: GENESIS_LAST_HASH.to_vec(),
            hash: GENESIS_HASH.to_vec(),
            data: genesis_data,
        };

        assert_eq!(genesis_block, comp_block);
    }

    #[test]
    fn test_mine_block() {
        let last_block = Block::genesis();
        let data = vec![String::from("Mined Data")];
        let mined_block = Block::mine_block(data, &last_block);

        assert_eq!(last_block.hash, mined_block.last_hash);
        assert_eq!(vec![String::from("Mined Data")], mined_block.data);
    }

    #[test]
    fn test_black_data_sorting() {
        let mut data = vec!["bcd", "cdf", "abc"];
        data.sort();
        let data: Vec<String> = data.iter().map(|item| item.to_string()).collect();
        let expected = vec![String::from("abc"), String::from("bcd"), String::from("cdf")];
        assert_eq!(expected, data);
    }

}
