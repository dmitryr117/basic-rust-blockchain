use crate::block::{Block, ChainBlock};

trait Blockchaining {
  fn add_block(&mut self, data: Vec<String>);
}

pub struct Blockchain {
  chain: Vec<Block>
}

impl Blockchain {
  fn new() -> Self {
    Self {
      chain: vec!(Block::genesis())
    }
  }
}

impl Blockchaining for Blockchain {
  fn add_block(&mut self, data: Vec<String>) {
      let last_block = self.chain.last().unwrap();
      let new_block = Block::mine_block(data, last_block);
      self.chain.push(new_block);
  }
}

#[cfg(test)]
mod test_blockchain {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn contains_chain_vec() {
    let blockchain = Blockchain::new();
    assert!(blockchain.chain.len() > 0);
  }

  #[test]
  fn starts_with_genesis() {
    let blockchain = Blockchain::new();
    assert_eq!(*(blockchain.chain.first().unwrap()), Block::genesis());
  }

  #[test]
  fn adds_new_block_to_chain() {
    let new_data = vec!(String::from("foo"), String::from("bar"));
    let mut blockchain = Blockchain::new();
    blockchain.add_block(new_data.clone());
    assert_eq!(blockchain.chain.last().unwrap().data, new_data);
  }
}