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
    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType;
    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(78,126), S(318,238), S(406,273), S(483,495), S(975,950), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(60,105), S(-31,105), S(54,79), S(71,3), S(48,30), S(-52,71), S(17,50), S(-32,98),
        S(10,53), S(-6,49), S(17,7), S(55,-29), S(42,-28), S(30,-17), S(11,24), S(10,38),
        S(-28,21), S(-19,8), S(-9,-14), S(9,-42), S(12,-39), S(5,-29), S(-4,-13), S(-8,-14),
        S(-36,6), S(-33,7), S(-8,-28), S(5,-36), S(6,-53), S(11,-38), S(-10,-31), S(-19,-19),
        S(-39,-4), S(-20,-18), S(-15,-27), S(-21,-22), S(-4,-35), S(-18,-26), S(18,-45), S(-16,-27),
        S(-38,15), S(-8,-3), S(-15,-13), S(-23,-10), S(-15,-26), S(17,-21), S(30,-33), S(-11,-30),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-168,-8), S(10,-10), S(-78,-19), S(-12,-39), S(-12,-68), S(-162,1), S(119,-87), S(-48,-113),
        S(-38,18), S(5,10), S(-1,-17), S(46,-25), S(17,12), S(-6,-5), S(-24,3), S(-65,19),
        S(48,-25), S(-30,-4), S(4,24), S(38,22), S(39,12), S(11,44), S(19,7), S(12,2),
        S(47,-23), S(16,1), S(24,15), S(48,32), S(26,26), S(43,16), S(18,19), S(43,10),
        S(8,-11), S(26,-25), S(16,41), S(10,46), S(23,51), S(24,32), S(6,15), S(23,24),
        S(-14,1), S(9,-28), S(-10,43), S(8,31), S(7,43), S(-1,29), S(25,-13), S(-2,11),
        S(-64,-68), S(5,-22), S(-6,-0), S(-4,25), S(8,-8), S(9,-4), S(-9,24), S(36,-34),
        S(-106,3), S(-12,29), S(-41,23), S(-10,-21), S(17,-11), S(10,26), S(-5,-17), S(25,-85),
    ],
    [
        S(-22,-43), S(-96,48), S(-46,-6), S(-20,-33), S(-105,-4), S(-121,10), S(-165,55), S(-10,49),
        S(24,-50), S(-0,2), S(-62,14), S(11,-25), S(-55,-7), S(9,-5), S(-78,24), S(-51,9),
        S(43,-57), S(57,-25), S(23,3), S(33,-39), S(50,-2), S(6,31), S(37,-8), S(29,9),
        S(56,-34), S(12,4), S(28,5), S(43,9), S(16,30), S(-3,14), S(4,8), S(43,-9),
        S(33,-13), S(8,-17), S(21,-3), S(17,36), S(19,28), S(-0,19), S(27,11), S(2,11),
        S(14,7), S(60,-22), S(2,0), S(13,7), S(2,14), S(34,-0), S(24,-36), S(14,8),
        S(44,5), S(5,5), S(29,-18), S(-3,-10), S(14,-17), S(19,-35), S(27,-29), S(23,-18),
        S(-34,54), S(8,5), S(5,-1), S(-53,11), S(-1,-37), S(-9,-7), S(-19,6), S(-31,52),
    ],
    [
        S(3,31), S(145,-37), S(179,-68), S(100,-50), S(92,-42), S(47,-17), S(-34,42), S(-3,29),
        S(13,19), S(45,-15), S(90,-20), S(79,-17), S(122,-28), S(60,-14), S(12,1), S(-44,38),
        S(-28,21), S(12,-5), S(10,2), S(45,-9), S(10,3), S(1,3), S(27,3), S(-26,35),
        S(11,0), S(-41,15), S(-3,18), S(35,-7), S(-6,-2), S(-25,16), S(-16,34), S(11,-7),
        S(-29,7), S(-34,6), S(11,14), S(-61,36), S(1,-13), S(-34,5), S(-18,-6), S(-46,10),
        S(-55,9), S(-5,-19), S(-33,1), S(-59,24), S(-3,-12), S(-27,-16), S(-33,14), S(-52,1),
        S(-64,-2), S(-36,-1), S(-18,-8), S(-9,-28), S(-8,-28), S(-30,10), S(-18,-18), S(-81,-11),
        S(-44,16), S(-32,11), S(-14,10), S(-13,8), S(10,-10), S(-7,-0), S(-46,23), S(-37,-1),
    ],
    [
        S(18,17), S(48,-25), S(-38,80), S(-25,62), S(77,-27), S(70,-11), S(-67,108), S(26,-27),
        S(-2,-25), S(-29,16), S(-52,64), S(3,9), S(25,21), S(-31,108), S(1,-16), S(18,-1),
        S(-12,36), S(-4,-10), S(-15,35), S(-46,118), S(-36,81), S(46,45), S(4,54), S(45,-40),
        S(16,-17), S(13,24), S(-39,106), S(-27,101), S(-6,80), S(-20,49), S(9,8), S(37,-57),
        S(1,-36), S(-10,-40), S(6,32), S(-9,47), S(-12,61), S(-14,31), S(13,9), S(22,-23),
        S(2,-58), S(4,-24), S(14,-24), S(2,-8), S(1,-14), S(-4,25), S(9,-25), S(20,-130),
        S(-42,79), S(-12,-24), S(16,-84), S(14,-54), S(2,-6), S(29,-90), S(25,-81), S(-16,-100),
        S(-3,11), S(15,-116), S(25,-81), S(15,-47), S(29,-111), S(5,-139), S(-75,0), S(-81,54),
    ],
    [
        S(119,-157), S(-200,-36), S(246,-63), S(223,-28), S(-166,151), S(-275,180), S(-30,-18), S(-16,-37),
        S(-306,58), S(-9,24), S(42,35), S(-317,142), S(-257,160), S(-379,129), S(-42,36), S(-137,-6),
        S(7,17), S(-104,53), S(-158,74), S(-130,52), S(-48,32), S(-301,107), S(-350,144), S(-409,163),
        S(-87,17), S(125,19), S(97,15), S(200,-53), S(390,-71), S(149,3), S(-28,50), S(1,4),
        S(60,-59), S(145,-22), S(81,-10), S(136,-15), S(222,-39), S(109,1), S(57,-10), S(-46,-6),
        S(101,-46), S(116,-51), S(31,-7), S(73,-12), S(11,4), S(71,-14), S(99,-47), S(3,-28),
        S(110,-76), S(62,-52), S(72,-35), S(31,-22), S(24,-16), S(37,-20), S(90,-42), S(80,-65),
        S(5,-62), S(80,-60), S(61,-47), S(-18,-31), S(62,-82), S(24,-62), S(92,-69), S(69,-97),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-141,-255), S(-28,5), S(-10,22), S(4,30), S(18,44), S(23,45), S(29,53), S(42,55), S(63,1), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-52,-164), S(-34,-91), S(-52,-25), S(-39,18), S(-27,30), S(-24,52), S(-22,59), S(-16,55), S(-11,56), S(4,39), S(-6,54), S(47,-11), S(32,26), S(201,-97), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-387,-243), S(-80,54), S(-21,-52), S(-16,-15), S(-6,7), S(3,24), S(14,29), S(16,35), S(23,36), S(30,41), S(34,43), S(33,44), S(62,33), S(60,36), S(234,-70), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-312,-267), S(-312,-267), S(-654,-309), S(-80,304), S(-60,-95), S(-45,32), S(-42,93), S(-35,92), S(-35,123), S(-30,114), S(-33,140), S(-25,143), S(-23,139), S(-27,161), S(-26,153), S(-29,152), S(-29,149), S(-22,123), S(-26,109), S(1,88), S(-4,87), S(23,23), S(89,-38), S(207,-135), S(251,-182), S(343,-299), S(578,-342), S(355,-291)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(1,12), S(-9,38), S(-16,64), S(10,81), S(23,104), S(57,147), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(5,12), S(8,18), S(16,23), S(45,57), S(54,250), S(576,746), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(14,20), S(13,14), S(13,14), S(52,29), S(388,-45), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(25,-18);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(18,17);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(95,-19);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(31,32);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(13,19), S(-4,21), S(22,9), S(-9,65)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-57,49), S(-50,-4), S(-46,2), S(-36,7), S(-9,-13), S(21,-18), S(66,-44), S(104,-72), S(191,-93), S(184,-74), S(319,-233), S(358,-176), S(210,-61), S(272,-274)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-13,-67), S(77,40), S(59,63), S(59,42), S(12,151), S(0,0)],
    [S(-7,-51), S(204,167), S(235,201), S(244,416), S(445,1419), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(0,67), S(16,-180), S(45,37), S(67,31), S(41,136), S(0,0)],
        [S(-9,17), S(-12,-184), S(33,51), S(57,28), S(21,213), S(0,0)],
    ],
    [
        [S(20,69), S(49,-122), S(149,136), S(160,391), S(234,1293), S(0,0)],
        [S(-8,15), S(-2,-184), S(49,44), S(88,211), S(193,804), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(10,44), S(61,23), S(-25,-3), S(41,64), S(107,24), S(0,0)],
        [S(3,9), S(27,40), S(-35,-3), S(32,105), S(57,168), S(0,0)],
    ],
    [
        [S(27,62), S(161,131), S(97,24), S(226,420), S(435,1254), S(0,0)],
        [S(-1,10), S(25,46), S(-36,-19), S(93,201), S(392,435), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(7,69), S(61,54), S(76,50), S(1,-220), S(103,35), S(0,0)],
        [S(-6,34), S(14,29), S(38,22), S(-3,-225), S(122,64), S(0,0)],
    ],
    [
        [S(6,91), S(110,187), S(155,194), S(150,177), S(540,1318), S(0,0)],
        [S(-5,16), S(23,7), S(34,8), S(-5,-248), S(281,703), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(16,-6), S(39,13), S(47,81), S(79,-8), S(-58,-190), S(0,0)],
        [S(-6,14), S(-3,4), S(13,13), S(9,-26), S(-120,-139), S(0,0)],
    ],
    [
        [S(24,70), S(127,42), S(175,127), S(293,212), S(454,857), S(0,0)],
        [S(-1,-3), S(-7,-2), S(-11,37), S(-2,5), S(-93,-201), S(0,0)],
    ],
];
#[rustfmt::skip]
const TEMPO: i32 = 26;

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

    fn threat_by_pawn(stm: bool, pt: PieceType) -> Self::ScorePairType {
        THREAT_BY_PAWN[stm as usize][pt as usize]
    }

    fn threat_by_knight(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_KNIGHT[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_bishop(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_BISHOP[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_rook(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_ROOK[stm as usize][defended as usize][pt as usize]
    }

    fn threat_by_queen(stm: bool, pt: PieceType, defended: bool) -> Self::ScorePairType {
        THREAT_BY_QUEEN[stm as usize][defended as usize][pt as usize]
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
    let stm = color == board.stm();
    let mut eval = Params::ScorePairType::default();

    let defended_bb = eval_data.attacked_by_2[!color as usize]
        | eval_data.attacked_by[!color as usize][PieceType::Pawn as usize]
        | (eval_data.attacked[!color as usize] & !eval_data.attacked_by_2[color as usize]);

    let mut pawn_threats =
        eval_data.attacked_by[color as usize][PieceType::Pawn as usize] & board.colors(!color);
    while pawn_threats.any() {
        let threatened = board.piece_at(pawn_threats.poplsb()).unwrap().piece_type();
        eval += Params::threat_by_pawn(stm, threatened);
    }

    let mut knight_threats =
        eval_data.attacked_by[color as usize][PieceType::Knight as usize] & board.colors(!color);
    while knight_threats.any() {
        let threat = knight_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_knight(stm, threatened, defended);
    }

    let mut bishop_threats =
        eval_data.attacked_by[color as usize][PieceType::Bishop as usize] & board.colors(!color);
    while bishop_threats.any() {
        let threat = bishop_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_bishop(stm, threatened, defended);
    }

    let mut rook_threats =
        eval_data.attacked_by[color as usize][PieceType::Rook as usize] & board.colors(!color);
    while rook_threats.any() {
        let threat = rook_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_rook(stm, threatened, defended);
    }

    let mut queen_threats =
        eval_data.attacked_by[color as usize][PieceType::Queen as usize] & board.colors(!color);
    while queen_threats.any() {
        let threat = queen_threats.poplsb();
        let threatened = board.piece_at(threat).unwrap().piece_type();
        let defended = defended_bb.has(threat);
        eval += Params::threat_by_queen(stm, threatened, defended);
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
