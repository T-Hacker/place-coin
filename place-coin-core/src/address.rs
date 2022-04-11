use crate::blockchain::Hash;
use sha3::{digest::Digest, Sha3_256};

pub type PrivateKey = Hash; // Private keys have the same size of hashes.

const CURRENT_VERSION: u32 = 0;

pub struct Address {
    version: u32,
    hash: Hash,
    crc32: u32,
}

impl Address {
    pub fn from_private_key(private_key: PrivateKey) -> Self {
        // Get public key from the private one.
        let private_key = k256::SecretKey::from_be_bytes(&private_key).unwrap();

        // Create hash of pubic key.
        let public_key = private_key.public_key();
        let public_key_bytes = bincode::serialize(&public_key).unwrap();

        let hasher = Sha3_256::default();
        hasher.update(&CURRENT_VERSION.to_le_bytes());
        hasher.update(&public_key_bytes);

        let hash = hasher.finalize();

        Self {
            version: CURRENT_VERSION,
            hash: hash.as_slice().try_into().unwrap(),
            crc32: 0, // TODO: Calculate CRC32.
        }
    }
}
