use std::fmt;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        assert!(value <= PieceType::None as u8);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    White,
    Black
}

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        assert!(value <= Color::Black as u8);
        unsafe { std::mem::transmute(value) }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Piece(u8);

impl Piece {
    pub const NONE: Piece = Piece(0);
    pub fn new(c: Color, pt: PieceType) -> Self {
        Piece((c as u8) | ((pt as u8) << 1))
    }

    pub fn color(self) -> Color {
        Color::from(self.0 & 1)
    }

    pub fn piece_type(self) -> PieceType {
        PieceType::from(self.0 >> 1)
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}{:?}", self.color(), self.piece_type())
    }
}
