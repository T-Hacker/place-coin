use crate::blockchain::Hash;
use serde::{de::Visitor, Deserialize, Serialize};
use sha3::{digest::Digest, Sha3_256};

pub type PrivateKey = Hash; // Private keys have the same size of hashes.
pub type PublicKey = PrivateKey;

const CURRENT_VERSION: u8 = 0;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Address {
    base58: String,
}

impl Address {
    pub fn from_private_key(private_key: &PrivateKey) -> Self {
        // Get public key from the private one.
        let private_key = k256::SecretKey::from_be_bytes(private_key).unwrap();

        // Create hash of pubic key.
        let public_key = private_key.public_key();
        let public_key_bytes = bincode::serialize(&public_key).unwrap();

        let mut hasher = Sha3_256::default();
        hasher.update(&public_key_bytes);

        let public_key_hash: Hash = hasher.finalize().as_slice().try_into().unwrap();

        // Create checksum by double hashing the version and the public key.
        let checksum = Self::calculate_checksum(CURRENT_VERSION, public_key_hash);
        let checksum = checksum.as_slice();

        // Create the address by concatenating the version, public key hash and checksum.
        let mut value = vec![CURRENT_VERSION];
        value.extend_from_slice(&public_key_hash);
        value.extend_from_slice(checksum);

        // Encode result in base58 format.
        let base58 = bs58::encode(&value).into_string();

        Self { base58 }
    }

    pub fn from_string(base58: &str) -> Self {
        Self {
            base58: base58.to_string(),
        }
    }

    pub fn as_str(&self) -> &str {
        self.base58.as_str()
    }

    pub fn validate(&self) -> bool {
        // Decode address.
        let address = {
            if let Ok(address) = bs58::decode(&self.base58).into_vec() {
                address
            } else {
                return false;
            }
        };

        // Check if address has the correct size.
        if address.len() != 1 + 32 + 4 {
            return false;
        }

        let version = &address[0];
        let public_key_hash = &address[1..33];
        let checksum = &address[33..];

        // Check if address has the correct version.
        if *version != CURRENT_VERSION {
            return false;
        }

        // Check if the checksum matches.
        let check =
            &Self::calculate_checksum(*version, public_key_hash.try_into().unwrap_or_default())
                [..4];

        if check != checksum {
            return false;
        }

        true
    }

    fn calculate_checksum(version: u8, public_key_hash: Hash) -> Vec<u8> {
        let mut hasher = Sha3_256::default();
        hasher.update(&[version]);
        hasher.update(&public_key_hash);
        let hash: Hash = hasher.finalize().as_slice().try_into().unwrap();

        let mut hasher = Sha3_256::default();
        hasher.update(hash);

        let hash: Hash = hasher.finalize().as_slice().try_into().unwrap();

        hash[..4].to_vec()
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

struct AddressVisitor;

impl<'de> Visitor<'de> for AddressVisitor {
    type Value = Address;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string containing a valid address.")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Address::from_string(v))
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(AddressVisitor)
    }
}
