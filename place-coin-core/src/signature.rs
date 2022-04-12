use crate::{
    address::{PrivateKey, PublicKey},
    blockchain::Hash,
};
use ecdsa::signature::Signer;
use serde::Serialize;
use sha3::{Digest, Sha3_256};

#[derive(Debug, PartialEq, Eq)]
pub struct Signature([u8; 64]);

impl Signature {
    pub fn new(private_key: &PrivateKey, hash: &Hash) -> Self {
        let signature_key = k256::ecdsa::SigningKey::from_bytes(private_key).unwrap();
        let signature: k256::ecdsa::Signature = signature_key.sign(hash);
        let signature = signature.to_vec();

        Self(signature.as_slice().try_into().unwrap())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

pub fn sign_transaction(
    transaction_hash: &Hash,
    output_index: u32,
    public_key: &PublicKey,
    private_key: &PrivateKey,
) -> Signature {
    let mut hasher = Sha3_256::default();
    hasher.update(transaction_hash);
    hasher.update(output_index.to_le_bytes());
    hasher.update(public_key);

    let hash: Hash = hasher.finalize().as_slice().try_into().unwrap();

    Signature::new(private_key, &hash)
}
