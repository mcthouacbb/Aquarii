use std::{
    fmt::Debug,
    ops::{self, Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, Board},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait EvalScoreType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<i32, Output = Self>
    + Div<i32, Output = Self>
{
}

impl EvalScoreType for i32 {}

pub trait EvalScorePairType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<i32, Output = Self>
{
    type ScoreType: EvalScoreType;

    fn mg(&self) -> Self::ScoreType;
    fn eg(&self) -> Self::ScoreType;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ScorePair(i32);

impl ScorePair {
    const fn new(mg: i32, eg: i32) -> Self {
        Self((((eg as u32) << 16).wrapping_add(mg as u32)) as i32)
    }

    const fn mg(&self) -> i32 {
        self.0 as i16 as i32
    }

    const fn eg(&self) -> i32 {
        ((self.0.wrapping_add(0x8000)) as u32 >> 16) as i16 as i32
    }
}

impl ops::Add for ScorePair {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl ops::Sub for ScorePair {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl ops::Mul<i32> for ScorePair {
    type Output = Self;
    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl ops::Mul<ScorePair> for i32 {
    type Output = ScorePair;
    fn mul(self, rhs: ScorePair) -> Self::Output {
        ScorePair(self * rhs.0)
    }
}

impl ops::Neg for ScorePair {
    type Output = ScorePair;
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl ops::AddAssign for ScorePair {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl ops::SubAssign for ScorePair {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl ops::MulAssign<i32> for ScorePair {
    fn mul_assign(&mut self, rhs: i32) {
        self.0 *= rhs;
    }
}

impl EvalScorePairType for ScorePair {
    type ScoreType = i32;

    fn mg(&self) -> Self::ScoreType {
        self.mg()
    }

    fn eg(&self) -> Self::ScoreType {
        self.eg()
    }
}

#[allow(non_snake_case)]
const fn S(mg: i32, eg: i32) -> ScorePair {
    ScorePair::new(mg, eg)
}

pub trait EvalValues {
    type ScoreType: EvalScoreType;
    type ScorePairType: EvalScorePairType<ScoreType = Self::ScoreType>;

    fn material(pt: PieceType) -> Self::ScorePairType;
    fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType;
    fn mobility(pt: PieceType, mob: u32) -> Self::ScorePairType;
    fn passed_pawn(rank: u8) -> Self::ScorePairType;
    fn pawn_phalanx(rank: u8) -> Self::ScorePairType;
    fn defended_pawn(rank: u8) -> Self::ScorePairType;
    fn safe_knight_check() -> Self::ScorePairType;
    fn safe_bishop_check() -> Self::ScorePairType;
    fn safe_rook_check() -> Self::ScorePairType;
    fn safe_queen_check() -> Self::ScorePairType;
    fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType;
    fn king_attacks(attacks: u32) -> Self::ScorePairType;
    fn threat_by_pawn(pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(69,117), S(303,338), S(342,384), S(420,680), S(861,1213), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(54,76), S(45,64), S(12,69), S(55,14), S(-19,50), S(-0,25), S(-102,96), S(-96,115),
        S(16,36), S(11,40), S(27,-6), S(42,-46), S(61,-56), S(72,-25), S(74,12), S(38,23),
        S(-19,20), S(-15,3), S(-13,-16), S(-8,-36), S(9,-32), S(13,-27), S(11,-8), S(2,-7),
        S(-26,2), S(-22,-2), S(-17,-20), S(-9,-30), S(-5,-26), S(-3,-23), S(-3,-14), S(-9,-20),
        S(-37,-8), S(-28,-13), S(-27,-21), S(-23,-16), S(-11,-20), S(-17,-18), S(4,-24), S(-16,-27),
        S(-23,0), S(-16,-4), S(-10,-14), S(-13,-14), S(-0,-11), S(19,-16), S(27,-20), S(-5,-23),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-94,-38), S(-117,-22), S(-26,-4), S(4,-16), S(-10,-1), S(-15,-19), S(-73,3), S(-64,-67),
        S(-28,12), S(6,17), S(29,-14), S(32,-14), S(46,-28), S(37,-1), S(35,-4), S(27,5),
        S(-14,21), S(27,-3), S(38,-1), S(53,-6), S(38,1), S(71,-22), S(26,-0), S(-3,13),
        S(10,15), S(11,6), S(24,17), S(49,14), S(31,12), S(36,13), S(20,7), S(30,8),
        S(-9,24), S(10,3), S(11,17), S(23,12), S(14,23), S(14,8), S(13,7), S(3,14),
        S(-16,0), S(-11,-4), S(-6,-4), S(7,1), S(12,10), S(-3,-15), S(5,-9), S(-10,4),
        S(-37,14), S(-22,16), S(-13,-13), S(-3,-5), S(-5,-5), S(2,-13), S(-4,-17), S(-7,4),
        S(-75,13), S(-21,-7), S(-25,0), S(-20,4), S(-11,-4), S(-11,-9), S(-18,13), S(-25,14),
    ],
    [
        S(35,8), S(-81,23), S(21,-5), S(-69,5), S(-46,-11), S(-82,-1), S(-44,15), S(-30,8),
        S(-10,4), S(-1,-3), S(-8,-19), S(-11,0), S(-7,-18), S(-13,-12), S(-28,12), S(-11,1),
        S(-1,11), S(17,-1), S(14,-11), S(18,-15), S(32,-16), S(35,-7), S(39,-3), S(21,4),
        S(-18,6), S(-5,1), S(4,7), S(25,8), S(6,6), S(14,1), S(-13,9), S(-14,17),
        S(-11,6), S(-15,9), S(-1,-1), S(12,-3), S(9,-1), S(-6,-5), S(-8,10), S(-1,-12),
        S(4,5), S(8,-1), S(3,-3), S(6,-6), S(8,2), S(10,-6), S(5,0), S(9,1),
        S(17,-2), S(6,-18), S(14,-11), S(-3,-0), S(8,-8), S(16,-10), S(21,-15), S(13,12),
        S(5,5), S(12,24), S(3,5), S(5,-2), S(12,-2), S(-0,8), S(26,-8), S(25,2),
    ],
    [
        S(-10,22), S(31,1), S(1,29), S(24,11), S(22,9), S(4,24), S(89,-7), S(76,-9),
        S(-2,12), S(-3,18), S(6,22), S(39,4), S(14,3), S(25,11), S(28,-3), S(19,12),
        S(-9,10), S(20,1), S(1,8), S(7,12), S(28,-8), S(1,2), S(55,0), S(9,9),
        S(-20,12), S(18,-2), S(-6,13), S(1,10), S(10,-14), S(3,-6), S(6,-2), S(5,-8),
        S(-29,1), S(-24,3), S(-10,10), S(-6,7), S(-1,1), S(-28,-1), S(-16,-2), S(-29,9),
        S(-29,2), S(-27,4), S(-21,2), S(-11,-4), S(-13,-1), S(-15,-18), S(-3,-23), S(-12,-24),
        S(-41,-1), S(-18,-10), S(-13,-6), S(-14,-7), S(-4,-16), S(-4,-19), S(-6,-14), S(-25,-14),
        S(-26,-5), S(-17,-3), S(-7,-2), S(-0,-6), S(5,-16), S(-13,-10), S(-7,-20), S(-28,-18),
    ],
    [
        S(-7,13), S(-14,-29), S(-32,19), S(14,14), S(21,-1), S(-1,28), S(132,-80), S(3,12),
        S(2,7), S(-14,-4), S(-14,45), S(-19,45), S(-34,79), S(10,12), S(-15,11), S(33,13),
        S(5,-7), S(-10,10), S(-10,19), S(-30,30), S(1,26), S(11,19), S(23,-10), S(-8,39),
        S(-16,7), S(2,0), S(-15,14), S(-10,32), S(0,14), S(-14,36), S(-7,14), S(-4,16),
        S(-0,-21), S(-12,12), S(-11,23), S(-19,53), S(-16,40), S(-19,29), S(2,-2), S(-3,24),
        S(-4,-10), S(-3,-2), S(-8,17), S(-10,16), S(-13,28), S(4,-12), S(2,-6), S(8,-4),
        S(10,-36), S(-5,-25), S(4,-35), S(9,-27), S(3,-9), S(11,-35), S(6,-60), S(35,-104),
        S(-1,-21), S(5,-40), S(7,-40), S(9,-20), S(11,-38), S(-7,-32), S(-2,-58), S(25,-52),
    ],
    [
        S(19,-99), S(116,-75), S(114,-51), S(-32,-14), S(-29,1), S(-37,-8), S(-12,18), S(124,-148),
        S(-133,4), S(-102,29), S(-50,27), S(126,1), S(99,12), S(-8,43), S(83,22), S(15,-11),
        S(-99,13), S(30,16), S(60,24), S(8,50), S(108,48), S(142,44), S(50,53), S(3,9),
        S(34,-18), S(2,26), S(-40,41), S(-58,50), S(-45,59), S(-58,60), S(6,37), S(-96,10),
        S(-101,-2), S(-27,11), S(-40,32), S(-64,46), S(-72,54), S(-24,34), S(-49,23), S(-121,9),
        S(-1,-30), S(1,-7), S(-25,16), S(-30,25), S(-6,25), S(-18,16), S(3,-9), S(-50,-16),
        S(47,-38), S(28,-19), S(20,-4), S(-12,7), S(-15,12), S(4,-2), S(43,-24), S(33,-48),
        S(39,-93), S(58,-55), S(31,-34), S(-45,-9), S(4,-30), S(-16,-24), S(35,-51), S(30,-89),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-43,-305), S(-33,-21), S(-14,2), S(-5,31), S(5,40), S(8,56), S(17,64), S(28,68), S(37,65), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-19,-160), S(-50,-68), S(-26,-49), S(-21,-20), S(-9,-3), S(-3,8), S(2,24), S(7,29), S(11,37), S(18,38), S(12,48), S(39,38), S(23,48), S(19,28), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-28,-262), S(-51,4), S(-24,-27), S(-17,-8), S(-12,2), S(-7,7), S(-4,16), S(1,20), S(5,24), S(13,28), S(16,31), S(23,36), S(30,39), S(32,44), S(23,46), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-284,-350), S(-284,-350), S(-286,-351), S(-59,-65), S(-72,-29), S(-20,-30), S(-22,38), S(-18,40), S(-17,67), S(-11,71), S(-7,73), S(-3,83), S(3,82), S(10,83), S(8,94), S(9,102), S(17,95), S(13,100), S(19,102), S(22,100), S(19,100), S(49,68), S(73,61), S(107,13), S(92,33), S(363,-156), S(196,-46), S(85,-29)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-7,9), S(-7,15), S(-6,39), S(10,68), S(-8,135), S(35,149), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(7,8), S(16,16), S(25,29), S(50,78), S(135,206), S(-456,641), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(23,22), S(15,17), S(14,23), S(19,45), S(152,80), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(21,9);
#[rustfmt::skip]
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(30,23);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(72,0);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(41,15);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(7,-2), S(1,-4), S(6,-14), S(-7,14)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-38,9), S(-35,7), S(-29,2), S(-23,11), S(-7,9), S(19,-1), S(55,-7), S(102,-32), S(158,-61), S(207,-81), S(208,-54), S(342,-105), S(237,43), S(154,6)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(30,5), S(71,21), S(65,59), S(92,11), S(88,-25), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(5,23), S(29,17), S(22,40), S(56,10), S(73,-76), S(0,0)],
    [S(-9,8), S(18,15), S(28,34), S(54,29), S(39,14), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(5,39), S(52,29), S(12,5), S(83,18), S(84,19), S(0,0)],
    [S(0,18), S(19,46), S(-12,-5), S(53,54), S(72,125), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(-3,46), S(36,43), S(38,47), S(-62,13), S(89,-17), S(0,0)],
    [S(-3,10), S(11,16), S(25,13), S(-78,-9), S(75,50), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(7,1), S(29,14), S(10,56), S(49,-27), S(17,-27), S(0,0)],
    [S(-3,8), S(-1,6), S(-6,47), S(2,1), S(-26,19), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 30;

pub struct EvalParams {}

impl EvalValues for EvalParams {
    type ScoreType = i32;
    type ScorePairType = ScorePair;

    fn material(pt: PieceType) -> Self::ScorePairType {
        MATERIAL[pt as usize]
    }

    fn psqt(c: Color, pt: PieceType, sq: Square) -> Self::ScorePairType {
        PSQT[pt as usize][sq.relative_sq(c).flip().value() as usize]
    }

    fn mobility(pt: PieceType, mob: u32) -> Self::ScorePairType {
        MOBILITY[pt as usize - PieceType::Knight as usize][mob as usize]
    }

    fn passed_pawn(rank: u8) -> Self::ScorePairType {
        PASSED_PAWN[rank as usize]
    }

    fn pawn_phalanx(rank: u8) -> Self::ScorePairType {
        PAWN_PHALANX[rank as usize]
    }

    fn defended_pawn(rank: u8) -> Self::ScorePairType {
        DEFENDED_PAWN[rank as usize]
    }

    fn safe_knight_check() -> Self::ScorePairType {
        SAFE_KNIGHT_CHECK
    }

    fn safe_bishop_check() -> Self::ScorePairType {
        SAFE_BISHOP_CHECK
    }

    fn safe_rook_check() -> Self::ScorePairType {
        SAFE_ROOK_CHECK
    }

    fn safe_queen_check() -> Self::ScorePairType {
        SAFE_QUEEN_CHECK
    }

    fn king_attacker_weight(pt: PieceType) -> Self::ScorePairType {
        KING_ATTACKER_WEIGHT[pt as usize - PieceType::Knight as usize]
    }

    fn king_attacks(attacks: u32) -> Self::ScorePairType {
        KING_ATTACKS[attacks as usize]
    }

    fn threat_by_pawn(pt: PieceType) -> Self::ScorePairType {
        THREAT_BY_PAWN[pt as usize]
    }

    fn threat_by_knight(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_KNIGHT[defended as usize][pt as usize]
    }

    fn threat_by_bishop(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_BISHOP[defended as usize][pt as usize]
    }

    fn threat_by_rook(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_ROOK[defended as usize][pt as usize]
    }

    fn threat_by_queen(pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_QUEEN[defended as usize][pt as usize]
    }

    fn tempo() -> Self::ScoreType {
        TEMPO
    }
}

struct EvalData<ScorePairType: EvalScorePairType> {
    attacked: [Bitboard; 2],
    attacked_by: [[Bitboard; 6]; 2],
    attacked_by_2: [Bitboard; 2],
    king_ring: [Bitboard; 2],
    king_attack_weight: [ScorePairType; 2],
    king_attacks: [i32; 2],
}

impl<ScorePairType: EvalScorePairType> Default for EvalData<ScorePairType> {
    fn default() -> Self {
        Self {
            attacked: [Bitboard::NONE; 2],
            attacked_by: [[Bitboard::NONE; 6]; 2],
            attacked_by_2: [Bitboard::NONE; 2],
            king_ring: [Bitboard::NONE; 2],
            king_attack_weight: [ScorePairType::default(), ScorePairType::default()],
            king_attacks: [0; 2],
        }
    }
}

fn evaluate_piece<Params: EvalValues>(
    board: &Board,
    pt: PieceType,
    color: Color,
    eval_data: &mut EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let opp_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));
    let mobility_area = !attacks::pawn_attacks_bb(!color, opp_pawns);

    let mut pieces = board.colored_pieces(Piece::new(color, pt));

    while pieces.any() {
        let sq = pieces.poplsb();

        let attacks = attacks::piece_attacks(pt, sq, board.occ());
        let mobility = (attacks & mobility_area).popcount();
        eval += Params::mobility(pt, mobility);

        eval_data.attacked_by_2[color as usize] |= attacks & eval_data.attacked[color as usize];
        eval_data.attacked[color as usize] |= attacks;
        eval_data.attacked_by[color as usize][pt as usize] |= attacks;

        let king_ring_attacks = eval_data.king_ring[!color as usize] & attacks;
        if king_ring_attacks.any() {
            eval_data.king_attack_weight[color as usize] += Params::king_attacker_weight(pt);
            eval_data.king_attacks[color as usize] += king_ring_attacks.popcount() as i32;
        }
    }
    eval
}

fn evaluate_kings<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let their_king = board.king_sq(!color);

    let rook_check_squares = attacks::rook_attacks(their_king, board.occ());
    let bishop_check_squares = attacks::bishop_attacks(their_king, board.occ());

    let knight_checks = eval_data.attacked_by[color as usize][PieceType::Knight as usize]
        & attacks::knight_attacks(their_king);
    let bishop_checks =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & bishop_check_squares;
    let rook_checks =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & rook_check_squares;
    let queen_checks = eval_data.attacked_by[color as usize][PieceType::Queen as usize]
        & (bishop_check_squares | rook_check_squares);

    let weak = !eval_data.attacked[!color as usize]
        | (!eval_data.attacked_by_2[!color as usize]
            & eval_data.attacked_by[!color as usize][PieceType::King as usize]);
    let safe = !board.colors(color)
        & (!eval_data.attacked[!color as usize] | (weak & eval_data.attacked_by_2[color as usize]));

    eval += Params::safe_knight_check() * (knight_checks & safe).popcount() as i32;
    eval += Params::safe_bishop_check() * (bishop_checks & safe).popcount() as i32;
    eval += Params::safe_rook_check() * (rook_checks & safe).popcount() as i32;
    eval += Params::safe_queen_check() * (queen_checks & safe).popcount() as i32;

    eval += eval_data.king_attack_weight[color as usize].clone();
    eval += Params::king_attacks(eval_data.king_attacks[color as usize].min(13) as u32);

    return eval;
}

