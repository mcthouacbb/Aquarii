use core::fmt;
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct CastleRights(u8);

impl CastleRights {
	pub const NONE: CastleRights = CastleRights(0);
	pub const WHITE_KING_SIDE: CastleRights = CastleRights(1);
	pub const WHITE_QUEEN_SIDE: CastleRights = CastleRights(2);
	pub const BLACK_KING_SIDE: CastleRights = CastleRights(4);
	pub const BLACK_QUEEN_SIDE: CastleRights = CastleRights(8);


	pub fn new(rights: u8) -> CastleRights {
		assert!(rights < 16, "Invalid castling rights");
		CastleRights(rights)
	}

	pub fn has(self, right: CastleRights) -> bool {
		assert!(right.0 < 16 && (right.0 & (right.0 - 1) == 0), "Invalid castling right bit");
		(self.0 & right.0) > 0
	}
}

impl ops::BitAnd for CastleRights {
	type Output = CastleRights;
	fn bitand(self, rhs: Self) -> Self::Output {
		CastleRights(self.0 & rhs.0)
	}
}

impl ops::BitAndAssign for CastleRights {
	fn bitand_assign(&mut self, rhs: Self) {
		self.0 &= rhs.0;
	}
}

impl ops::BitOr for CastleRights {
	type Output = CastleRights;
	fn bitor(self, rhs: Self) -> Self::Output {
		CastleRights(self.0 | rhs.0)
	}
}

impl ops::BitOrAssign for CastleRights {
	fn bitor_assign(&mut self, rhs: Self) {
		self.0 |= rhs.0;
	}
}

impl fmt::Display for CastleRights {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.has(CastleRights::WHITE_KING_SIDE) {
			write!(f, "K")?;
		}
		if self.has(CastleRights::WHITE_QUEEN_SIDE) {
			write!(f, "Q")?;
		}
		if self.has(CastleRights::BLACK_KING_SIDE) {
			write!(f, "k")?;
		}
		if self.has(CastleRights::BLACK_QUEEN_SIDE) {
			write!(f, "q")?;
		}
		Ok(())
	}
}
