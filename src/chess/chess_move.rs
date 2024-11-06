use crate::types::{PieceType, Square};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MoveKind {
    None,
    Enpassant,
    Castle,
    Promotion
}

impl From<u8> for MoveKind {
	fn from(value: u8) -> Self {
		assert!(value <= MoveKind::Promotion as u8);
		unsafe { std::mem::transmute(value) }
	}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    data: u16,
}

impl Move {
    fn new(from: Square, to: Square, kind: MoveKind, promo: u8) -> Self {
        Self { data: from as u16 | ((to as u16) << 6) | ((kind as u16) << 12) | ((promo as u16) << 14) }
    }

    pub fn normal(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::None, 0)
    }

    pub fn castle(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::Castle, 0)
    }

    pub fn enpassant(from: Square, to: Square) -> Self {
        Self::new(from, to, MoveKind::Enpassant, 0)
    }

    pub fn promo(from: Square, to: Square, promo: PieceType) -> Self {
        Self::new(from, to, MoveKind::Promotion, promo as u8 - PieceType::Knight as u8)
    }

	pub fn from_sq(&self) -> Square {
		Square::from((self.data & 63) as u8)
	}

	pub fn to_sq(&self) -> Square {
		Square::from(((self.data >> 6) & 63) as u8)
	}

	pub fn kind(&self) -> MoveKind {
		MoveKind::from(((self.data >> 12) & 3) as u8)
	}

	pub fn promo_piece(&self) -> PieceType {
		PieceType::from(((self.data >> 14) + PieceType::Knight as u16) as u8)
	}
}
