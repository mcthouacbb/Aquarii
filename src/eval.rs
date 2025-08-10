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
const MATERIAL: [ScorePair; 6] = [S(82,140), S(318,273), S(410,309), S(495,543), S(985,1012), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(73,110), S(-44,117), S(73,87), S(80,6), S(24,37), S(-45,69), S(4,54), S(-41,114),
        S(13,60), S(-10,60), S(15,10), S(57,-25), S(51,-33), S(21,-11), S(13,26), S(6,45),
        S(-27,23), S(-20,9), S(-7,-20), S(9,-47), S(15,-43), S(7,-36), S(-4,-16), S(-6,-18),
        S(-39,8), S(-34,7), S(-7,-31), S(8,-43), S(6,-58), S(13,-42), S(-10,-36), S(-20,-23),
        S(-41,-6), S(-19,-22), S(-16,-29), S(-22,-28), S(-4,-39), S(-20,-31), S(21,-50), S(-14,-31),
        S(-37,16), S(-5,-3), S(-16,-13), S(-25,-10), S(-16,-27), S(18,-20), S(34,-35), S(-9,-32),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-144,-10), S(-17,-12), S(-110,-28), S(-59,-32), S(-51,-62), S(-203,3), S(48,-84), S(-21,-101),
        S(-46,19), S(9,13), S(14,-28), S(39,-14), S(12,13), S(-7,-8), S(8,-19), S(-53,16),
        S(46,-27), S(-31,5), S(-0,28), S(47,26), S(40,18), S(2,61), S(21,9), S(7,-3),
        S(56,-30), S(22,5), S(25,14), S(49,40), S(28,32), S(41,24), S(21,25), S(46,23),
        S(12,-23), S(29,-23), S(22,48), S(14,52), S(22,59), S(30,36), S(15,9), S(30,19),
        S(-11,-8), S(10,-32), S(-7,44), S(11,36), S(11,51), S(2,32), S(29,-14), S(3,11),
        S(-75,-76), S(11,-29), S(-1,6), S(-2,34), S(12,-11), S(11,-0), S(-4,24), S(44,-45),
        S(-94,-35), S(-10,21), S(-37,24), S(-8,-23), S(16,-9), S(14,32), S(-3,-17), S(62,-106),
    ],
    [
        S(5,-56), S(-34,31), S(-106,6), S(-47,-33), S(-143,10), S(-201,24), S(-196,62), S(2,61),
        S(30,-56), S(4,2), S(-65,15), S(-35,-18), S(-90,3), S(17,-7), S(-87,25), S(-34,1),
        S(37,-61), S(61,-26), S(13,10), S(37,-40), S(39,3), S(-52,58), S(44,-4), S(27,9),
        S(61,-38), S(23,-0), S(38,-2), S(57,8), S(23,33), S(-13,21), S(17,8), S(57,-17),
        S(46,-20), S(7,-26), S(29,-7), S(22,42), S(19,41), S(10,21), S(36,12), S(13,6),
        S(26,1), S(72,-21), S(14,-3), S(21,5), S(11,16), S(38,2), S(35,-42), S(22,11),
        S(49,6), S(12,2), S(38,-23), S(4,-13), S(22,-22), S(24,-31), S(36,-32), S(29,-48),
        S(-26,57), S(12,11), S(11,-6), S(-58,14), S(4,-45), S(-4,-12), S(-37,10), S(-27,62),
    ],
    [
        S(-16,48), S(138,-26), S(193,-68), S(91,-48), S(67,-33), S(37,-11), S(-26,47), S(2,36),
        S(25,18), S(50,-13), S(100,-20), S(84,-15), S(127,-24), S(63,-6), S(22,-0), S(-39,45),
        S(-33,28), S(9,-2), S(-15,11), S(37,-3), S(8,9), S(-22,12), S(34,1), S(-28,37),
        S(9,4), S(-41,19), S(-6,24), S(41,-13), S(-6,-4), S(-30,23), S(-11,39), S(19,-11),
        S(-28,-1), S(-50,11), S(19,12), S(-62,34), S(-4,-12), S(-38,6), S(6,-15), S(-40,6),
        S(-56,2), S(-10,-21), S(-28,-5), S(-67,28), S(-4,-16), S(-17,-33), S(-28,14), S(-51,0),
        S(-68,-6), S(-37,-5), S(-19,-13), S(-5,-35), S(-8,-41), S(-33,7), S(-13,-26), S(-80,-30),
        S(-46,15), S(-34,11), S(-13,7), S(-11,9), S(18,-15), S(-1,-4), S(-43,22), S(-33,-8),
    ],
    [
        S(31,-1), S(35,-5), S(-35,117), S(-66,89), S(95,-2), S(121,-47), S(-75,135), S(59,-42),
        S(1,-20), S(-41,52), S(-73,103), S(-7,39), S(4,57), S(-46,164), S(-1,-16), S(8,14),
        S(-18,44), S(-18,24), S(-19,47), S(-49,116), S(-30,84), S(21,91), S(3,58), S(41,-30),
        S(20,-25), S(12,26), S(-44,140), S(-27,108), S(-11,105), S(-31,73), S(6,6), S(33,-36),
        S(4,-51), S(-10,-34), S(4,50), S(-13,57), S(-13,73), S(-17,37), S(9,24), S(26,-40),
        S(11,-90), S(5,-21), S(19,-25), S(8,-27), S(5,-40), S(3,11), S(13,-38), S(24,-154),
        S(-38,82), S(-7,-51), S(26,-114), S(20,-83), S(7,-22), S(39,-145), S(27,-98), S(-16,-120),
        S(-7,8), S(20,-146), S(30,-102), S(21,-74), S(40,-141), S(0,-167), S(-47,-78), S(-94,53),
    ],
    [
        S(-73,-179), S(-89,-50), S(281,-64), S(349,-50), S(-249,177), S(-416,214), S(53,-27), S(33,-43),
        S(-351,33), S(-74,47), S(74,23), S(-311,156), S(-297,180), S(-421,140), S(-16,27), S(-173,-6),
        S(18,27), S(-79,60), S(-201,87), S(-151,63), S(-36,38), S(-337,124), S(-384,162), S(-485,188),
        S(-151,21), S(99,28), S(154,13), S(212,-55), S(426,-75), S(181,3), S(-25,61), S(23,5),
        S(39,-56), S(192,-26), S(61,-1), S(144,-10), S(244,-39), S(116,4), S(62,-15), S(-44,-9),
        S(114,-56), S(126,-55), S(33,-11), S(88,-13), S(13,9), S(79,-16), S(101,-50), S(10,-33),
        S(120,-79), S(66,-58), S(78,-38), S(37,-24), S(29,-19), S(43,-21), S(106,-49), S(91,-72),
        S(21,-88), S(93,-65), S(75,-54), S(-15,-36), S(75,-91), S(32,-72), S(110,-78), S(79,-107),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-173,-219), S(-29,9), S(-11,15), S(7,28), S(22,44), S(28,37), S(35,47), S(49,50), S(72,-11), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-101,-67), S(-44,-110), S(-55,-35), S(-39,14), S(-26,27), S(-23,49), S(-21,57), S(-15,54), S(-9,54), S(4,38), S(-2,52), S(57,-19), S(42,5), S(232,-118), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-378,-242), S(-67,36), S(-29,-62), S(-24,-16), S(-13,8), S(-1,26), S(10,32), S(13,38), S(21,39), S(29,46), S(33,46), S(33,51), S(65,36), S(66,37), S(245,-75), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-313,-281), S(-313,-281), S(-680,-341), S(-131,406), S(-44,-156), S(-37,19), S(-37,114), S(-30,96), S(-29,138), S(-24,119), S(-27,151), S(-19,144), S(-17,144), S(-22,175), S(-22,158), S(-23,155), S(-27,160), S(-19,130), S(-29,122), S(-2,108), S(-11,98), S(18,37), S(92,-39), S(154,-121), S(270,-205), S(276,-283), S(653,-415), S(394,-350)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(1,16), S(-11,46), S(-18,72), S(10,93), S(27,114), S(60,163), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,15), S(9,18), S(17,27), S(41,74), S(97,231), S(653,883), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(15,21), S(13,14), S(14,16), S(54,31), S(423,-63), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(23,-15);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(20,20);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(98,-15);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(34,42);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(13,25), S(-5,28), S(23,15), S(-10,82)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-64,59), S(-56,-3), S(-49,2), S(-41,8), S(-10,-17), S(23,-23), S(75,-54), S(120,-90), S(222,-126), S(194,-56), S(323,-245), S(351,-143), S(287,-210), S(199,-107)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(-7,-64), S(129,125), S(138,168), S(136,265), S(-26,1039), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(12,77), S(31,-395), S(95,93), S(98,145), S(27,624), S(0,0)],
    [S(-9,20), S(-6,-450), S(48,55), S(71,92), S(12,399), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(17,62), S(100,68), S(21,25), S(79,208), S(124,278), S(0,0)],
    [S(2,9), S(29,56), S(-21,-25), S(66,156), S(60,291), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(9,90), S(93,96), S(122,97), S(21,-113), S(98,238), S(0,0)],
    [S(-4,25), S(21,17), S(36,18), S(-31,-190), S(85,350), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(21,29), S(74,29), S(132,44), S(134,-2), S(20,25), S(0,0)],
    [S(-4,9), S(-4,-3), S(-3,41), S(5,-13), S(-22,-26), S(0,0)],
];
#[rustfmt::skip]
const TEMPO: i32 = 49;

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
