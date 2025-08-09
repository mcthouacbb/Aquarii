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
const MATERIAL: [ScorePair; 6] = [S(46,69), S(224,86), S(271,139), S(385,214), S(640,467), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(119,-5), S(54,-6), S(-9,51), S(-27,47), S(69,-14), S(14,46), S(-190,112), S(18,47),
        S(-31,8), S(31,29), S(-52,14), S(-33,-18), S(2,-19), S(-13,-5), S(24,-5), S(-4,-9),
        S(15,-4), S(9,-14), S(3,9), S(5,-25), S(1,-21), S(-27,-14), S(9,-11), S(-14,2),
        S(-4,-9), S(7,-19), S(-2,3), S(10,-26), S(6,-18), S(3,-10), S(7,-12), S(-9,-3),
        S(-8,-1), S(0,-8), S(6,4), S(-15,1), S(11,-21), S(-3,-18), S(18,-24), S(0,-2),
        S(-13,11), S(7,-5), S(7,16), S(-20,-0), S(-1,-19), S(11,-16), S(9,-10), S(-3,-13),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(32,-172), S(-106,-3), S(-182,68), S(-85,29), S(-64,44), S(-131,32), S(200,24), S(-187,10),
        S(-69,62), S(5,3), S(53,-16), S(30,-39), S(22,-41), S(35,-33), S(2,-39), S(-95,34),
        S(4,14), S(-17,-22), S(-16,6), S(35,-6), S(6,4), S(63,18), S(10,3), S(-2,-28),
        S(-13,-21), S(12,5), S(10,-0), S(30,-11), S(39,-14), S(5,-17), S(26,3), S(64,-42),
        S(6,-18), S(45,-1), S(25,-1), S(2,11), S(21,12), S(22,29), S(57,-3), S(26,-36),
        S(19,-28), S(5,-16), S(9,17), S(-7,18), S(32,-4), S(12,-21), S(20,-9), S(12,11),
        S(50,-26), S(42,-32), S(27,-37), S(8,25), S(20,-6), S(-13,-33), S(0,-18), S(20,60),
        S(-94,-9), S(12,49), S(-47,-9), S(-29,74), S(2,3), S(60,-20), S(6,-49), S(-87,211),
    ],
    [
        S(12,-24), S(-24,40), S(-102,3), S(-96,-41), S(35,-21), S(-128,35), S(-34,58), S(-66,-9),
        S(-19,2), S(-6,-19), S(-37,0), S(143,-68), S(3,-11), S(-45,25), S(-18,8), S(-14,20),
        S(50,-51), S(52,-36), S(13,-3), S(-23,21), S(54,16), S(-30,45), S(17,8), S(-1,4),
        S(60,-28), S(32,-10), S(-14,25), S(28,9), S(-27,28), S(-16,32), S(10,17), S(10,-2),
        S(51,-42), S(11,-10), S(11,-0), S(4,35), S(26,33), S(-7,51), S(9,10), S(50,-52),
        S(18,10), S(35,-12), S(4,-7), S(18,10), S(9,2), S(47,-16), S(19,38), S(14,8),
        S(56,28), S(4,14), S(-22,-14), S(15,-12), S(36,-36), S(-16,9), S(31,-2), S(24,-36),
        S(-113,32), S(29,-63), S(5,-7), S(-34,1), S(-13,-26), S(6,-3), S(-113,-30), S(-29,15),
    ],
    [
        S(50,-1), S(79,-30), S(228,-70), S(236,-76), S(151,-57), S(26,-13), S(135,-24), S(83,-2),
        S(2,7), S(19,-10), S(61,-35), S(151,-51), S(113,-59), S(9,15), S(24,1), S(-8,25),
        S(-11,15), S(-40,18), S(-72,51), S(21,-18), S(8,13), S(21,-3), S(-18,-10), S(-39,20),
        S(-28,18), S(-27,10), S(54,-19), S(-18,-1), S(-17,-6), S(-67,23), S(-21,3), S(7,-20),
        S(-54,16), S(-86,16), S(-42,7), S(-31,44), S(26,-36), S(-17,-3), S(-32,-10), S(-20,-0),
        S(-42,23), S(-40,5), S(-46,7), S(-53,21), S(-51,10), S(-32,13), S(-19,21), S(-59,11),
        S(-47,43), S(16,-41), S(10,-2), S(-45,35), S(-53,-1), S(-50,-12), S(-19,-18), S(-49,15),
        S(-52,29), S(-37,17), S(-29,19), S(-23,18), S(-28,12), S(-20,11), S(-52,14), S(-39,8),
    ],
    [
        S(104,-82), S(1,9), S(-134,118), S(-22,22), S(107,-4), S(94,-53), S(138,-103), S(27,-90),
        S(-14,66), S(-16,28), S(-10,89), S(31,35), S(59,-59), S(-7,62), S(-11,-11), S(28,-76),
        S(14,5), S(-27,32), S(-44,54), S(0,46), S(4,76), S(-18,14), S(-76,87), S(-24,33),
        S(-52,38), S(5,84), S(-19,55), S(-28,130), S(-27,55), S(-34,86), S(-28,1), S(-2,-17),
        S(-29,47), S(-69,32), S(-29,48), S(-22,6), S(-16,76), S(-39,78), S(-10,36), S(4,-57),
        S(23,-43), S(10,-20), S(-15,-55), S(-39,106), S(-39,33), S(-22,86), S(-31,72), S(27,21),
        S(-17,-109), S(3,-17), S(9,-35), S(-12,-14), S(-8,38), S(29,-83), S(12,-105), S(32,-78),
        S(-31,-78), S(12,-100), S(14,-104), S(-5,-39), S(18,-118), S(18,-76), S(56,-190), S(147,-86),
    ],
    [
        S(118,-84), S(318,-100), S(-160,-79), S(76,-22), S(-312,93), S(-304,97), S(-156,14), S(-191,-20),
        S(54,-29), S(212,3), S(-45,33), S(-185,27), S(-150,74), S(-129,50), S(-42,31), S(-260,36),
        S(64,2), S(-111,38), S(-71,54), S(-330,49), S(-104,42), S(-104,46), S(-78,47), S(-191,38),
        S(92,9), S(53,18), S(-86,28), S(186,-15), S(216,-21), S(118,0), S(71,14), S(-66,28),
        S(9,-23), S(35,7), S(-23,25), S(13,13), S(112,-16), S(66,10), S(55,7), S(12,5),
        S(0,-32), S(75,-15), S(73,-13), S(3,11), S(-4,10), S(35,3), S(57,-26), S(75,-15),
        S(150,-48), S(20,3), S(52,-14), S(25,-13), S(49,-11), S(36,-4), S(74,-27), S(107,-53),
        S(-14,-15), S(78,-40), S(46,-32), S(46,-62), S(46,-14), S(38,-33), S(77,-26), S(70,-62),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(77,-309), S(-23,-0), S(-34,44), S(-20,37), S(-9,44), S(-7,49), S(-3,55), S(7,52), S(11,28), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-50,139), S(-33,49), S(-32,-23), S(-25,-15), S(-22,-1), S(-20,14), S(-20,14), S(-18,15), S(-16,16), S(5,-17), S(26,-6), S(64,-68), S(67,-34), S(75,-86), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(103,-45), S(-50,-26), S(-42,-4), S(-41,6), S(-38,8), S(-27,7), S(-28,19), S(-20,16), S(-15,15), S(-15,15), S(4,4), S(-6,18), S(-2,17), S(-2,22), S(179,-73), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-250,-128), S(-250,-128), S(-252,-129), S(-128,86), S(-82,157), S(-110,161), S(-111,16), S(-115,49), S(-113,84), S(-115,139), S(-115,132), S(-111,139), S(-115,153), S(-112,154), S(-106,146), S(-103,128), S(-93,106), S(-112,129), S(-78,74), S(-74,72), S(-55,75), S(-30,17), S(-30,32), S(288,-212), S(380,-266), S(633,-406), S(1137,-609), S(220,-171)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(8,7), S(-8,20), S(5,27), S(-17,54), S(31,61), S(42,72), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,17), S(9,24), S(17,22), S(18,43), S(116,96), S(83,189), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(11,7), S(7,11), S(10,11), S(70,12), S(389,-79), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(17,-5);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(11,10);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(71,-17);
const SAFE_QUEEN_CHECK: ScorePair = S(18, -0);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(11,7), S(4,9), S(10,13), S(1,45)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-22,20), S(-20,-9), S(-19,-5), S(-18,1), S(-9,-1), S(6,-14), S(20,-12), S(45,-30), S(87,-45), S(83,-52), S(143,-96), S(38,-53), S(188,17), S(226,-282)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(-15,-39), S(68,35), S(75,85), S(72,98), S(-7,613), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(3,37), S(16,31), S(70,58), S(43,60), S(24,405), S(0,0)],
    [S(-3,12), S(-14,32), S(40,27), S(30,62), S(2,154), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(13,29), S(57,31), S(25,3), S(62,73), S(56,301), S(0,0)],
    [S(-4,10), S(17,6), S(-25,-3), S(37,89), S(24,226), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(3,57), S(53,43), S(87,28), S(-127,51), S(50,126), S(0,0)],
    [S(-10,17), S(6,13), S(28,-9), S(-142,15), S(48,252), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(16,11), S(44,40), S(85,-40), S(78,-6), S(2,6), S(0,0)],
    [S(-3,4), S(8,17), S(9,-15), S(-20,21), S(-16,-18), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 25;

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
