pub mod block;
pub mod blockchain;
pub mod transaction;

pub const CURRENT_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::{Blockchain, Hash},
        transaction::{Credits, Transaction, TransactionInput, TransactionOutput},
    };
    use anyhow::Result;
    use rayon::iter::ParallelIterator;

    const MY_NODE_ID: Hash = [1; 32];
    const OTHER_NODE_ID: Hash = [8; 32];

    fn setup_blockchain() -> Result<Blockchain> {
        let mut blockchain = Blockchain::new(MY_NODE_ID);

        // Mine a block.
        blockchain.mine()?;

        Ok(blockchain)
    }

    #[test]
    fn test_simple_transaction() -> Result<()> {
        let mut blockchain = setup_blockchain()?;

        // Create a new transaction to transfer some credits.
        let last_block = blockchain.get_last_block();
        let last_transaction = last_block.get_transactions().first().unwrap();

        let last_transaction_hash = last_transaction.get_hash();

        let inputs = vec![TransactionInput::FromOutput {
            hash: *last_transaction_hash,
            index: 0,
        }];

        let outputs = vec![TransactionOutput::ToInput {
            value: 99,
            public_key_hash: OTHER_NODE_ID,
        }];

        let transaction = Transaction::try_new(&blockchain, inputs, outputs, 0)?;
        blockchain.new_transaction(transaction)?;

        // Mine to commit the new block.
        blockchain.mine()?;

        // Check if credits were transferred.
        let credits = blockchain.get_peer_credits(&OTHER_NODE_ID);
        assert_eq!(credits, 99, "Peer did not receive the credits.");

        let total_unspent_credits = blockchain
            .get_all_unspent_outputs()
            .map(|output| match output {
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
}
