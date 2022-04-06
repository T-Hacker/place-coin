use crate::{
    block::Block,
    transaction::{Address, Credits, Point, Transaction},
};
use anyhow::{bail, Result};
use rayon::prelude::*;
use tiny_keccak::{Hasher, Sha3};

pub type Proof = u128;
pub type Hash = [u8; 32];

#[derive(Debug)]
pub struct Blockchain {
    chain: Vec<Block>,
    transactions: Vec<Transaction>,
}

impl Default for Blockchain {
    fn default() -> Self {
        let genesis_block = Block::new(0, Default::default(), 100, Default::default());

        Self {
            chain: vec![genesis_block],
            transactions: Default::default(),
        }
    }
}

impl Blockchain {
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Check if sender has the credits to do the transaction.
        match &transaction {
            Transaction::Peer { sender, amount, .. } => {
                let credits = &self.get_peer_credits(sender, None);
                if *sender != 0 && amount > credits {
                    bail!("Sender does not have the credits to do the transaction. Credits: {credits}, Amount: {amount}")
                }
            }

            Transaction::Pixel {
                sender, position, ..
            } => {
                let credits = self.get_peer_credits(sender, None);
                let cost = self.get_pixel_cost(sender, position, None);
                if cost > credits {
                    bail!("Buyer doesn't have enough credits to paint pixel. Credits: {credits}, Cost: {cost}")
                }
            }
        }

        self.transactions.push(transaction);

        Ok(())
    }

    pub fn new_block(&mut self, proof: Proof, previous_hash: Hash) -> usize {
        let mut transactions = Default::default();
        std::mem::swap(&mut self.transactions, &mut transactions);

        let block = Block::new(
            self.chain.len() as u64,
            transactions,
            proof,
            Some(previous_hash),
        );

        self.chain.push(block);

        self.chain.len() - 1
    }

    pub fn get_last_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn proof_of_work(&self) -> Proof {
        let last_block = self.get_last_block();
        let last_proof = last_block.get_proof();

        (0..Proof::MAX)
            .into_par_iter()
            .find_any(|possible_proof| Self::validate_proof(last_proof, possible_proof))
            .unwrap()
    }

    pub fn get_peer_credits(&self, peer_address: &Address, block_index: Option<u64>) -> Credits {
        let block_index = block_index.unwrap_or_else(|| self.get_last_block().get_index()) + 1;

        self.chain
            .par_iter()
            .take(block_index as usize)
            .flat_map(|block| block.get_transactions())
            .filter_map(|transaction| match transaction {
                Transaction::Peer {
                    sender,
                    recipient,
                    amount,
                } => {
                    if sender == peer_address {
                        Some(-amount)
                    } else if recipient == peer_address {
                        Some(*amount)
                    } else {
                        None
                    }
                }

                Transaction::Pixel {
                    sender, position, ..
                } => {
                    if sender == peer_address {
                        let pixel_cost =
                            self.get_pixel_cost(peer_address, position, Some(block_index));

                        Some(-pixel_cost)
                    } else {
                        None
                    }
                }
            })
            .sum::<Credits>()
    }

    pub fn get_pixel_cost(
        &self,
        peer_id: &Address,
        position: &Point,
        block_index: Option<u64>,
    ) -> Credits {
        // Calculate base pixel cost.
        let base_cost = {
            let (x, y) = position;
            let (x, y) = (f64::from(*x), f64::from(*y));
            let radius = (x.powi(2) + y.powi(2)).sqrt(); // Distance to center of canvas.
            let radius = radius * 0.01; // Scale back radius to make the highest cost 1% of total possible currency available.

            1 + (radius * ((i32::MAX - 1) as f64)) as Credits
        };

        // Find previous owners of the pixel.
        let peers = self
            .get_pixel_owners(position, block_index)
            .filter(|id| *id != peer_id);

        // Add share price of the pixel for each previous owners.
        let peer_count = peers.count();

        // Calculate final price of the pixel.
        base_cost + base_cost * peer_count as i64
    }

    fn get_pixel_owners<'a>(
        &'a self,
        position: &'a Point,
        block_index: Option<u64>,
    ) -> impl ParallelIterator<Item = &Address> + 'a {
        let block_index = block_index.unwrap_or_else(|| self.get_last_block().get_index()) + 1;

        self.chain
            .par_iter()
            .take(block_index as usize)
            .flat_map(|block| block.get_transactions())
            .filter_map(move |transaction| match transaction {
                Transaction::Peer { .. } => None,
                Transaction::Pixel {
                    sender,
                    position: pixel_pos,
                    ..
                } => {
                    if pixel_pos == position {
                        Some(sender)
                    } else {
                        None
                    }
                }
            })
    }

    fn validate_proof(last_proof: &Proof, proof: &Proof) -> bool {
        let mut sha3 = Sha3::v256();
        sha3.update(&last_proof.to_le_bytes());
        sha3.update(&proof.to_le_bytes());

        let mut hash: Hash = Default::default();
        sha3.finalize(&mut hash);

        hash.iter().take(1).all(|e| *e == 0)
    }
}
