use cryptochain::blockchain::Blockchain;
use cryptochain::block::Block;

mod is_valid_chain {
  #[test]
  fn when_chain_does_not_start_with_genesis() {
    // is valid chain returns false

  }
}

mod chain_starts_with_genesis_block {
  #[test]
  fn and_last_hash_reference_has_changed() {
    // is valid chain returns false
    
  }

  #[test]
  fn chain_has_block_with_invalid_field() {
    // is valid chain returns false
    
  }

  #[test]
  fn chain_containe_only_valid_blocks() {
    // is valid chain returns true
    
  }
}