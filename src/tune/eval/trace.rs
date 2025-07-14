use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::{eval::EvalScorePairType, tune::SparseTrace};

#[derive(Debug, Default, Clone, PartialEq)]
struct SparseTracePair {
	mg: SparseTrace,
	eg: SparseTrace
}

impl EvalScorePairType for SparseTracePair {
	type ScoreType = SparseTrace;

	fn mg(&self) -> Self::ScoreType {
		self.mg.clone()
	}

	fn eg(&self) -> Self::ScoreType {
		self.eg.clone()
	}
}

impl AddAssign for SparseTracePair {
    fn add_assign(&mut self, rhs: Self) {
		self.mg += rhs.mg;
		self.eg += rhs.eg;
    }
}

impl Add for SparseTracePair {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl SubAssign for SparseTracePair {
    fn sub_assign(&mut self, rhs: Self) {
        self.mg -= rhs.mg;
		self.eg -= rhs.eg;
    }
}

impl Sub for SparseTracePair {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result -= rhs;
        result
    }
}

impl Neg for SparseTracePair {
    type Output = Self;
    fn neg(self) -> Self::Output {
		Self {
			mg: -self.mg,
			eg: -self.eg
		}
    }
}

impl Mul<f32> for SparseTracePair {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
		Self {
			mg: self.mg * rhs,
			eg: self.eg * rhs
		}
    }
}

impl Div<f32> for SparseTracePair {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
		Self {
			mg: self.mg / rhs,
			eg: self.eg / rhs
		}
    }
}

impl Mul<i32> for SparseTracePair {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
		self * rhs as f32
    }
}

impl Div<i32> for SparseTracePair {
    type Output = Self;
    fn div(self, rhs: i32) -> Self::Output {
		self / rhs as f32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum EvalFeature {
	Material,
	Psqt,
	Mobility,
	PassedPawn,
	PawnPhalanx,
	DefendedPawn,
	SafeKnightCheck,
	SafeBishopCheck,
	SafeRookCheck,
	SafeQueenCheck,
	KingAttackerWeight,
	KingAttacks,
	ThreatByPawn,
	ThreatByKnight,
	ThreatByBishop,
	ThreatByRook,
	ThreatByQueen
}


