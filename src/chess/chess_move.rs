use crate::types::{PieceType, Square};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    data: u16,
}

pub enum MoveKind {
    None,
    Enpassant,
    Castle,
    Promotion,
}

impl Move {
    fn from_raw(data: u16) -> Self {
        Self { data: data }
    }

    pub fn new(from: Square, to: Square, kind: MoveKind) -> Self {
        return Self::from_raw();
    }
}
