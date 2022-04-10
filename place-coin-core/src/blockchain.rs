use crate::{
    block::Block,
    transaction::{Credits, Transaction, TransactionInput, TransactionOutput},
};
use anyhow::{bail, Result};
use rayon::prelude::*;
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;

pub type Proof = u128;
pub type Hash = [u8; 32];

const BLOCK_LOCK_TIME: u32 = 0; // Minimum block height that must exist before the reward can be cashed out.

#[derive(Debug)]
pub struct Blockchain {
    miner_public_key_hash: Hash,
    blocks: HashMap<Hash, Block>,
    transactions: Vec<Transaction>,
    last_block_hash: Hash,
}

impl Blockchain {
    pub fn new(miner_public_key_hash: Hash) -> Self {
        let genesis_block = Block::new(Default::default(), 100, Default::default());
        let genesis_block_hash = genesis_block.calculate_hash();

        let mut blocks = HashMap::new();
        blocks.insert(genesis_block_hash, genesis_block);

        Self {
            miner_public_key_hash,
            blocks,
            transactions: Default::default(),
            last_block_hash: genesis_block_hash,
        }
    }

    pub fn new_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Add the transaction to be later added to the next block.
        self.transactions.push(transaction);

        Ok(())
    }

    pub fn mine(&mut self) -> Result<()> {
        let mut transactions = Default::default();
        std::mem::swap(&mut self.transactions, &mut transactions);

        // Calculate block reward.
        let total_unspent_outputs: Credits = transactions
            .iter()
            .map(|transaction| transaction.get_balance())
            .sum();

        let block_reward = 1000 + total_unspent_outputs;

        // Add reward transaction.
        let last_block = self.get_last_block();
        let inputs = vec![TransactionInput::FromReward {
            height: last_block.get_block_height()? + 1,
            value: block_reward,
        }];

        let outputs = vec![TransactionOutput::ToInput {
            value: block_reward,
            public_key_hash: self.miner_public_key_hash,
        }];

        let reward_transaction = Transaction::try_new(self, inputs, outputs, BLOCK_LOCK_TIME)?;
        transactions.push(reward_transaction);

        // Create proof of work.
        let proof = self.proof_of_work();

        // Create the new block.
        let new_block = Block::new(transactions, proof, Some(self.last_block_hash));
        let new_block_hash = new_block.calculate_hash();

        self.blocks.insert(new_block_hash, new_block);
        self.last_block_hash = new_block_hash;

        Ok(())
    }

    pub fn get_peer_credits(&self, peer_address: &Hash) -> Credits {
        self.blocks
            .par_iter()
            .flat_map(|(_, block)| block.get_transactions())
            .map(|transaction| {
                std::iter::repeat(transaction).zip(transaction.get_outputs().iter().enumerate())
            })
            .flatten_iter()
            .filter_map(|(transaction, (output_index, output))| match output {
                TransactionOutput::ToInput {
                    value,
                    public_key_hash,
                } => {
                    let transaction_hash = transaction.get_hash();

                    if public_key_hash != peer_address
                        || self.is_output_spent(transaction_hash, output_index as u32)
                    {
                        None
                    } else {
                        Some(value)
                    }
                }

                TransactionOutput::ToPixel { .. } => None,
            })
            .sum()
    }

    pub fn get_last_block(&self) -> &Block {
        self.blocks.get(&self.last_block_hash).unwrap()
    }

    pub fn get_all_unspent_outputs(
        &self,
    ) -> impl ParallelIterator<Item = (&Transaction, &TransactionOutput, usize)> + '_ {
        self.blocks
            .par_iter()
            .flat_map(|(_, block)| block.get_transactions())
            .map(|transaction| {
                std::iter::repeat(transaction).zip(transaction.get_outputs().iter().enumerate())
            })
            .flatten_iter()
            .filter_map(|(transaction, (output_index, output))| match output {
                TransactionOutput::ToInput { .. } => {
                    if self.is_output_spent(transaction.get_hash(), output_index as u32) {
                        None
                    } else {
                        Some((transaction, output, output_index))
                    }
                }

                TransactionOutput::ToPixel { .. } => None,
            })
    }

    pub fn find_transaction(&self, transaction_hash: &Hash) -> Option<&Transaction> {
        self.blocks
            .par_iter()
            .flat_map(|(_, block)| block.get_transactions())
            .find_any(|transaction| {
                let this_transaction_hash = transaction.get_hash();

                this_transaction_hash == transaction_hash
            })
    }

    pub fn create_simple_transaction(
        &mut self,
        sender: &Hash,
        recipient: &Hash,
        value: Credits,
        tax: Credits,
    ) -> Result<()> {
        debug_assert!(value > 0);
        debug_assert!(tax >= 0);

        // Collect unspent transactions to create the amount of credits needed.
        let unspent_outputs = self
            .get_all_unspent_outputs()
            .filter_map(|(transaction, output, output_index)| match output {
                TransactionOutput::ToInput {
                    value,
                    public_key_hash,
                } => {
                    if public_key_hash == sender {
                        Some((transaction, output_index, *value))
                    } else {
                        None
                    }
                }
                TransactionOutput::ToPixel { .. } => None,
            })
            .collect::<Vec<_>>();

        let (transactions, total) = {
            let mut unspent_outputs = unspent_outputs.iter();

            let total_target_value = value + tax;
            let mut total = 0;
            let mut outputs = vec![];
            while total < total_target_value {
                if let Some((transaction, output_index, value)) = unspent_outputs.next() {
                    total += value;
                    outputs.push((transaction.get_hash(), output_index));
                } else {
                    break;
                }
            }

            (outputs, total)
        };

        if total < value + tax {
            bail!("Not enough credits to make the transaction.")
        }

        // Create transaction that uses all the unpent transaction outputs necessary.
        let inputs = transactions
            .into_iter()
            .map(
                |(transaction_hash, output_index)| TransactionInput::FromOutput {
                    hash: *transaction_hash,
                    index: *output_index as u32,
                },
            )
            .collect();

        let value_output = TransactionOutput::ToInput {
            value,
            public_key_hash: *recipient,
        };

        let change_value = total - value - tax;
        let change_output = TransactionOutput::ToInput {
            value: change_value,
            public_key_hash: *sender,
        };

        let outputs = vec![value_output, change_output];

        let transaction = Transaction::try_new(self, inputs, outputs, 0)?;
        self.new_transaction(transaction)?;

        Ok(())
    }

    fn proof_of_work(&self) -> Proof {
        let last_block = self.get_last_block();
        let last_proof = last_block.get_proof();

        (0..Proof::MAX)
            .into_par_iter()
            .find_any(|possible_proof| Self::validate_proof(last_proof, possible_proof))
            .unwrap()
    }

    fn validate_proof(last_proof: &Proof, proof: &Proof) -> bool {
        let mut hasher = Sha3_256::default();
        hasher.update(&last_proof.to_le_bytes());
        hasher.update(&proof.to_le_bytes());

        let digest = hasher.finalize();
        let hash: Hash = digest.as_slice().try_into().unwrap();

        hash.iter().take(1).all(|e| *e == 0)
    }

    fn is_output_spent(&self, transaction_hash: &Hash, output_index: u32) -> bool {
        self.blocks
            .par_iter()
            .flat_map(|(_, block)| block.get_transactions())
            .flat_map(|transaction| transaction.get_inputs())
            .any(|input| match input {
                TransactionInput::FromOutput { hash, index } => {
                    *index == output_index && hash == transaction_hash
                }

                TransactionInput::FromReward { .. } => false, // Because miners automatically cache in rewards.
            })
    }
}
