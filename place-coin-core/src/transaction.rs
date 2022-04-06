pub type Address = u128;
pub type Point = (i32, i32);
pub type Color = (u8, u8, u8);
pub type Credits = i64;

#[derive(Debug)]
pub enum Transaction {
    Peer {
        sender: Address,
        recipient: Address,
        amount: Credits,
    },

    Pixel {
        sender: Address,
        position: Point,
        color: Color,
    },
}
