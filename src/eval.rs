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
    fn our_passer_dist(dist: i32) -> Self::ScorePairType;
    fn their_passer_dist(dist: i32) -> Self::ScorePairType;
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
    fn push_threat(stm: bool) -> Self::ScorePairType;
    fn tempo() -> Self::ScoreType;
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [S(75,127), S(317,246), S(411,269), S(487,492), S(970,953), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(59,107), S(-20,98), S(46,84), S(78,0), S(48,32), S(-38,61), S(11,54), S(-40,104),
        S(11,53), S(-14,52), S(17,8), S(45,-21), S(35,-29), S(24,-14), S(5,19), S(6,39),
        S(-23,20), S(-18,8), S(-8,-17), S(12,-43), S(15,-41), S(5,-32), S(-6,-13), S(-5,-16),
        S(-34,5), S(-35,10), S(-7,-29), S(3,-36), S(2,-49), S(11,-37), S(-12,-29), S(-18,-19),
        S(-36,-4), S(-18,-18), S(-14,-26), S(-18,-22), S(-3,-34), S(-15,-27), S(18,-44), S(-13,-28),
        S(-35,14), S(-6,-3), S(-14,-13), S(-22,-13), S(-15,-28), S(18,-21), S(30,-33), S(-10,-29),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(-163,-26), S(11,-6), S(-71,-34), S(15,-62), S(-52,-60), S(-167,-1), S(135,-75), S(-35,-103),
        S(-41,18), S(21,2), S(-9,-15), S(47,-15), S(9,11), S(-12,-3), S(-22,-3), S(-67,22),
        S(47,-29), S(-34,5), S(4,16), S(33,24), S(35,11), S(6,49), S(10,15), S(2,-1),
        S(49,-19), S(19,-2), S(22,11), S(47,30), S(22,29), S(44,16), S(18,22), S(36,23),
        S(10,-19), S(38,-29), S(22,42), S(14,45), S(23,51), S(33,29), S(22,9), S(29,13),
        S(-16,-4), S(6,-21), S(-10,41), S(9,32), S(3,49), S(0,26), S(23,-11), S(-3,12),
        S(-70,-61), S(1,-17), S(-7,4), S(-7,25), S(5,-2), S(7,-3), S(-14,28), S(28,-33),
        S(-94,-3), S(-18,30), S(-37,23), S(-9,-19), S(19,-20), S(7,28), S(-10,-15), S(37,-79),
    ],
    [
        S(-18,-34), S(-86,38), S(-20,-11), S(-41,-38), S(-99,-1), S(-140,8), S(-138,52), S(-7,46),
        S(8,-43), S(2,2), S(-69,18), S(16,-23), S(-61,-5), S(-11,4), S(-85,30), S(-46,-1),
        S(39,-50), S(50,-24), S(23,5), S(28,-36), S(44,2), S(4,29), S(31,-1), S(26,11),
        S(53,-31), S(17,-4), S(36,-1), S(45,10), S(22,27), S(1,16), S(10,10), S(52,-19),
        S(30,-6), S(18,-19), S(29,-7), S(24,32), S(23,28), S(11,17), S(36,13), S(13,5),
        S(13,5), S(58,-19), S(3,0), S(11,9), S(2,14), S(35,-1), S(26,-41), S(12,10),
        S(44,-4), S(2,5), S(27,-17), S(-6,-6), S(12,-16), S(17,-35), S(24,-27), S(25,-42),
        S(-30,56), S(4,4), S(1,1), S(-60,21), S(-2,-34), S(-12,-3), S(-34,4), S(-42,66),
    ],
    [
        S(7,31), S(155,-39), S(180,-68), S(87,-50), S(85,-42), S(40,-14), S(-41,41), S(5,27),
        S(11,21), S(44,-16), S(91,-22), S(83,-16), S(115,-26), S(46,-6), S(14,4), S(-57,45),
        S(-38,26), S(7,-2), S(-2,7), S(31,-1), S(0,9), S(-11,7), S(16,4), S(-27,32),
        S(5,-0), S(-41,16), S(-5,19), S(38,-9), S(-1,-3), S(-12,16), S(-1,28), S(14,-11),
        S(-27,6), S(-22,0), S(17,7), S(-52,31), S(11,-13), S(-20,-2), S(-4,-9), S(-45,12),
        S(-53,8), S(-7,-15), S(-28,2), S(-44,17), S(-4,-11), S(-24,-16), S(-31,16), S(-53,-1),
        S(-67,1), S(-42,0), S(-18,-9), S(-6,-26), S(-7,-32), S(-30,13), S(-20,-19), S(-79,-19),
        S(-47,18), S(-33,9), S(-15,13), S(-14,8), S(14,-14), S(-5,-2), S(-46,23), S(-38,-1),
    ],
    [
        S(1,35), S(58,-47), S(-66,104), S(-12,35), S(43,-4), S(112,-71), S(-71,116), S(23,-28),
        S(-3,-24), S(-35,29), S(-59,80), S(-28,43), S(21,19), S(-45,131), S(2,-28), S(25,-13),
        S(-21,26), S(-19,23), S(-24,39), S(-51,108), S(-31,74), S(48,43), S(1,45), S(41,-37),
        S(20,-10), S(15,21), S(-32,96), S(-31,118), S(3,71), S(-20,55), S(16,7), S(37,-52),
        S(10,-66), S(2,-62), S(16,25), S(-4,39), S(-0,50), S(-1,17), S(23,15), S(27,-35),
        S(9,-76), S(2,-4), S(14,-25), S(-0,1), S(3,-19), S(1,15), S(6,-3), S(13,-103),
        S(-42,78), S(-8,-35), S(15,-72), S(11,-47), S(-2,5), S(28,-86), S(15,-51), S(-8,-110),
        S(-11,27), S(16,-105), S(25,-90), S(10,-41), S(28,-107), S(9,-149), S(-68,-10), S(-57,18),
    ],
    [
        S(69,-153), S(-213,-42), S(292,-67), S(232,-33), S(-223,157), S(-285,180), S(-10,-24), S(100,-39),
        S(-304,18), S(-45,34), S(26,24), S(-352,146), S(-255,167), S(-386,128), S(-49,36), S(-20,-19),
        S(10,22), S(-96,60), S(-166,72), S(-138,53), S(-62,35), S(-358,118), S(-351,147), S(-422,164),
        S(-117,16), S(95,26), S(98,18), S(177,-51), S(406,-74), S(154,3), S(-21,47), S(41,-4),
        S(63,-54), S(166,-24), S(107,-13), S(150,-14), S(243,-41), S(111,2), S(67,-14), S(-50,-6),
        S(115,-56), S(104,-42), S(37,-8), S(68,-10), S(10,8), S(81,-17), S(96,-46), S(5,-28),
        S(112,-79), S(58,-51), S(64,-33), S(25,-20), S(16,-11), S(31,-19), S(86,-39), S(79,-63),
        S(-1,-65), S(73,-54), S(56,-48), S(-24,-24), S(57,-80), S(19,-61), S(87,-68), S(60,-88),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(-177,-203), S(-22,10), S(-3,11), S(10,21), S(21,40), S(28,38), S(34,42), S(45,48), S(64,-6), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-56,-157), S(-32,-110), S(-54,-28), S(-40,18), S(-30,33), S(-26,52), S(-24,58), S(-19,57), S(-14,58), S(2,41), S(-5,56), S(37,-6), S(35,25), S(226,-97), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-389,-245), S(-68,39), S(-21,-51), S(-18,-14), S(-7,5), S(2,26), S(13,30), S(17,35), S(24,36), S(30,44), S(36,43), S(31,47), S(63,33), S(57,40), S(230,-67), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-295,-269), S(-295,-269), S(-628,-321), S(-119,417), S(-46,-123), S(-42,33), S(-39,84), S(-31,93), S(-32,108), S(-25,106), S(-29,127), S(-22,139), S(-20,133), S(-23,163), S(-23,150), S(-25,142), S(-21,139), S(-21,126), S(-20,102), S(5,89), S(-2,84), S(32,13), S(80,-38), S(222,-146), S(256,-179), S(272,-261), S(510,-307), S(381,-335)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(-3,14), S(-12,38), S(-16,64), S(6,83), S(24,104), S(56,148), S(0,0)];
