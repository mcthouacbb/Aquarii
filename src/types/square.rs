use std::ops;

#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub enum Square {
	A1, B1, C1, D1, E1, F1, G1, H1,
	A2, B2, C2, D2, E2, F2, G2, H2,
	A3, B3, C3, D3, E3, F3, G3, H3,
	A4, B4, C4, D4, E4, F4, G4, H4,
	A5, B5, C5, D5, E5, F5, G5, H5,
	A6, B6, C6, D6, E6, F6, G6, H6,
	A7, B7, C7, D7, E7, F7, G7, H7,
	A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
	pub fn from_rank_file(rank: u8, file: u8) -> Square {
		return Square::from(rank * 8 + file)
	}

	pub fn value(self) -> u8 {
		self as u8
	}

	pub fn rank(self) -> u8 {
		self.value() / 8
	}

	pub fn file(self) -> u8 {
		self.value() % 8
	}
}

impl From<u8> for Square {
	fn from(value: u8) -> Self {
		assert!(value <= Square::H8 as u8);
		unsafe { std::mem::transmute(value) }
	}
}

impl ops::Add<u8> for Square {
	type Output = Self;
	fn add(self, rhs: u8) -> Self::Output {
		return Square::from(self.value() + rhs);
	}
}

impl ops::AddAssign<u8> for Square {
	fn add_assign(&mut self, rhs: u8) {
		*self = *self + rhs;
	}
}

impl ops::Sub<u8> for Square {
	type Output = Self;
	fn sub(self, rhs: u8) -> Self::Output {
		assert!(self.value() >= rhs);
		return Square::from(self.value() - rhs);
	}
}

impl ops::SubAssign<u8> for Square {
	fn sub_assign(&mut self, rhs: u8) {
		*self = *self - rhs;
	}
}
