use core::fmt;
use std::ops;

use super::Square;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Bitboard(u64);

impl Bitboard {
	pub const FILE_A: Bitboard = Bitboard(0x0101010101010101u64);
	pub const FILE_B: Bitboard = Bitboard(0x0202020202020202u64);
	pub const FILE_C: Bitboard = Bitboard(0x0404040404040404u64);
	pub const FILE_D: Bitboard = Bitboard(0x0808080808080808u64);
	pub const FILE_E: Bitboard = Bitboard(0x1010101010101010u64);
	pub const FILE_F: Bitboard = Bitboard(0x2020202020202020u64);
	pub const FILE_G: Bitboard = Bitboard(0x4040404040404040u64);
	pub const FILE_H: Bitboard = Bitboard(0x8080808080808080u64);
	
	pub const RANK_1: Bitboard = Bitboard(0x00000000000000FFu64);
	pub const RANK_2: Bitboard = Bitboard(0x000000000000FF00u64);
	pub const RANK_3: Bitboard = Bitboard(0x0000000000FF0000u64);
	pub const RANK_4: Bitboard = Bitboard(0x00000000FF000000u64);
	pub const RANK_5: Bitboard = Bitboard(0x000000FF00000000u64);
	pub const RANK_6: Bitboard = Bitboard(0x0000FF0000000000u64);
	pub const RANK_7: Bitboard = Bitboard(0x00FF000000000000u64);
	pub const RANK_8: Bitboard = Bitboard(0xFF00000000000000u64);

	pub const EMPTY: Bitboard = Bitboard(0u64);
	pub const ALL: Bitboard = Bitboard(!0u64);

	pub fn from_square(sq: Square) -> Bitboard {
		return Bitboard(1 << sq.value());
	}

	pub fn value(self) -> u64 {
		self.0
	}

	pub fn north(self) -> Bitboard {
		Bitboard(self.0 << 8)
	}

	pub fn south(self) -> Bitboard {
		Bitboard(self.0 >> 8)
	}

	pub fn west(self) -> Bitboard {
		Bitboard(self.0 >> 1) & !Self::FILE_A
	}

	pub fn east(self) -> Bitboard {
		Bitboard(self.0 << 1) & !Self::FILE_H
	}

	pub fn north_west(self) -> Bitboard {
		self.north().west()
	}

	pub fn north_east(self) -> Bitboard {
		self.north().east()
	}

	pub fn south_west(self) -> Bitboard {
		self.south().west()
	}

	pub fn south_east(self) -> Bitboard {
		self.south().east()
	}

	pub fn lsb(self) -> Square {
		Square::from(self.0.trailing_zeros() as u8)
	}

	pub fn msb(self) -> Square {
		Square::from((63 - self.0.leading_zeros()) as u8)
	}

	pub fn popcount(self) -> u32 {
		self.0.count_ones()
	}

	pub fn poplsb(&mut self) -> Square {
		let lsb = self.lsb();
		self.0 &= self.0 - 1;
		lsb
	}

	pub fn any(self) -> bool {
		self.0 > 0
	}

	pub fn empty(self) -> bool {
		self.0 == 0
	}

	pub fn multiple(self) -> bool {
		(self.0 & (self.0 - 1)) > 0
	}

	pub fn one(self) -> bool {
		self.any() && !self.multiple()
	}

	pub fn has(self, sq: Square) -> bool {
		return ((self.0 >> (sq as u8)) & 1u64) > 0;
	}
}

impl ops::BitAnd<Bitboard> for Bitboard {
	type Output = Bitboard;
	fn bitand(self, rhs: Bitboard) -> Self::Output {
		Bitboard(self.0 & rhs.0)
	}
}

impl ops::BitAndAssign<Bitboard> for Bitboard {
	fn bitand_assign(&mut self, rhs: Bitboard) {
		*self = *self & rhs;
	}
}

impl ops::BitOr<Bitboard> for Bitboard {
	type Output = Bitboard;
	fn bitor(self, rhs: Bitboard) -> Self::Output {
		Bitboard(self.0 | rhs.0)
	}
}

impl ops::BitOrAssign<Bitboard> for Bitboard {
	fn bitor_assign(&mut self, rhs: Bitboard) {
		*self = *self | rhs;
	}
}

impl ops::BitXor<Bitboard> for Bitboard {
	type Output = Bitboard;
	fn bitxor(self, rhs: Bitboard) -> Self::Output {
		Bitboard(self.0 ^ rhs.0)
	}
}

impl ops::BitXorAssign<Bitboard> for Bitboard {
	fn bitxor_assign(&mut self, rhs: Bitboard) {
		*self = *self ^ rhs;
	}
}

impl ops::Not for Bitboard {
	type Output = Bitboard;
	fn not(self) -> Self::Output {
		Bitboard(!self.0)
	}
}

fn reverse(mut n: u8) -> u8 {
    n = (n & 0xF0) >> 4 | (n & 0x0F) << 4;
    n = (n & 0xCC) >> 2 | (n & 0x33) << 2;
    n = (n & 0xAA) >> 1 | (n & 0x55) << 1;
    return n;
}

impl fmt::Display for Bitboard {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		// this is cursed
		let mut bb = self.0;
		for _ in 0..8 {
			let row: u64 = bb >> 56;
			write!(f, "{:08b}\n", reverse(row as u8))?;
			bb <<= 8;
		}
		Ok(())
	}
}