fn evaluate_threats<Params: EvalValues>(
    board: &Board,
    color: Color,
    eval_data: &EvalData<Params::ScorePairType>,
) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();

    let defended_bb = eval_data.attacked_by_2[!color as usize]
        | eval_data.attacked_by[!color as usize][PieceType::Pawn as usize]
        | (eval_data.attacked[!color as usize] & !eval_data.attacked_by_2[color as usize]);

    let mut pawn_threats =
        eval_data.attacked_by[color as usize][PieceType::Pawn as usize] & board.colors(!color);
    while pawn_threats.any() {
        let threatened = board.piece_at(pawn_threats.poplsb()).unwrap().piece_type();
        eval += Params::threat_by_pawn(threatened);
    }

    let mut knight_threats =
        eval_data.attacked_by[color as usize][PieceType::Knight as usize] & board.colors(!color);
    while knight_threats.any() {
        let threat = knight_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_knight(threatened, defended);
    }

    let mut bishop_threats =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & board.colors(!color);
    while bishop_threats.any() {
        let threat = bishop_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_bishop(threatened, defended);
    }

    let mut rook_threats =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & board.colors(!color);
    while rook_threats.any() {
        let threat = rook_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_rook(threatened, defended);
    }

    let mut queen_threats =
        eval_data.attacked_by[color as usize][PieceType::Queen as usize] & board.colors(!color);
    while queen_threats.any() {
        let threat = queen_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_queen(threatened, defended);
    }

    eval
}

