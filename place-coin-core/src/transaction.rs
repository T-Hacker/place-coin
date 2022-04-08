use crate::blockchain::{Blockchain, Hash};
use anyhow::{bail, Context, Result};
use serde::{ser::SerializeSeq, Serialize};
use tiny_keccak::{Hasher, Sha3};

pub type Version = u32;
pub type Point = (i32, i32);
pub type Color = (u8, u8, u8);
pub type Credits = i64;

const CURRENT_TRANSACTION_VERSION: u32 = 0;

#[derive(Debug, Serialize)]
pub enum TransactionInput {
    FromOutput { hash: Hash, index: u32 },
    FromReward { height: u64, value: Credits },
}

#[derive(Debug, Serialize)]
pub enum TransactionOutput {
    ToInput {
        value: Credits,
        public_key_hash: Hash,
    },

    ToPixel {
        value: Credits,
        position: Point,
        color: Color,
    },
}

#[derive(Debug, Serialize)]
struct TransactionData {
    version: Version,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    lock_time: u32,
}

#[derive(Debug)]
pub struct Transaction {
    data: TransactionData,
    balance: Credits,
    hash: Hash,
}

impl Transaction {
    pub fn try_new(
        blockchain: &Blockchain,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u32,
    ) -> Result<Self> {
        // Calculate balance.
        let input_value: Credits = inputs
            .iter()
            .map(|input| match input {
                TransactionInput::FromOutput { hash, index } => {
                    let input_transaction = blockchain
                        .find_transaction(hash)
                        .context("Fail to find input transaction.")?;

                    let output = input_transaction
                        .get_outputs()
                        .get(*index as usize)
                        .context("Fail to find output in input transaction.")?;

                    match output {
                        TransactionOutput::ToInput { value, .. } => Ok(*value),
                        TransactionOutput::ToPixel { .. } => {
                            bail!("Mismatch output type in the input transaction.")
                        }
                    }
                }

                TransactionInput::FromReward { value, .. } => Ok(*value),
            })
            .collect::<Result<Vec<_>>>()?
            .iter()
            .sum();

        let output_value: Credits = outputs
            .iter()
            .map(|output| match output {
                TransactionOutput::ToInput { value, .. } => *value,
                TransactionOutput::ToPixel { value, .. } => *value,
            })
            .sum();

        let balance = input_value - output_value;
        if balance < 0 {
            bail!("A transaction can't have a negative balance.")
        }

        // Create inner data.
        let data = TransactionData {
            version: CURRENT_TRANSACTION_VERSION,
            inputs,
            outputs,
            lock_time,
        };

        // Calculate hash.
        let encoded = bincode::serialize(&data).unwrap();

        let mut hasher = Sha3::v256();
        hasher.update(&encoded);

        let mut hash: Hash = Default::default();
        hasher.finalize(&mut hash);

        // Return final type.
        Ok(Self {
            data,
            balance,
            hash,
        })
    }

    pub fn get_version(&self) -> Version {
        self.data.version
    }

    pub fn get_inputs(&self) -> &[TransactionInput] {
        &self.data.inputs
    }

    pub fn get_outputs(&self) -> &[TransactionOutput] {
        &self.data.outputs
    }

    pub fn get_lock_time(&self) -> u32 {
        self.data.lock_time
    }

    pub fn get_balance(&self) -> Credits {
        self.balance
    }

    pub fn get_hash(&self) -> &Hash {
        &self.hash
    }
}

impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(1))?;

        seq.serialize_element(&self.data)?;

        seq.end()
    }
}