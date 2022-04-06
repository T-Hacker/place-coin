pub mod block;
pub mod blockchain;
pub mod transaction;

#[cfg(test)]
mod tests {
    use crate::{
        blockchain::Blockchain,
        transaction::{Address, Transaction},
    };
    use anyhow::Result;

    const MY_NODE_ID: Address = 1337;

    fn setup_blockchain() -> Result<Blockchain> {
        let mut blockchain = Blockchain::default();

        // Mine a block.
        let proof = blockchain.proof_of_work();

        // Reward ourselves.
        blockchain.add_transaction(Transaction::Peer {
            sender: 0,
            recipient: MY_NODE_ID,
            amount: 1000,
        })?;

        // Forge a new block.
        let last_block = blockchain.get_last_block();
        let previous_hash = last_block.calculate_hash();
        let _block = blockchain.new_block(proof, previous_hash);

        Ok(blockchain)
    }

    #[test]
    fn test_simple_transaction() -> Result<()> {
        let mut blockchain = setup_blockchain()?;

        // Transfer some credits.
        blockchain.add_transaction(Transaction::Peer {
            sender: MY_NODE_ID,
            recipient: 123,
            amount: 9,
        })?;

        // Mine again.
        let proof = blockchain.proof_of_work();

        // Reward ourselves.
        blockchain.add_transaction(Transaction::Peer {
            sender: 0,
            recipient: MY_NODE_ID,
            amount: 1000,
        })?;

        // Forge a new block.
        let last_block = blockchain.get_last_block();
        let previous_hash = last_block.calculate_hash();
        let _block = blockchain.new_block(proof, previous_hash);

        let my_credits = blockchain.get_peer_credits(&MY_NODE_ID, None);
        assert_eq!(my_credits, 1991);

        let other_credits = blockchain.get_peer_credits(&123, None);
        assert_eq!(other_credits, 9);

        Ok(())
    }

    #[test]
    fn test_paint_pixel() -> Result<()> {
        let mut blockchain = setup_blockchain()?;

        // Paint a pixel.
        blockchain.add_transaction(Transaction::Pixel {
            sender: MY_NODE_ID,
            position: (0, 0),
            color: (0xFF, 0, 0),
        })?;

        // Mine again.
        let proof = blockchain.proof_of_work();

        // Reward ourselves.
        blockchain.add_transaction(Transaction::Peer {
            sender: 0,
            recipient: MY_NODE_ID,
            amount: 1000,
        })?;

        // Forge a new block.
        let last_block = blockchain.get_last_block();
        let previous_hash = last_block.calculate_hash();
        let _block = blockchain.new_block(proof, previous_hash);

        let credits = blockchain.get_peer_credits(&MY_NODE_ID, None);
        assert_eq!(credits, 1999);

        Ok(())
    }
}
