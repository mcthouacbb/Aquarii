use std::fmt;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King
}

impl From<u8> for PieceType {
    fn from(value: u8) -> Self {
        assert!(value <= PieceType::King as u8);
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
    BlackKing
}

impl Piece {
    pub fn new(c: Color, pt: PieceType) -> Self {
        Piece::from((c as u8) | ((pt as u8) << 1))
    }

    pub fn color(self) -> Color {
        Color::from((self as u8) & 1)
    }

    pub fn piece_type(self) -> PieceType {
        PieceType::from((self as u8) >> 1)
    }

    pub fn char_repr(self) -> char {
        let chars = [
            'P', 'p', 'N', 'n', 'B', 'b', 'R', 'r', 'Q', 'q', 'K', 'k'
        ];
        chars[self as usize]
    }
}

impl From<u8> for Piece {
    fn from(value: u8) -> Self {
        assert!(value <= Piece::BlackKing as u8);
        unsafe { std::mem::transmute(value) }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.color(), self.piece_type())
    }
}
