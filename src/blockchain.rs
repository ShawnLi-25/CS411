use std::collections::HashMap;

use crate::block::Block;
use crate::crypto::hash::H256;

pub struct Blockchain {
    blocks: HashMap<H256, Block>,
    orphans: HashMap<H256, Vec<Block>>, // key is the hash of the parent
    longest_hash: H256,
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        let genesis = Block::genesis();
        let longest_hash = genesis.get_hash();
        let mut map: HashMap<H256, Block> = HashMap::new();
        let orphans: HashMap<H256, Vec<Block>> = HashMap::new();
        map.insert(genesis.get_hash(), genesis);
        Self {
            blocks: map,
            orphans: orphans,
            longest_hash: longest_hash
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let mut b = block.clone();
        let parent_hash = &b.header.parent;
        match self.blocks.get(parent_hash) {
            Some(prev_block) => {
                let cur_index = prev_block.index + 1;
                b.index = cur_index;
                let longest_block = self.blocks.get(&self.longest_hash).unwrap();
                if cur_index > longest_block.index {
                    self.longest_hash = b.hash.clone();
                }
                let new_parent_hash = b.hash.clone();
                self.blocks.insert(b.hash.clone(), b);
                self.handle_orphan(&new_parent_hash);
            },
            None => {
                match self.orphans.get_mut(parent_hash) {
                    Some(children_vec) => {
                        children_vec.push(b);
                    },
                    None => {
                        let mut children_vec = Vec::<Block>::new();
                        let parent_hash_copy = parent_hash.clone();
                        children_vec.push(b);
                        self.orphans.insert(parent_hash_copy, children_vec);
                    }
                }
            }
        }
    }

    fn handle_orphan(&mut self, new_parent: &H256) {
        match self.orphans.remove(new_parent) {
            Some(children_vec) => {
                for child in children_vec.iter() {
                    self.insert(child);
                }
            },
            None => {}
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.longest_hash.clone()
    }

    /// Get the last block's hash of the longest chain
    #[cfg(any(test, test_utilities))]
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        unimplemented!()
    }
}

#[cfg(any(test, test_utilities))]
mod tests {
    use super::*;
    use crate::block::test::generate_random_block;
    use crate::crypto::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        assert_eq!(&genesis_hash, &H256::from([0u8; 32]));
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }

    #[test]
    fn switch_tip() {
        /*
         * structure:
         * genesis <- block_1_1 <- block_1_2 <- block_1_3 <- block_1_4
         *              ^
         *              ---------  block_2_1 <- block_2_2
         */
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block_1_1 = generate_random_block(&genesis_hash);
        blockchain.insert(&block_1_1);
        let block_1_2 = generate_random_block(&block_1_1.hash());
        blockchain.insert(&block_1_2);
        assert_eq!(blockchain.tip(), block_1_2.hash());
        let block_2_1 = generate_random_block(&block_1_1.hash());
        blockchain.insert(&block_2_1);
        assert_eq!(blockchain.tip(), block_1_2.hash());
        let block_2_2 = generate_random_block(&block_2_1.hash());
        blockchain.insert(&block_2_2);
        assert_eq!(blockchain.tip(), block_2_2.hash());
        let block_1_3 = generate_random_block(&block_1_2.hash());
        blockchain.insert(&block_1_3);
        assert_eq!(blockchain.tip(), block_2_2.hash());
        let block_1_4 = generate_random_block(&block_1_3.hash());
        blockchain.insert(&block_1_4);
        assert_eq!(blockchain.tip(), block_1_4.hash());
    }

    #[test]
    fn handle_orphan() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block1 = generate_random_block(&genesis_hash);
        let block2 = generate_random_block(&block1.hash());
        let block3 = generate_random_block(&block2.hash());
        blockchain.insert(&block3);
        blockchain.insert(&block2);
        blockchain.insert(&block1);
        assert_eq!(blockchain.tip(), block3.hash());

        // naming rule: block_<branch>_<index>
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block_1_1 = generate_random_block(&genesis_hash);
        let block_1_2 = generate_random_block(&block_1_1.hash());
        let block_1_3 = generate_random_block(&block_1_2.hash());
        let block_2_2 = generate_random_block(&block_1_1.hash());
        let block_2_3 = generate_random_block(&block_2_2.hash());
        let block_2_4 = generate_random_block(&block_2_3.hash());
        let block_2_5 = generate_random_block(&block_2_4.hash());
        blockchain.insert(&block_2_5);
        blockchain.insert(&block_2_4);
        blockchain.insert(&block_2_3);
        blockchain.insert(&block_2_2);
        blockchain.insert(&block_1_3);
        blockchain.insert(&block_1_2);
        assert_eq!(blockchain.tip(), genesis_hash);
        blockchain.insert(&block_1_1);
        assert_eq!(blockchain.tip(), block_2_5.hash());
    }
}
