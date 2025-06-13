use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{Board, Move},
    policy::{self, PolicyValueType, PolicyValues},
    types::{Color, PieceType, Square},
};

#[derive(Debug, Default, Clone, PartialEq)]
struct SparseTrace {
    features: HashMap<u32, f32>,
}

impl SparseTrace {
    fn single(feature: u32) -> Self {
        Self {
            features: HashMap::from([(feature, 1.0)]),
        }
    }
}

impl AddAssign for SparseTrace {
    fn add_assign(&mut self, rhs: Self) {
        for (feature_idx, value) in rhs.features {
            self.features
                .entry(feature_idx)
                .and_modify(|e| *e += value)
                .or_insert(value);
        }
    }
}

impl Add for SparseTrace {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl SubAssign for SparseTrace {
    fn sub_assign(&mut self, rhs: Self) {
        for (feature_idx, value) in rhs.features {
            self.features
                .entry(feature_idx)
                .and_modify(|e| *e -= value)
                .or_insert(-value);
        }
    }
}

impl Sub for SparseTrace {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result -= rhs;
        result
    }
}

impl Neg for SparseTrace {
    type Output = Self;
    fn neg(self) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value = -*value;
        }
        result
    }
}

impl Mul<f32> for SparseTrace {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value *= rhs;
        }
        result
    }
}

impl Div<f32> for SparseTrace {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        let mut result = self.clone();
        for value in result.features.values_mut() {
            *value /= rhs;
        }
        result
    }
}

impl PolicyValueType for SparseTrace {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum PolicyFeature {
    CapBonus,
    PawnProtectedPenalty,
    PawnThreatEvasion,
    PsqtScore,
    PromoBonus,
    BadSeePenalty,
    CheckBonus,
}

impl PolicyFeature {
    pub const TOTAL_TERMS: u32 = 7;

    fn from_raw(raw: u32) -> Self {
        unsafe { std::mem::transmute(raw) }
    }

    fn ft_cnt(self) -> u32 {
        match self {
            Self::CapBonus => 5,
            Self::PawnProtectedPenalty => 5,
            Self::PawnThreatEvasion => 5,
            // 6 piece types * 64 squares * 2 phases
            Self::PsqtScore => 6 * 64 * 2,
            Self::PromoBonus => 2,
            Self::BadSeePenalty => 1,
            Self::CheckBonus => 1,
        }
    }

    fn ft_offset(self) -> u32 {
        let mut offset = 0;
        for i in 0..(self as u32) {
            offset += Self::ft_cnt(Self::from_raw(i));
        }
        offset
    }

    fn total_fts() -> u32 {
        let mut count = 0;
        for i in 0..Self::TOTAL_TERMS {
            count += Self::ft_cnt(Self::from_raw(i));
        }
        count
    }
}

use PolicyFeature::*;

struct PolicyTrace {}

impl PolicyTrace {}

impl PolicyValues for PolicyTrace {
    type Value = SparseTrace;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        let idx = match pt {
            PieceType::Pawn => CapBonus.ft_offset(),
            PieceType::Knight => CapBonus.ft_offset() + 1,
            PieceType::Bishop => CapBonus.ft_offset() + 2,
            PieceType::Rook => CapBonus.ft_offset() + 3,
            PieceType::Queen => CapBonus.ft_offset() + 4,
            _ => 0,
        };
        SparseTrace::single(idx)
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        let idx = match pt {
            PieceType::Pawn => PawnProtectedPenalty.ft_offset(),
            PieceType::Knight => PawnProtectedPenalty.ft_offset() + 1,
            PieceType::Bishop => PawnProtectedPenalty.ft_offset() + 2,
            PieceType::Rook => PawnProtectedPenalty.ft_offset() + 3,
            PieceType::Queen => PawnProtectedPenalty.ft_offset() + 4,
            _ => 0,
        };
        SparseTrace::single(idx)
    }

    fn pawn_threat_evasion(pt: PieceType) -> Self::Value {
        let idx = match pt {
            PieceType::Pawn => PawnThreatEvasion.ft_offset(),
            PieceType::Knight => PawnThreatEvasion.ft_offset() + 1,
            PieceType::Bishop => PawnThreatEvasion.ft_offset() + 2,
            PieceType::Rook => PawnThreatEvasion.ft_offset() + 3,
            PieceType::Queen => PawnThreatEvasion.ft_offset() + 4,
            _ => 0,
        };
        SparseTrace::single(idx)
    }

    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value {
        let mg_weight = phase.min(24) as f32 / 24.0;
        let eg_weight = 1.0 - mg_weight;

        let mg_offset =
            PsqtScore.ft_offset() + pt as u32 * 128 + sq.relative_sq(c).flip() as u32 * 2;
        let eg_offset = mg_offset + 1;

        SparseTrace {
            features: HashMap::from([(mg_offset, mg_weight), (eg_offset, eg_weight)]),
        }
    }

    fn promo_bonus(pt: PieceType) -> Self::Value {
        let idx = match pt {
            PieceType::Queen => PromoBonus.ft_offset(),
            _ => PromoBonus.ft_offset() + 1,
        };
        SparseTrace::single(idx)
    }

    fn bad_see_penalty() -> Self::Value {
        SparseTrace::single(BadSeePenalty.ft_offset())
    }

    fn check_bonus() -> Self::Value {
        SparseTrace::single(CheckBonus.ft_offset())
    }
}

pub fn compute_coeffs(board: &Board, mv: Move) -> Vec<(u32, f32)> {
    let trace = policy::get_policy_impl::<PolicyTrace>(board, mv);
    let mut result = Vec::new();

    for elem in trace.features {
        result.push(elem);
    }

    result
}

pub fn zero_params() -> Vec<f32> {
    let mut result = Vec::new();
    for i in 0..PolicyFeature::total_fts() {
        result.push(0.0);
    }
    result
}
