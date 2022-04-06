use crate::{
    blockchain::{Hash, Proof},
    transaction::Transaction,
};
use chrono::{DateTime, Utc};
use tiny_keccak::{Hasher, Sha3};

#[derive(Debug)]
pub struct Block {
    index: u64,
    timestamp: DateTime<Utc>,
    transactions: Vec<Transaction>,
    proof: Proof,
    previous_hash: Option<Hash>,
}

impl Block {
    pub fn new(
        index: u64,
        transactions: Vec<Transaction>,
        proof: Proof,
        previous_hash: Option<Hash>,
    ) -> Self {
        Self {
            index,
            timestamp: Utc::now(),
            transactions,
            proof,
            previous_hash,
        }
    }

    pub fn get_index(&self) -> u64 {
        self.index
    }

    pub fn get_transactions(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_proof(&self) -> &Proof {
        &self.proof
    }

    pub fn calculate_hash(&self) -> Hash {
        let mut sha3 = Sha3::v256();

        sha3.update(&self.index.to_le_bytes());
        sha3.update(&self.timestamp.timestamp_nanos().to_le_bytes());

        for t in &self.transactions {
            match t {
                Transaction::Peer {
                    sender,
                    recipient,
                    amount,
                } => {
                    sha3.update(&sender.to_le_bytes());
                    sha3.update(&recipient.to_le_bytes());
                    sha3.update(&amount.to_le_bytes());
                }

                Transaction::Pixel {
                    sender,
                    position: coordinate,
                    color,
                } => {
                    sha3.update(&sender.to_le_bytes());

                    let (x, y) = coordinate;
                    sha3.update(&x.to_le_bytes());
                    sha3.update(&y.to_le_bytes());

                    let (r, g, b) = color;
                    sha3.update(&r.to_le_bytes());
                    sha3.update(&g.to_le_bytes());
                    sha3.update(&b.to_le_bytes());
                }
            }
        }

        sha3.update(&self.proof.to_le_bytes());

        if let Some(previous_hash) = &self.previous_hash {
            sha3.update(previous_hash);
        }

        let mut hash: Hash = Default::default();
        sha3.finalize(&mut hash);

        hash
    }
}
