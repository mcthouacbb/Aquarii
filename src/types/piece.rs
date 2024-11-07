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
}

impl PieceType {
    pub const fn from_raw(value: u8) -> Self {
        assert!(value <= Self::King as u8);
        unsafe { std::mem::transmute(value) }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const fn from_raw(value: u8) -> Self {
        assert!(value <= Self::Black as u8);
        unsafe { std::mem::transmute(value) }
    }
}

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum Piece {
    WhitePawn,
    BlackPawn,
    WhiteKnight,
    BlackKnight,
    WhiteBishop,
    BlackBishop,
    WhiteRook,
    BlackRook,
    WhiteQueen,
    BlackQueen,
    WhiteKing,
    BlackKing,
}

impl Piece {
    pub const fn from_raw(value: u8) -> Self {
        assert!(value <= Self::BlackKing as u8);
        unsafe { std::mem::transmute(value) }
    }

    pub const fn new(c: Color, pt: PieceType) -> Self {
        Self::from_raw((c as u8) | ((pt as u8) << 1))
    }

    pub const fn color(self) -> Color {
        Color::from_raw((self as u8) & 1)
    }

    pub const fn piece_type(self) -> PieceType {
        PieceType::from_raw((self as u8) >> 1)
    }

    pub const fn char_repr(self) -> char {
        let chars = ['P', 'p', 'N', 'n', 'B', 'b', 'R', 'r', 'Q', 'q', 'K', 'k'];
        chars[self as usize]
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.color(), self.piece_type())
    }
}
