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
const MATERIAL: [ScorePair; 6] = [S(73,126), S(288,243), S(371,271), S(440,487), S(875,914), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(58,103), S(-35,105), S(63,71), S(64,9), S(25,37), S(-26,54), S(11,48), S(-37,101),
        S(5,55), S(-10,53), S(13,10), S(46,-20), S(36,-30), S(20,-12), S(7,23), S(9,37),
        S(-24,22), S(-17,8), S(-7,-15), S(8,-43), S(15,-39), S(6,-33), S(-4,-13), S(-3,-16),
        S(-34,5), S(-31,6), S(-6,-28), S(7,-38), S(7,-53), S(11,-38), S(-9,-32), S(-17,-20),
        S(-35,-5), S(-17,-17), S(-15,-26), S(-19,-23), S(-3,-34), S(-18,-24), S(19,-44), S(-13,-28),
        S(-32,14), S(-5,-1), S(-15,-10), S(-22,-9), S(-14,-29), S(16,-18), S(31,-32), S(-8,-27),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-133,-30), S(-53,16), S(-80,-34), S(-24,-33), S(-10,-68), S(-166,-7), S(22,-60), S(-16,-119),
        S(-44,16), S(2,13), S(7,-22), S(26,-8), S(14,4), S(-19,-2), S(6,-14), S(-55,18),
        S(49,-31), S(-23,1), S(0,25), S(39,29), S(33,21), S(8,53), S(18,3), S(5,3),
        S(46,-22), S(21,1), S(30,15), S(49,28), S(24,34), S(39,24), S(20,19), S(40,17),
        S(9,-19), S(30,-25), S(22,42), S(11,47), S(21,53), S(27,34), S(13,12), S(25,25),
        S(-11,-5), S(8,-28), S(-6,44), S(12,33), S(10,51), S(1,32), S(26,-17), S(0,6),
        S(-64,-66), S(10,-25), S(-6,7), S(-4,26), S(10,-11), S(10,-9), S(-2,7), S(37,-35),
        S(-74,-18), S(-12,23), S(-34,13), S(-8,-15), S(9,2), S(16,28), S(-5,-12), S(45,-90),
    ],
    [
        S(6,-47), S(-32,27), S(-101,11), S(-40,-39), S(-122,-2), S(-193,28), S(-137,52), S(17,52),
        S(30,-49), S(4,8), S(-52,14), S(-14,-12), S(-93,6), S(13,-2), S(-72,20), S(-42,15),
        S(32,-48), S(55,-22), S(9,13), S(30,-32), S(39,2), S(-49,52), S(30,0), S(18,7),
        S(51,-32), S(20,-8), S(29,-1), S(50,6), S(21,26), S(-13,21), S(15,2), S(47,-13),
        S(35,-10), S(2,-20), S(26,-8), S(17,38), S(18,28), S(8,18), S(36,5), S(8,13),
        S(20,-1), S(63,-17), S(8,-1), S(17,1), S(7,16), S(35,1), S(31,-37), S(23,1),
        S(53,-6), S(10,1), S(33,-20), S(1,-14), S(19,-21), S(19,-28), S(31,-30), S(31,-41),
        S(-28,55), S(12,7), S(9,-5), S(-58,15), S(-3,-37), S(-5,-7), S(-19,-0), S(-13,51),
    ],
    [
        S(-6,40), S(124,-26), S(169,-59), S(52,-34), S(60,-27), S(32,-12), S(-41,42), S(4,33),
        S(20,18), S(47,-10), S(93,-20), S(72,-10), S(115,-24), S(59,-10), S(2,12), S(-27,33),
        S(-27,24), S(0,3), S(-2,7), S(28,-3), S(13,4), S(-26,11), S(22,6), S(-25,29),
        S(6,2), S(-38,14), S(0,19), S(39,-11), S(2,-9), S(-20,18), S(-6,29), S(20,-16),
        S(-22,-3), S(-38,10), S(7,16), S(-42,26), S(-1,-10), S(-29,5), S(-8,-3), S(-37,5),
        S(-51,6), S(-5,-18), S(-21,-3), S(-59,25), S(1,-15), S(-16,-24), S(-29,13), S(-44,2),
        S(-60,-6), S(-33,-8), S(-15,-14), S(-4,-31), S(-10,-37), S(-29,7), S(-12,-23), S(-72,-27),
        S(-40,14), S(-28,10), S(-10,7), S(-8,8), S(19,-16), S(1,-4), S(-36,18), S(-28,-6),
    ],
    [
        S(26,12), S(22,15), S(-38,117), S(-63,91), S(114,-37), S(103,-19), S(-73,129), S(44,-22),
        S(-7,-15), S(-38,41), S(-74,98), S(-22,54), S(4,55), S(-31,122), S(-5,-17), S(12,5),
        S(-17,44), S(-17,32), S(-20,38), S(-47,96), S(-34,79), S(14,76), S(1,48), S(43,-22),
        S(21,-15), S(16,26), S(-36,113), S(-25,106), S(-5,89), S(-30,56), S(4,13), S(35,-48),
        S(5,-45), S(-12,-29), S(4,41), S(-9,46), S(-8,72), S(-12,20), S(8,23), S(20,-24),
        S(9,-80), S(9,-36), S(20,-31), S(8,-27), S(7,-27), S(2,11), S(17,-51), S(17,-108),
        S(-41,87), S(-3,-48), S(23,-99), S(20,-74), S(7,-18), S(38,-132), S(23,-71), S(-9,-120),
        S(-7,9), S(17,-119), S(32,-124), S(22,-72), S(33,-130), S(8,-154), S(-40,-81), S(-80,30),
    ],
    [
        S(88,-144), S(-229,-26), S(323,-63), S(183,-22), S(-138,152), S(-287,182), S(12,-22), S(115,-43),
        S(-334,41), S(-23,27), S(-6,33), S(-259,135), S(-244,162), S(-384,127), S(13,18), S(-198,9),
        S(32,1), S(-72,55), S(-175,70), S(-153,59), S(-16,30), S(-312,111), S(-349,146), S(-419,162),
        S(-137,26), S(75,23), S(114,12), S(185,-50), S(391,-70), S(138,9), S(-20,51), S(-8,5),
        S(58,-64), S(173,-26), S(69,-8), S(118,-8), S(208,-36), S(88,7), S(41,-8), S(-57,-4),
        S(118,-56), S(108,-49), S(13,-4), S(75,-13), S(18,6), S(65,-13), S(90,-46), S(1,-28),
        S(110,-78), S(65,-54), S(64,-32), S(32,-24), S(23,-16), S(38,-21), S(91,-42), S(79,-64),
        S(13,-73), S(84,-61), S(65,-49), S(-14,-32), S(65,-83), S(26,-65), S(97,-70), S(70,-95),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-138,-200), S(-28,6), S(-12,18), S(6,25), S(18,38), S(22,37), S(29,43), S(41,43), S(63,-11), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-67,-154), S(-42,-93), S(-50,-29), S(-36,18), S(-24,27), S(-22,53), S(-20,58), S(-13,55), S(-11,57), S(3,43), S(-2,51), S(50,-9), S(26,22), S(210,-96), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-363,-202), S(-46,25), S(-26,-58), S(-21,-15), S(-11,7), S(-1,24), S(9,30), S(13,34), S(21,33), S(27,41), S(32,40), S(28,45), S(63,30), S(59,32), S(216,-67), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-276,-261), S(-276,-261), S(-631,-323), S(-121,476), S(-46,-128), S(-34,18), S(-33,98), S(-27,91), S(-26,101), S(-22,102), S(-26,138), S(-19,130), S(-16,128), S(-19,152), S(-21,145), S(-22,138), S(-22,142), S(-14,110), S(-20,93), S(0,92), S(-5,81), S(21,23), S(100,-64), S(149,-116), S(268,-193), S(116,-179), S(661,-395), S(361,-337)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-3,16), S(-10,40), S(-17,66), S(7,83), S(26,103), S(53,146), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(4,12), S(9,16), S(16,23), S(36,65), S(74,221), S(623,546), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(14,18), S(13,12), S(13,15), S(53,27), S(359,-56), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(21,-14);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(17,18);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(93,-16);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(30,35);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(11,22), S(-5,23), S(20,10), S(-10,70)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-60,51), S(-52,-3), S(-48,2), S(-38,7), S(-10,-13), S(19,-19), S(65,-46), S(108,-75), S(200,-112), S(165,-55), S(284,-199), S(309,-110), S(289,-251), S(219,-171)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(-5,-71), S(117,109), S(121,158), S(111,241), S(-23,912), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(10,67), S(11,-276), S(88,82), S(91,134), S(24,530), S(0,0)],
    [S(-8,15), S(-25,-323), S(40,49), S(63,84), S(6,332), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(13,56), S(86,66), S(17,20), S(65,186), S(112,206), S(0,0)],
    [S(1,10), S(27,45), S(-17,-20), S(57,135), S(58,231), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(6,80), S(81,89), S(104,87), S(48,-86), S(88,220), S(0,0)],
    [S(-4,24), S(20,18), S(33,16), S(-8,-148), S(72,305), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(21,26), S(67,30), S(117,51), S(116,2), S(21,16), S(0,0)],
    [S(-4,7), S(-1,-9), S(-1,34), S(3,-7), S(-24,-18), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 44;

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
