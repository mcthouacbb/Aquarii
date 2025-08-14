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
    pub const fn new(mg: i32, eg: i32) -> Self {
        Self((((eg as u32) << 16).wrapping_add(mg as u32)) as i32)
    }

    pub const fn mg(&self) -> i32 {
        self.0 as i16 as i32
    }

    pub const fn eg(&self) -> i32 {
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
const MATERIAL: [ScorePair; 6] = [S(72,106), S(343,131), S(417,215), S(583,336), S(958,729), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(184,-9), S(86,-12), S(-15,78), S(-45,72), S(101,-21), S(18,71), S(-294,171), S(32,70),
        S(-44,12), S(44,46), S(-80,23), S(-49,-29), S(4,-29), S(-20,-8), S(37,-7), S(-8,-12),
        S(23,-7), S(13,-21), S(5,15), S(7,-38), S(2,-32), S(-42,-21), S(13,-18), S(-21,4),
        S(-5,-15), S(12,-29), S(-2,5), S(18,-39), S(9,-28), S(5,-14), S(12,-19), S(-13,-5),
        S(-12,-1), S(0,-12), S(10,7), S(-23,2), S(16,-31), S(-5,-27), S(28,-36), S(1,-4),
        S(-19,17), S(11,-7), S(11,25), S(-30,1), S(-2,-29), S(18,-24), S(13,-14), S(-4,-21),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(54,-275), S(-169,1), S(-273,107), S(-130,46), S(-83,68), S(-205,52), S(314,33), S(-298,21),
        S(-101,86), S(7,3), S(85,-27), S(41,-63), S(34,-64), S(52,-51), S(-2,-58), S(-152,53),
        S(7,21), S(-25,-33), S(-21,9), S(55,-10), S(10,7), S(99,29), S(14,11), S(-9,-40),
        S(-25,-29), S(18,9), S(15,1), S(45,-14), S(59,-20), S(7,-26), S(40,4), S(99,-64),
        S(8,-30), S(69,-0), S(39,-3), S(4,18), S(33,18), S(32,47), S(84,-2), S(37,-51),
        S(28,-43), S(6,-23), S(16,28), S(-10,27), S(50,-4), S(18,-32), S(33,-17), S(19,14),
        S(80,-46), S(60,-46), S(43,-59), S(13,38), S(31,-6), S(-19,-53), S(-5,-21), S(33,88),
        S(-157,-14), S(18,71), S(-71,-11), S(-45,114), S(4,2), S(97,-29), S(9,-77), S(-118,313),
    ],
    [
        S(17,-38), S(-37,65), S(-155,6), S(-153,-66), S(58,-32), S(-195,57), S(-49,92), S(-106,-15),
        S(-31,3), S(-8,-29), S(-56,2), S(218,-103), S(6,-21), S(-74,41), S(-31,11), S(-18,29),
        S(73,-81), S(75,-50), S(22,-7), S(-42,34), S(82,26), S(-45,67), S(29,12), S(0,5),
        S(85,-40), S(50,-15), S(-23,39), S(50,11), S(-41,40), S(-23,49), S(16,26), S(13,-5),
        S(79,-62), S(19,-18), S(17,-0), S(7,55), S(41,50), S(-11,77), S(15,15), S(78,-85),
        S(26,14), S(57,-19), S(6,-10), S(28,15), S(14,3), S(72,-24), S(28,58), S(22,13),
        S(81,47), S(6,22), S(-31,-22), S(25,-17), S(56,-55), S(-25,13), S(48,-2), S(36,-59),
        S(-172,52), S(44,-95), S(7,-12), S(-49,0), S(-22,-40), S(10,-6), S(-171,-49), S(-47,27),
    ],
    [
        S(73,-3), S(124,-46), S(348,-108), S(362,-117), S(226,-85), S(43,-21), S(206,-35), S(123,-3),
        S(3,13), S(28,-14), S(92,-53), S(227,-77), S(172,-92), S(10,23), S(44,0), S(-15,39),
        S(-14,23), S(-65,28), S(-100,73), S(32,-26), S(10,19), S(31,-4), S(-24,-15), S(-59,29),
        S(-43,27), S(-41,16), S(81,-28), S(-29,-2), S(-24,-9), S(-105,35), S(-32,5), S(9,-29),
        S(-84,25), S(-135,26), S(-63,11), S(-46,67), S(43,-55), S(-28,-5), S(-50,-14), S(-29,-1),
        S(-62,34), S(-60,7), S(-74,12), S(-78,31), S(-75,12), S(-47,17), S(-30,32), S(-91,19),
        S(-73,67), S(29,-67), S(19,-4), S(-67,51), S(-83,-1), S(-74,-21), S(-26,-27), S(-76,24),
        S(-80,44), S(-58,26), S(-43,30), S(-35,27), S(-44,21), S(-31,18), S(-80,20), S(-61,11),
    ],
    [
        S(154,-122), S(-23,29), S(-206,177), S(-40,35), S(186,-33), S(206,-126), S(212,-160), S(46,-142),
        S(-21,97), S(-24,39), S(-11,135), S(44,61), S(102,-103), S(-3,89), S(-23,-11), S(39,-108),
        S(28,-4), S(-42,46), S(-69,80), S(-4,74), S(3,122), S(-28,21), S(-113,125), S(-37,50),
        S(-84,65), S(7,127), S(-32,80), S(-40,192), S(-42,82), S(-53,136), S(-39,-4), S(-6,-17),
        S(-47,82), S(-104,47), S(-47,76), S(-36,13), S(-27,128), S(-61,120), S(-15,52), S(7,-91),
        S(32,-60), S(11,-23), S(-23,-86), S(-62,164), S(-58,41), S(-36,134), S(-48,112), S(36,38),
        S(-33,-151), S(2,-21), S(12,-50), S(-20,-20), S(-15,63), S(46,-130), S(14,-148), S(43,-116),
        S(-43,-128), S(21,-156), S(20,-158), S(-9,-58), S(26,-187), S(23,-116), S(79,-279), S(225,-126),
    ],
    [
        S(178,-130), S(513,-163), S(-264,-126), S(134,-34), S(-459,142), S(-476,147), S(-255,26), S(-303,-21),
        S(95,-47), S(333,3), S(-72,53), S(-285,42), S(-231,116), S(-202,78), S(-65,47), S(-408,53),
        S(112,5), S(-182,62), S(-117,84), S(-494,72), S(-152,63), S(-147,69), S(-123,73), S(-280,60),
        S(142,12), S(85,26), S(-131,42), S(282,-23), S(328,-32), S(184,-0), S(108,22), S(-108,43),
        S(6,-32), S(51,12), S(-33,38), S(26,18), S(175,-26), S(103,14), S(75,12), S(22,4),
        S(-7,-47), S(110,-22), S(115,-21), S(3,18), S(-11,16), S(54,4), S(88,-41), S(111,-22),
        S(229,-72), S(31,3), S(81,-22), S(41,-20), S(76,-18), S(56,-6), S(110,-40), S(162,-80),
        S(-27,-19), S(120,-61), S(71,-49), S(71,-95), S(70,-22), S(57,-50), S(117,-40), S(107,-96),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(123,-483), S(-35,6), S(-53,69), S(-31,57), S(-14,68), S(-12,76), S(-5,84), S(10,80), S(16,43), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-79,211), S(-51,78), S(-49,-35), S(-39,-22), S(-35,-1), S(-33,22), S(-32,23), S(-30,24), S(-26,24), S(7,-26), S(38,-8), S(100,-104), S(108,-52), S(121,-134), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(42,14), S(-63,-46), S(-56,-10), S(-55,3), S(-50,8), S(-33,6), S(-34,23), S(-22,18), S(-14,15), S(-14,17), S(14,-1), S(-2,22), S(3,21), S(2,28), S(282,-118), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-353,-201), S(-353,-201), S(-356,-202), S(-143,11), S(-94,227), S(-142,238), S(-143,12), S(-150,70), S(-145,121), S(-149,203), S(-147,191), S(-141,198), S(-148,220), S(-145,227), S(-136,217), S(-129,186), S(-115,156), S(-142,186), S(-94,106), S(-89,103), S(-60,111), S(-17,17), S(-11,36), S(435,-319), S(649,-437), S(980,-624), S(1131,-664), S(206,-187)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(11,12), S(-12,31), S(6,43), S(-27,84), S(47,94), S(66,110), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(9,26), S(13,36), S(25,34), S(27,68), S(178,145), S(155,296), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(18,11), S(10,18), S(15,17), S(105,19), S(579,-113), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(26,-8);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(17,16);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(109,-27);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(28,-0);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(18,10), S(8,13), S(17,20), S(3,68)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-34,32), S(-31,-13), S(-30,-8), S(-30,2), S(-16,-1), S(7,-20), S(27,-17), S(71,-47), S(132,-68), S(127,-79), S(219,-142), S(46,-81), S(298,-10), S(292,-371)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(-23,-55), S(105,50), S(115,130), S(113,148), S(-12,957), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(4,56), S(28,57), S(107,91), S(63,95), S(41,606), S(0,0)],
    [S(-4,18), S(-15,60), S(61,40), S(47,93), S(0,237), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(20,45), S(91,47), S(37,6), S(96,112), S(83,491), S(0,0)],
    [S(-6,16), S(26,11), S(-37,-6), S(57,141), S(37,352), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(5,86), S(79,65), S(135,43), S(-191,80), S(75,199), S(0,0)],
    [S(-15,26), S(8,20), S(46,-15), S(-217,27), S(72,374), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(24,17), S(69,59), S(130,-56), S(118,-4), S(5,10), S(0,0)],
    [S(-4,6), S(12,25), S(12,-21), S(-32,32), S(-24,-23), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 38;

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
