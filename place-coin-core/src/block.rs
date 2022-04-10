use crate::{
    blockchain::{Hash, Proof},
    transaction::{Transaction, TransactionInput},
};
use anyhow::{bail, Context, Result};
use chrono::{serde::ts_nanoseconds, DateTime, Utc};
use serde::Serialize;
use sha3::{Digest, Sha3_256};

#[derive(Debug, Serialize)]
pub struct Block {
    #[serde(with = "ts_nanoseconds")]
    timestamp: DateTime<Utc>,

    transactions: Vec<Transaction>,
    proof: Proof,
    previous_hash: Option<Hash>,
}

impl Block {
    pub fn new(transactions: Vec<Transaction>, proof: Proof, previous_hash: Option<Hash>) -> Self {
        Self {
            timestamp: Utc::now(),
            transactions,
            proof,
            previous_hash,
        }
    }

    pub fn get_block_height(&self) -> Result<u64> {
        // The last transaction in a block must be the reward transactions. This has the block height.
        if let Some(last_transaction) = self.transactions.last() {
            let input = last_transaction
                .get_inputs()
                .first()
                .context("At least one input must exist.")?;

            if let TransactionInput::FromReward { height, .. } = input {
                Ok(*height)
            } else {
                bail!("No reward transaction found.")
            }
        } else {
            // This block doesn't have transactions. Lets check if its the genesis block.
            if self.previous_hash.is_none() {
                Ok(0)
            } else {
                bail!("Block height can't be found.")
            }
        }
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_proof(&self) -> &Proof {
        &self.proof
    }

    pub fn calculate_hash(&self) -> Hash {
        let encoded = bincode::serialize(self).unwrap();

        let mut hasher = Sha3_256::default();
        hasher.update(&encoded);

        let digest = hasher.finalize();
        let hash: Hash = digest.as_slice().try_into().unwrap();

        hash
    }
}