fn evaluate_pawns<Params: EvalValues>(board: &Board, color: Color) -> Params::ScorePairType {
    let mut eval = Params::ScorePairType::default();
    let our_pawns = board.colored_pieces(Piece::new(color, PieceType::Pawn));
    let their_pawns = board.colored_pieces(Piece::new(!color, PieceType::Pawn));

    let mut tmp = our_pawns;
    while tmp.any() {
        let sq = tmp.poplsb();
        let relative_rank = sq.relative_sq(color).rank();
        let stoppers = their_pawns & attacks::passed_pawn_span(color, sq);
        if stoppers.empty() {
            eval += Params::passed_pawn(relative_rank);
        }
    }

    let mut phalanxes = our_pawns & our_pawns.west();
    while phalanxes.any() {
        eval += Params::pawn_phalanx(phalanxes.poplsb().relative_sq(color).rank());
    }
    let mut defended = our_pawns & attacks::pawn_attacks_bb(color, our_pawns);
    while defended.any() {
        eval += Params::defended_pawn(defended.poplsb().relative_sq(color).rank());
    }
    eval
}

pub fn eval_impl<Params: EvalValues>(board: &Board) -> Params::ScoreType {
    let stm = board.stm();
    let mut eval = Params::ScorePairType::default();
    for pt in [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ] {
        let mut stm_bb = board.colored_pieces(Piece::new(stm, pt));
        let mut nstm_bb = board.colored_pieces(Piece::new(!stm, pt));

        while stm_bb.any() {
            eval += Params::material(pt) + Params::psqt(stm, pt, stm_bb.poplsb());
        }

        while nstm_bb.any() {
            eval -= Params::material(pt) + Params::psqt(!stm, pt, nstm_bb.poplsb());
        }
    }

    let mut eval_data = EvalData::default();
    // TODO: handle pawn attacks
    let wking_atks = attacks::king_attacks(board.king_sq(Color::White));
    let bking_atks = attacks::king_attacks(board.king_sq(Color::Black));
    eval_data.attacked[Color::White as usize] = wking_atks;
    eval_data.attacked[Color::Black as usize] = bking_atks;
    eval_data.attacked_by[Color::White as usize][PieceType::King as usize] = wking_atks;
    eval_data.attacked_by[Color::Black as usize][PieceType::King as usize] = bking_atks;
    eval_data.attacked_by[Color::White as usize][PieceType::Pawn as usize] =
        attacks::pawn_attacks_bb(Color::White, board.colored_pieces(Piece::WhitePawn));
    eval_data.attacked_by[Color::Black as usize][PieceType::Pawn as usize] =
        attacks::pawn_attacks_bb(Color::Black, board.colored_pieces(Piece::BlackPawn));

    eval_data.king_ring[Color::White as usize] =
        (wking_atks | wking_atks.north()) & !Bitboard::from_square(board.king_sq(Color::White));
    eval_data.king_ring[Color::Black as usize] =
        (bking_atks | bking_atks.south()) & !Bitboard::from_square(board.king_sq(Color::Black));

    eval += evaluate_piece::<Params>(board, PieceType::Knight, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Knight, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Bishop, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Bishop, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Rook, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Rook, !stm, &mut eval_data);
    eval += evaluate_piece::<Params>(board, PieceType::Queen, stm, &mut eval_data)
        - evaluate_piece::<Params>(board, PieceType::Queen, !stm, &mut eval_data);

    eval += evaluate_kings::<Params>(board, stm, &eval_data)
        - evaluate_kings::<Params>(board, !stm, &eval_data);
    eval += evaluate_threats::<Params>(board, stm, &eval_data)
        - evaluate_threats::<Params>(board, !stm, &eval_data);

    eval += evaluate_pawns::<Params>(board, stm) - evaluate_pawns::<Params>(board, !stm);

    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount()) as i32;

    (eval.mg() * phase.min(24) + eval.eg() * (24 - phase.min(24))) / 24 + Params::tempo()
}

pub fn eval(board: &Board) -> i32 {
    eval_impl::<EvalParams>(board)
}
