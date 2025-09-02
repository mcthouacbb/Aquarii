use std::collections::HashMap;

use crate::{
    chess::{Board, Move},
    policy::{self, PolicyValues},
    tune::SparseTrace,
    types::{Color, PieceType, Square},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum PolicyFeature {
    CapBonus,
    PawnProtectedPenalty,
    ThreatEvasion,
    PsqtScore,
    PassedPawnPush,
    Threat,
    PromoBonus,
    BadSeePenalty,
    CheckBonus,
}

use PolicyFeature::*;

impl PolicyFeature {
    pub const TOTAL_FEATURES: u32 = 9;

    fn from_raw(raw: u32) -> Self {
        unsafe { std::mem::transmute(raw) }
    }

    fn ft_cnt(self) -> u32 {
        match self {
            Self::CapBonus => 5,
            Self::PawnProtectedPenalty => 5,
            Self::ThreatEvasion => 5 * 5,
            // 6 piece types * 64 squares * 2 phases
            Self::PsqtScore => 6 * 64 * 2,
            Self::PassedPawnPush => 2 * 8,
            // 5 attackers * 5 victims
            Self::Threat => 5 * 5,
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

    fn iter() -> impl Iterator<Item = Self> {
        (0..Self::TOTAL_FEATURES).map(|i| Self::from_raw(i))
    }

    fn total_fts() -> u32 {
        let mut count = 0;
        for feature in Self::iter() {
            count += Self::ft_cnt(feature);
        }
        count
    }

    fn format_single(params: &Vec<f32>, offset: u32) -> String {
        format!("{:.3}", params[offset as usize])
    }

    #[allow(non_snake_case)]
    fn format_array_1D(params: &Vec<f32>, offset: u32, size: u32) -> String {
        let mut result = "[".to_owned();
        for i in offset..(offset + size) {
            if i != offset + size - 1 {
                result += format!("{:.3}, ", params[i as usize]).as_str();
            } else {
                result += format!("{:.3}", params[i as usize]).as_str();
            }
        }
        result + "]"
    }

    #[allow(non_snake_case)]
    fn format_array_2D(params: &Vec<f32>, offset: u32, size1: u32, size2: u32) -> String {
        let mut result = "[\n".to_owned();
        for i in 0..size1 {
            result += "    ";
            result += Self::format_array_1D(params, offset + size2 * i, size2).as_str();
            result += ",\n";
        }
        result + "]"
    }

    fn format_single_feature(feature: Self, params: &Vec<f32>) -> String {
        match feature {
            Self::CapBonus => Self::format_cap_bonus(params),
            Self::PawnProtectedPenalty => Self::format_pawn_protected_penalty(params),
            Self::ThreatEvasion => Self::pawn_threat_evasion(params),
            Self::PsqtScore => Self::format_psqt_score(params),
            Self::PassedPawnPush => Self::format_passed_pawn_push(params),
            Self::Threat => Self::format_threat(params),
            Self::PromoBonus => Self::format_promo_bonus(params),
            Self::BadSeePenalty => Self::format_bad_see_penalty(params),
            Self::CheckBonus => Self::format_check_bonus(params),
        }
    }

    fn format_cap_bonus(params: &Vec<f32>) -> String {
        "const CAP_BONUS: [f32; 5] = ".to_owned()
            + Self::format_array_1D(params, CapBonus.ft_offset(), CapBonus.ft_cnt()).as_str()
    }

    fn format_pawn_protected_penalty(params: &Vec<f32>) -> String {
        "const PAWN_PROTECTED_PENALTY: [f32; 5] = ".to_owned()
            + Self::format_array_1D(
                params,
                PawnProtectedPenalty.ft_offset(),
                PawnProtectedPenalty.ft_cnt(),
            )
            .as_str()
    }

    fn pawn_threat_evasion(params: &Vec<f32>) -> String {
        "const THREAT_EVASION: [[f32; 5]; 5] = ".to_owned()
            + Self::format_array_2D(params, ThreatEvasion.ft_offset(), 5, 5).as_str()
    }

    fn format_psqt_score(params: &Vec<f32>) -> String {
        let mut result = "const PSQT_SCORE: [[(f32, f32); 64]; 6] = [\n".to_owned();
        for pt in 0..6 {
            result += "    [\n";
            for y in 0..8 {
                result += "        ";
                for x in 0..8 {
                    let mg_offset = PsqtScore.ft_offset() + pt * 64 * 2 + y * 8 * 2 + x * 2;
                    let eg_offset = mg_offset + 1;
                    result += format!(
                        "S({:.3}, {:.3}),",
                        params[mg_offset as usize], params[eg_offset as usize]
                    )
                    .as_str();
                    if x != 7 {
                        result += " ";
                    }
                }
                result += "\n";
            }
            result += "    ],\n";
        }
        result + "]"
    }

    fn format_passed_pawn_push(params: &Vec<f32>) -> String {
        let mut result = "const PASSED_PAWN_PUSH: [(f32, f32); 8] = [\n    ".to_owned();
        for rank in 0..8 {
            let mg_offset = PassedPawnPush.ft_offset() + 2 * rank;
            let eg_offset = mg_offset + 1;
            result += format!(
                "S({:.3}, {:.3})",
                params[mg_offset as usize], params[eg_offset as usize]
            )
            .as_str();
            if rank != 7 {
                result += ", ";
            }
        }
        result + "]"
    }

    fn format_threat(params: &Vec<f32>) -> String {
        "const THREAT: [[f32; 5]; 5] = ".to_owned()
            + Self::format_array_2D(params, Threat.ft_offset(), 5, 5).as_str()
    }

    fn format_promo_bonus(params: &Vec<f32>) -> String {
        "const PROMO_BONUS: [f32; 2] = ".to_owned()
            + Self::format_array_1D(params, PromoBonus.ft_offset(), PromoBonus.ft_cnt()).as_str()
    }

    fn format_bad_see_penalty(params: &Vec<f32>) -> String {
        "const BAD_SEE_PENALTY: f32 = ".to_owned()
            + Self::format_single(params, BadSeePenalty.ft_offset()).as_str()
    }

    fn format_check_bonus(params: &Vec<f32>) -> String {
        "const CHECK_BONUS: f32 = ".to_owned()
            + Self::format_single(params, CheckBonus.ft_offset()).as_str()
    }

    pub fn format_all_features(params: &Vec<f32>) -> String {
        let mut result = String::new();
        for feature in Self::iter() {
            result += "#[rustfmt::skip]\n";
            result += Self::format_single_feature(feature, params).as_str();
            result += ";\n";
        }
        result
    }
}

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
            _ => unreachable!(),
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
            _ => unreachable!(),
        };
        SparseTrace::single(idx)
    }

    fn threat_evasion(threat: PieceType, moving: PieceType) -> Self::Value {
        SparseTrace::single(ThreatEvasion.ft_offset() + 5 * threat as u32 + moving as u32)
    }

    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value {
        let mg_weight = phase as f32 / 24.0;
        let eg_weight = 1.0 - mg_weight;

        let mg_offset =
            PsqtScore.ft_offset() + pt as u32 * 64 * 2 + sq.relative_sq(c).flip() as u32 * 2;
        let eg_offset = mg_offset + 1;

        SparseTrace {
            features: HashMap::from([(mg_offset, mg_weight), (eg_offset, eg_weight)]),
        }
    }

    fn passed_pawn_push(rank: u8, phase: i32) -> Self::Value {
        let mg_weight = phase as f32 / 24.0;
        let eg_weight = 1.0 - mg_weight;

        let mg_offset = PassedPawnPush.ft_offset() + 2 * (rank as u32);
        let eg_offset = mg_offset + 1;

        SparseTrace {
            features: HashMap::from([(mg_offset, mg_weight), (eg_offset, eg_weight)]),
        }
    }

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        assert!(moving != PieceType::King && threatened != PieceType::King);
        SparseTrace::single(
            Threat.ft_offset() + 5 * (moving as u32 - PieceType::Pawn as u32) + threatened as u32,
        )
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

pub fn compute_coeffs(board: &Board, mv: Move, data: &policy::PolicyData) -> Vec<(u32, f32)> {
    let trace = policy::get_policy_impl::<PolicyTrace>(board, mv, data);
    let mut result = Vec::new();

    for elem in trace.features {
        result.push(elem);
    }

    result
}

pub fn zero_params() -> Vec<f32> {
    let mut result = Vec::new();
    for _ in 0..PolicyFeature::total_fts() {
        result.push(0.0);
    }
    result
}
