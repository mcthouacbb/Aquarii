use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use crate::{chess::Board, eval::{self, EvalScorePairType, EvalValues}, tune::SparseTrace, types::{Color, PieceType, Square}};

#[derive(Debug, Default, Clone, PartialEq)]
struct SparseTracePair {
    mg: SparseTrace,
    eg: SparseTrace,
}

impl SparseTracePair {
	fn pair(offset: u32) -> Self {
		Self {
			mg: SparseTrace::single(offset),
			eg: SparseTrace::single(offset + 1),
		}
	}
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
            eg: -self.eg,
        }
    }
}

impl Mul<f32> for SparseTracePair {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            mg: self.mg * rhs,
            eg: self.eg * rhs,
        }
    }
}

impl Div<f32> for SparseTracePair {
    type Output = Self;
    fn div(self, rhs: f32) -> Self::Output {
        Self {
            mg: self.mg / rhs,
            eg: self.eg / rhs,
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
    ThreatByQueen,
    Tempo,
}

use EvalFeature::*;

impl EvalFeature {
    pub const TOTAL_FEATURES: u32 = 18;

    fn from_raw(raw: u32) -> Self {
        unsafe { std::mem::transmute(raw) }
    }

    fn ft_cnt(self) -> u32 {
        match self {
            Material => 2 * 6,
            Psqt => 2 * 6 * 64,
            Mobility => 2 * 4 * 28,
            PassedPawn => 2 * 8,
            PawnPhalanx => 2 * 8,
            DefendedPawn => 2 * 8,
            SafeKnightCheck => 2,
            SafeBishopCheck => 2,
            SafeRookCheck => 2,
            SafeQueenCheck => 2,
            KingAttackerWeight => 2 * 4,
            KingAttacks => 2 * 14,
            ThreatByPawn => 2 * 6,
            ThreatByKnight => 2 * 2 * 6,
            ThreatByBishop => 2 * 2 * 6,
            ThreatByRook => 2 * 2 * 6,
            ThreatByQueen => 2 * 2 * 6,
            Tempo => 1,
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

    fn format_pair(params: &Vec<f32>, offset: u32) -> String {
        format!(
            "S({},{})",
            Self::format_single(params, offset),
            Self::format_single(params, offset + 1)
        )
    }

    #[allow(non_snake_case)]
    fn format_array_1D_pair(params: &Vec<f32>, offset: u32, size: u32) -> String {
        let mut result = "[".to_owned();
        for i in 0..size {
            if i != size - 1 {
                result += format!("{}, ", Self::format_pair(params, offset + i * 2)).as_str();
            } else {
                result += format!("{}", Self::format_pair(params, offset + i * 2)).as_str();
            }
        }
        result + "]"
    }

    #[allow(non_snake_case)]
    fn format_array_2D_pair(params: &Vec<f32>, offset: u32, size1: u32, size2: u32) -> String {
        let mut result = "[\n".to_owned();
        for i in 0..size1 {
            result += "    ";
            result += Self::format_array_1D_pair(params, offset + size2 * i * 2, size2).as_str();
            result += ",\n";
        }
        result + "]"
    }

    fn format_single_feature(feature: Self, params: &Vec<f32>) -> String {
        match feature {
            Material => Self::format_material(params),
            Psqt => Self::format_psqt(params),
            Mobility => Self::format_mobility(params),
            PassedPawn => Self::format_passed_pawn(params),
            PawnPhalanx => Self::format_pawn_phalanx(params),
            DefendedPawn => Self::format_defended_pawn(params),
            SafeKnightCheck => Self::format_safe_knight_check(params),
            SafeBishopCheck => Self::format_safe_bishop_check(params),
            SafeRookCheck => Self::format_safe_rook_check(params),
            SafeQueenCheck => Self::format_safe_queen_check(params),
            KingAttackerWeight => Self::format_king_attacker_weight(params),
            KingAttacks => Self::format_king_attacks(params),
            ThreatByPawn => Self::format_threat_by_pawn(params),
            ThreatByKnight => Self::format_threat_by_knight(params),
            ThreatByBishop => Self::format_threat_by_bishop(params),
            ThreatByRook => Self::format_threat_by_rook(params),
            ThreatByQueen => Self::format_threat_by_queen(params),
            Tempo => Self::format_tempo(params),
        }
    }

    fn format_material(params: &Vec<f32>) -> String {
        "const MATERIAL: [ScorePair; 6] = ".to_owned()
            + Self::format_array_1D_pair(params, Material.ft_offset(), Material.ft_cnt() / 2).as_str()
    }

    fn format_psqt(params: &Vec<f32>) -> String {
        let mut result = "const PSQT: [[ScorePair; 64]; 6] = [\n".to_owned();
        for pt in 0..6 {
            result += "    [\n";
            for y in 0..8 {
                result += "        ";
                for x in 0..8 {
                    let offset = Psqt.ft_offset() + pt * 64 * 2 + y * 8 * 2 + x * 2;
                    result += format!("{},", Self::format_pair(params, offset)).as_str();
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

    fn format_mobility(params: &Vec<f32>) -> String {
        "const MOBILITY: [[ScorePair; 28]; 4] = ".to_owned()
            + Self::format_array_2D_pair(params, Mobility.ft_offset(), 4, 28).as_str()
    }

    fn format_passed_pawn(params: &Vec<f32>) -> String {
        "const PASSED_PAWN: [ScorePair; 8] = ".to_owned()
            + Self::format_array_1D_pair(params, PassedPawn.ft_offset(), PassedPawn.ft_cnt() / 2)
                .as_str()
    }

    fn format_pawn_phalanx(params: &Vec<f32>) -> String {
        "const PAWN_PHALANX: [ScorePair; 8] = ".to_owned()
            + Self::format_array_1D_pair(params, PawnPhalanx.ft_offset(), PawnPhalanx.ft_cnt() / 2)
                .as_str()
    }

    fn format_defended_pawn(params: &Vec<f32>) -> String {
        "const DEFENDED_PAWN: [ScorePair; 8] = ".to_owned()
            + Self::format_array_1D_pair(params, DefendedPawn.ft_offset(), DefendedPawn.ft_cnt() / 2)
                .as_str()
    }

    fn format_safe_knight_check(params: &Vec<f32>) -> String {
        "const SAFE_KNIGHT_CHECK: ScorePair = ".to_owned()
            + Self::format_pair(params, SafeKnightCheck.ft_offset()).as_str()
    }

    fn format_safe_bishop_check(params: &Vec<f32>) -> String {
        "const SAFE_BISHOP_CHECK: ScorePair = ".to_owned()
            + Self::format_pair(params, SafeBishopCheck.ft_offset()).as_str()
    }

    fn format_safe_rook_check(params: &Vec<f32>) -> String {
        "const SAFE_ROOK_CHECK: ScorePair = ".to_owned()
            + Self::format_pair(params, SafeRookCheck.ft_offset()).as_str()
    }

    fn format_safe_queen_check(params: &Vec<f32>) -> String {
        "const SAFE_QUEEN_CHECK: ScorePair = ".to_owned()
            + Self::format_pair(params, SafeQueenCheck.ft_offset()).as_str()
    }

    fn format_king_attacker_weight(params: &Vec<f32>) -> String {
        "const KING_ATTACKER_WEIGHT: [ScorePair; 4] = ".to_owned()
            + Self::format_array_1D_pair(
                params,
                KingAttackerWeight.ft_offset(),
                KingAttackerWeight.ft_cnt() / 2,
            )
            .as_str()
    }

    fn format_king_attacks(params: &Vec<f32>) -> String {
        "const KING_ATTACKS: [ScorePair; 14] = ".to_owned()
            + Self::format_array_1D_pair(params, KingAttacks.ft_offset(), KingAttacks.ft_cnt() / 2)
                .as_str()
    }

    fn format_threat_by_pawn(params: &Vec<f32>) -> String {
        "const THREAT_BY_PAWN: [ScorePair; 6] = ".to_owned()
            + Self::format_array_1D_pair(params, ThreatByPawn.ft_offset(), ThreatByPawn.ft_cnt() / 2)
                .as_str()
    }

    fn format_threat_by_knight(params: &Vec<f32>) -> String {
        "const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = ".to_owned()
            + Self::format_array_2D_pair(params, ThreatByKnight.ft_offset(), 2, 6).as_str()
    }

    fn format_threat_by_bishop(params: &Vec<f32>) -> String {
        "const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = ".to_owned()
            + Self::format_array_2D_pair(params, ThreatByBishop.ft_offset(), 2, 6).as_str()
    }

    fn format_threat_by_rook(params: &Vec<f32>) -> String {
        "const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = ".to_owned()
            + Self::format_array_2D_pair(params, ThreatByRook.ft_offset(), 2, 6).as_str()
    }

    fn format_threat_by_queen(params: &Vec<f32>) -> String {
        "const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = ".to_owned()
            + Self::format_array_2D_pair(params, ThreatByQueen.ft_offset(), 2, 6).as_str()
    }

    fn format_tempo(params: &Vec<f32>) -> String {
        "const TEMPO: i32 = ".to_owned()
            + format!("{}", params[Tempo.ft_offset() as usize]).as_str()
    }

    pub fn format_all_features(params: &Vec<f32>) -> String {
        let mut result = String::new();
        for feature in Self::iter() {
            result += Self::format_single_feature(feature, params).as_str();
            result += ";\n";
        }
        result
    }
}

struct EvalTrace {

}

impl EvalValues for EvalTrace {
	type ScoreType = SparseTrace;
	type ScorePairType = SparseTracePair;
	fn material(pt: PieceType) -> Self::ScorePairType {
		SparseTracePair::pair(Material.ft_offset() + 2 * pt as u32)
	}

	fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType {
		SparseTracePair::pair(Psqt.ft_offset() + 2 * (64 * pt as u32 + sq.relative_sq(c).flip() as u32))
	}

	fn mobility(pt: PieceType, mob: u32) -> Self::ScorePairType {
		SparseTracePair::pair(Mobility.ft_offset() + 2 * ((pt as u32 - PieceType::Knight as u32) * 28 + mob))
	}

	fn passed_pawn(rank: u8) -> Self::ScorePairType {
		SparseTracePair::pair(PassedPawn.ft_offset() + 2 * rank as u32)
	}

	fn pawn_phalanx(rank: u8) -> Self::ScorePairType {
		SparseTracePair::pair(PawnPhalanx.ft_offset() + 2 * rank as u32)
	}

	fn defended_pawn(rank: u8) -> Self::ScorePairType {
		SparseTracePair::pair(DefendedPawn.ft_offset() + 2 * rank as u32)
	}

	fn safe_knight_check() -> Self::ScorePairType {
		SparseTracePair::pair(SafeKnightCheck.ft_offset())
	}

	fn safe_bishop_check() -> Self::ScorePairType {
		SparseTracePair::pair(SafeBishopCheck.ft_offset())
	}

	fn safe_rook_check() -> Self::ScorePairType {
		SparseTracePair::pair(SafeRookCheck.ft_offset())
	}

	fn safe_queen_check() -> Self::ScorePairType {
		SparseTracePair::pair(SafeQueenCheck.ft_offset())
	}

	fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType {
		SparseTracePair::pair(KingAttackerWeight.ft_offset() + 2 * (pt as u32 - PieceType::Knight as u32))
	}

	fn king_attacks(attacks: u32) -> Self::ScorePairType {
		SparseTracePair::pair(KingAttacks.ft_offset() + 2 * attacks)
	}

	fn threat_by_pawn(pt: PieceType) -> Self::ScorePairType {
		SparseTracePair::pair(ThreatByPawn.ft_offset() + 2 * pt as u32)
	}

	fn threat_by_knight(pt: PieceType, defended: bool) -> Self::ScorePairType {
		SparseTracePair::pair(ThreatByKnight.ft_offset() + 2 * (6 * defended as u32 + pt as u32))
	}

	fn threat_by_bishop(pt: PieceType, defended: bool) -> Self::ScorePairType {
		SparseTracePair::pair(ThreatByBishop.ft_offset() + 2 * (6 * defended as u32 + pt as u32))
	}

	fn threat_by_rook(pt: PieceType, defended: bool) -> Self::ScorePairType {
		SparseTracePair::pair(ThreatByRook.ft_offset() + 2 * (6 * defended as u32 + pt as u32))
	}

	fn threat_by_queen(pt: PieceType, defended: bool) -> Self::ScorePairType {
		SparseTracePair::pair(ThreatByQueen.ft_offset() + 2 * (6 * defended as u32 + pt as u32))
	}

	fn tempo() -> Self::ScoreType {
		SparseTrace::single(Tempo.ft_offset())
	}
}

pub fn compute_coeffs(board: &Board) -> Vec<(u32, f32)> {
    let trace = eval::eval_impl::<EvalTrace>(board);
    let mut result = Vec::new();

    for elem in trace.features {
        result.push(elem);
    }

    result
}

pub fn zero_params() -> Vec<f32> {
    let mut result = Vec::new();
    for _ in 0..EvalFeature::total_fts() {
        result.push(0.0);
    }
    result
}
