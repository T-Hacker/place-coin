pub mod address;
pub mod block;
pub mod blockchain;
pub mod color;
pub mod signature;
pub mod transaction;

pub const CURRENT_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use crate::{
        address::Address,
        blockchain::{Blockchain, Hash},
        signature::Signature,
        transaction::{Credits, TransactionOutput},
    };
    use anyhow::Result;
    use rand_core::OsRng;
    use rayon::iter::ParallelIterator;

    const MY_NODE_ID: Hash = [1; 32];
    const OTHER_NODE_ID: Hash = [8; 32];

    fn setup_blockchain() -> Result<Blockchain> {
        let miner_address = Address::from_private_key(&MY_NODE_ID);
        let mut blockchain = Blockchain::new(miner_address);

        // Mine a block.
        blockchain.mine()?;

        // Validate block mined.
        assert!(blockchain.get_last_block().is_valid(&blockchain));

        Ok(blockchain)
    }

    #[test]
    fn test_simple_transaction() -> Result<()> {
        let mut blockchain = setup_blockchain()?;

        // Create a new transaction to transfer some credits.
        let sender_address = Address::from_private_key(&MY_NODE_ID);
        let recipient_address = Address::from_private_key(&OTHER_NODE_ID);

        blockchain.create_simple_transaction(
            &MY_NODE_ID,
            &sender_address,
            &recipient_address,
            &OTHER_NODE_ID,
            99,
            5,
        )?;

        // Mine to commit the new block.
        blockchain.mine()?;

        // Validate block mined.
        assert!(blockchain.get_last_block().is_valid(&blockchain));

        // Check if credits were transferred.
        let credits = blockchain.get_peer_credits(&recipient_address);
        assert_eq!(credits, 99, "Peer did not receive the credits.");

        let total_unspent_credits = blockchain
            .get_all_unspent_outputs()
            .map(|(_, output, _)| match output {
                TransactionOutput::ToInput { value, .. } => *value,
                TransactionOutput::ToPixel { .. } => 0,
            })
            .sum::<Credits>();

        assert_eq!(total_unspent_credits, 2000);

        Ok(())
    }

    // #[test]
    // fn test_paint_pixel() -> Result<()> {
    //     let mut blockchain = setup_blockchain()?;

    //     // Paint a pixel.
    //     blockchain.add_transaction(Transaction::Pixel {
    //         sender: MY_NODE_ID,
    //         position: (0, 0),
    //         color: (0xFF, 0, 0),
    //     })?;

    //     // Mine again.
    //     let proof = blockchain.proof_of_work();

    //     // Reward ourselves.
    //     blockchain.add_transaction(Transaction::Peer {
    //         sender: 0,
    //         recipient: MY_NODE_ID,
    //         amount: 1000,
    //     })?;

    //     // Forge a new block.
    //     let last_block = blockchain.get_last_block();
    //     let previous_hash = last_block.calculate_hash();
    //     let _block = blockchain.new_block(proof, previous_hash);

    //     let credits = blockchain.get_peer_credits(&MY_NODE_ID, None);
    //     assert_eq!(credits, 1999);

    //     Ok(())
    // }

    #[test]
    fn test_signature_by_changing_one_byte() -> Result<()> {
        let mut hash: Hash = [0xFF; 32];

        let private_key = k256::SecretKey::random(&mut OsRng);
        let private_key = private_key.to_be_bytes();
        let private_key = private_key.as_slice();
        let signature_1 = Signature::new(private_key.try_into()?, &hash);

        hash[16] = 0;

        let signature_2 = Signature::new(private_key.try_into()?, &hash);

        assert_ne!(signature_1, signature_2);

        Ok(())
    }

    #[test]
    fn test_signature_with_different_private_keys() -> Result<()> {
        let hash: Hash = [0xFF; 32];

        let private_key_1 = k256::SecretKey::random(&mut OsRng);
        let private_key_1 = private_key_1.to_be_bytes();
        let private_key_1 = private_key_1.as_slice();
        let signature_1 = Signature::new(private_key_1.try_into()?, &hash);

        let private_key_2 = k256::SecretKey::random(&mut OsRng);
        let private_key_2 = private_key_2.to_be_bytes();
        let private_key_2 = private_key_2.as_slice();
        let signature_2 = Signature::new(private_key_2.try_into()?, &hash);

        assert_ne!(signature_1, signature_2);

        Ok(())
    }
}