#[rustfmt::skip]
const OUR_PASSER_DIST: [ScorePair; 8] = [S(0, 0); 8];
#[rustfmt::skip]
const THEIR_PASSER_DIST: [ScorePair; 8] = [S(0, 0); 8];

#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(6,12), S(9,16), S(16,24), S(42,65), S(44,266), S(564,747), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(14,19), S(13,13), S(15,13), S(55,29), S(411,-66), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(27,-17);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(19,17);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(96,-18);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(31,31);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(11,20), S(-5,21), S(19,10), S(-9,68)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-58,50), S(-50,-3), S(-47,2), S(-35,8), S(-8,-12), S(21,-16), S(68,-45), S(115,-78), S(193,-101), S(186,-71), S(310,-230), S(347,-168), S(209,-79), S(303,-378)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [[ScorePair; 6]; 2] = [
    [S(-11,-74), S(79,42), S(63,63), S(67,42), S(13,168), S(0,0)],
    [S(-5,-61), S(205,178), S(238,208), S(258,431), S(475,1401), S(0,0)],
];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,63), S(19,-160), S(41,35), S(67,29), S(38,174), S(0,0)],
        [S(-2,15), S(-6,-176), S(36,55), S(64,12), S(20,229), S(0,0)],
    ],
    [
        [S(25,64), S(64,-111), S(143,160), S(171,406), S(206,1302), S(0,0)],
        [S(-4,15), S(2,-171), S(48,42), S(86,199), S(202,693), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(4,48), S(62,17), S(-27,0), S(46,62), S(110,55), S(0,0)],
        [S(4,11), S(28,43), S(-32,-6), S(45,96), S(63,198), S(0,0)],
    ],
    [
        [S(27,65), S(158,134), S(89,25), S(217,416), S(403,1181), S(0,0)],
        [S(-2,13), S(23,49), S(-35,-20), S(90,204), S(390,421), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(8,66), S(62,55), S(70,51), S(30,-218), S(105,43), S(0,0)],
        [S(-5,33), S(15,28), S(36,22), S(16,-219), S(100,124), S(0,0)],
    ],
    [
        [S(8,89), S(112,185), S(158,190), S(176,181), S(546,1335), S(0,0)],
        [S(-3,15), S(25,6), S(31,11), S(26,-249), S(270,733), S(0,0)],
    ],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[[ScorePair; 6]; 2]; 2] = [
    [
        [S(16,-8), S(38,29), S(54,80), S(79,9), S(-50,-181), S(0,0)],
        [S(-5,14), S(1,-5), S(16,5), S(8,-19), S(-105,-142), S(0,0)],
    ],
    [
        [S(23,78), S(128,48), S(183,127), S(295,221), S(462,800), S(0,0)],
        [S(-1,-3), S(-5,-6), S(-12,51), S(-2,5), S(-85,-191), S(0,0)],
    ],
];
#[rustfmt::skip]
const PUSH_THREAT: [ScorePair; 2] = [S(14,-3), S(26,-6)];
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

    fn our_passer_dist(dist: i32) -> Self::ScorePairType {
        OUR_PASSER_DIST[dist as usize]
    }

    fn their_passer_dist(dist: i32) -> Self::ScorePairType {
        THEIR_PASSER_DIST[dist as usize]
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

    fn push_threat(stm: bool) -> Self::ScorePairType {
        PUSH_THREAT[stm as usize]
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
    let third_rank = if color == Color::White {
        Bitboard::RANK_3
    } else {
        Bitboard::RANK_6
    };

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

    let non_pawns = board.colors(!color) & !board.pieces(PieceType::Pawn);
    let mut pushes = attacks::pawn_pushes_bb(
        color,
        board.colored_pieces(Piece::new(color, PieceType::Pawn)),
    ) & !board.occ();
    pushes |= attacks::pawn_pushes_bb(color, pushes & third_rank) & !board.occ();

    let push_threats = attacks::pawn_attacks_bb(color, pushes) & non_pawns;
    eval += Params::push_threat(stm) * push_threats.popcount() as i32;

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
            let our_passer_dist = Square::chebyshev(board.king_sq(color), sq);
            let their_passer_dist = Square::chebyshev(board.king_sq(!color), sq);
            eval += Params::our_passer_dist(our_passer_dist)
                + Params::their_passer_dist(their_passer_dist);
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
