use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Serialize, Deserialize)]
pub enum Color {
    White,
    Black,

    Gray,
    Brown,
    Blue,
    Green,
    Teal,
    Pink,
    Purple,
    Red,
    Yellow,
    Indigo,

    DarkGray,
    DarkBrown,
    DarkBlue,
    DarkGreen,
    DarkTeal,
    DarkPink,
    DarkPurple,
    DarkRed,
    DarkYellow,
    DarkIndigo,

    LightGray,
    LightBrown,
    LightBlue,
    LightGreen,
    LightTeal,
    LightPink,
    LightPurple,
    LightRed,
    LightYellow,
    LightIndigo,
}
