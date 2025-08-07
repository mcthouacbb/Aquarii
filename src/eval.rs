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
const MATERIAL: [ScorePair; 6] = [S(146,208), S(637,172), S(770,433), S(1096,644), S(1751,1500), S(0,0)];
#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    [
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
        S(361,-12), S(211,-19), S(-1,152), S(-43,134), S(249,-31), S(64,129), S(-448,293), S(101,137),
        S(-111,24), S(56,98), S(-156,42), S(-115,-50), S(-2,-59), S(-36,-14), S(61,-16), S(-43,-27),
        S(34,-11), S(28,-45), S(1,31), S(26,-80), S(11,-72), S(-96,-41), S(28,-38), S(-54,4),
        S(-22,-29), S(14,-55), S(-15,17), S(28,-74), S(12,-54), S(1,-26), S(18,-36), S(-39,-8),
        S(-29,-7), S(-8,-22), S(8,19), S(-54,5), S(21,-59), S(-17,-53), S(46,-72), S(-6,-10),
        S(-49,34), S(15,-9), S(12,55), S(-64,10), S(-13,-52), S(22,-43), S(14,-24), S(-21,-37),
        S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0),
    ],
    [
        S(138,-532), S(-350,23), S(-568,252), S(-277,101), S(-237,181), S(-514,112), S(564,85), S(-557,60),
        S(-225,204), S(25,2), S(164,-68), S(18,-86), S(78,-119), S(139,-99), S(33,-131), S(-261,103),
        S(8,34), S(-37,-63), S(-56,20), S(103,-27), S(21,2), S(54,106), S(28,1), S(5,-59),
        S(-37,-65), S(26,21), S(30,0), S(82,-38), S(118,-57), S(17,-61), S(61,10), S(188,-125),
        S(21,-64), S(128,-23), S(65,-12), S(2,27), S(54,33), S(59,77), S(173,-21), S(75,-96),
        S(69,-88), S(26,-54), S(40,40), S(-3,35), S(108,-19), S(47,-79), S(82,-45), S(46,33),
        S(183,-98), S(125,-95), S(95,-134), S(42,69), S(77,-26), S(-23,-121), S(7,-47), S(87,179),
        S(-269,-21), S(55,155), S(-130,-43), S(-52,218), S(22,5), S(212,-95), S(33,-127), S(-237,651),
    ],
    [
        S(97,-109), S(-174,145), S(-350,39), S(-291,-119), S(112,-50), S(-431,121), S(-32,148), S(-163,-52),
        S(-53,-8), S(-13,-52), S(-118,5), S(203,-128), S(-40,-12), S(-126,68), S(-70,10), S(-29,55),
        S(116,-140), S(162,-103), S(-23,6), S(-64,62), S(158,49), S(-153,166), S(75,7), S(-44,39),
        S(173,-92), S(89,-27), S(-44,81), S(91,44), S(-70,87), S(-61,113), S(7,60), S(30,-21),
        S(167,-146), S(36,-31), S(25,-4), S(21,113), S(75,103), S(-24,162), S(15,28), S(154,-164),
        S(73,22), S(130,-37), S(23,-22), S(73,23), S(44,6), S(163,-58), S(79,122), S(64,16),
        S(187,50), S(29,36), S(-58,-52), S(65,-40), S(128,-116), S(-32,20), S(115,-9), S(80,-109),
        S(-342,115), S(126,-197), S(44,-28), S(-74,-11), S(-13,-94), S(46,-17), S(-318,-101), S(-65,30),
    ],
    [
        S(160,-8), S(222,-74), S(624,-196), S(568,-198), S(364,-140), S(-41,-10), S(467,-82), S(251,0),
        S(21,27), S(99,-31), S(223,-106), S(448,-136), S(330,-152), S(35,59), S(115,9), S(-1,86),
        S(-23,44), S(-108,50), S(-234,153), S(90,-62), S(51,22), S(85,-12), S(-50,-27), S(-133,68),
        S(-76,46), S(-98,30), S(161,-57), S(-60,-7), S(-27,-36), S(-210,66), S(-53,1), S(22,-50),
        S(-153,45), S(-274,42), S(-130,22), S(-80,114), S(87,-114), S(-46,-16), S(-90,-50), S(-52,-12),
        S(-111,59), S(-109,14), S(-155,23), S(-171,58), S(-151,17), S(-103,38), S(-46,47), S(-169,25),
        S(-132,127), S(48,-134), S(40,-8), S(-141,107), S(-177,2), S(-162,-34), S(-43,-59), S(-137,47),
        S(-144,88), S(-102,44), S(-81,55), S(-59,47), S(-85,43), S(-45,26), S(-146,38), S(-103,24),
    ],
    [
        S(345,-250), S(49,-75), S(-422,380), S(-693,506), S(252,61), S(258,-135), S(400,-297), S(179,-280),
        S(2,177), S(-21,54), S(25,203), S(-22,255), S(144,-52), S(-1,166), S(14,-97), S(121,-236),
        S(54,26), S(-18,1), S(-107,121), S(19,135), S(5,220), S(-25,7), S(-200,248), S(-43,74),
        S(-144,90), S(22,222), S(-54,127), S(-69,344), S(-73,146), S(-105,227), S(-86,14), S(-4,-31),
        S(-80,163), S(-208,134), S(-98,132), S(-61,-17), S(-67,223), S(-126,223), S(-24,30), S(24,-155),
        S(80,-130), S(39,-60), S(-44,-180), S(-111,285), S(-109,59), S(-60,211), S(-81,187), S(89,65),
        S(-25,-336), S(7,-19), S(38,-95), S(-25,-25), S(-17,138), S(112,-298), S(56,-331), S(100,-200),
        S(-87,-182), S(44,-324), S(60,-324), S(8,-106), S(81,-373), S(68,-237), S(144,-528), S(470,-285),
    ],
    [
        S(259,-251), S(842,-278), S(-416,-252), S(397,-70), S(-899,278), S(-1001,302), S(-384,42), S(-487,-61),
        S(195,-89), S(520,20), S(-100,108), S(-532,77), S(-508,238), S(-415,156), S(-209,113), S(-839,121),
        S(182,3), S(-371,127), S(-366,176), S(-996,146), S(-295,125), S(-255,131), S(-171,135), S(-480,101),
        S(251,25), S(148,57), S(-213,79), S(519,-46), S(645,-69), S(376,-5), S(239,39), S(-228,98),
        S(51,-94), S(65,25), S(-84,80), S(36,34), S(317,-50), S(171,35), S(156,22), S(38,9),
        S(75,-117), S(215,-44), S(210,-42), S(-3,38), S(2,29), S(79,12), S(174,-75), S(238,-53),
        S(436,-149), S(62,2), S(159,-51), S(94,-41), S(169,-45), S(131,-18), S(229,-77), S(305,-147),
        S(-41,-39), S(240,-120), S(159,-97), S(156,-187), S(160,-45), S(131,-103), S(249,-77), S(213,-190),
    ],
];
#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
    [S(298,-1729), S(-68,84), S(-102,222), S(-66,204), S(-33,229), S(-33,255), S(-21,271), S(9,271), S(16,195), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-111,421), S(-111,156), S(-102,-79), S(-75,-45), S(-67,1), S(-61,47), S(-59,50), S(-47,46), S(-43,55), S(18,-51), S(86,-18), S(186,-207), S(190,-112), S(197,-264), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(208,-217), S(-159,-162), S(-120,-32), S(-117,15), S(-104,26), S(-69,29), S(-73,71), S(-49,65), S(-33,62), S(-36,70), S(25,33), S(-11,76), S(4,76), S(-1,84), S(535,-196), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)],
    [S(-413,-315), S(-413,-315), S(-700,-517), S(-235,19), S(-100,370), S(-201,282), S(-198,-78), S(-207,38), S(-193,103), S(-197,283), S(-192,259), S(-180,282), S(-190,346), S(-182,366), S(-160,326), S(-150,275), S(-127,237), S(-167,292), S(-70,141), S(-54,124), S(-25,155), S(49,3), S(36,60), S(680,-489), S(1153,-717), S(1296,-854), S(1001,-554), S(140,-123)],
];
#[rustfmt::skip]
const PASSED_PAWN: [ScorePair; 8] = [S(0,0), S(22,21), S(-17,63), S(20,77), S(-58,167), S(97,185), S(90,226), S(0,0)];
#[rustfmt::skip]
const PAWN_PHALANX: [ScorePair; 8] = [S(0,0), S(20,54), S(28,75), S(52,70), S(60,132), S(347,283), S(452,465), S(0,0)];
#[rustfmt::skip]
const DEFENDED_PAWN: [ScorePair; 8] = [S(0,0), S(0,0), S(35,32), S(25,45), S(32,45), S(208,59), S(1166,-202), S(0,0)];
#[rustfmt::skip]
const SAFE_KNIGHT_CHECK: ScorePair = S(41,-12);
#[rustfmt::skip]
const SAFE_BISHOP_CHECK: ScorePair = S(38,33);
#[rustfmt::skip]
const SAFE_ROOK_CHECK: ScorePair = S(190,-48);
#[rustfmt::skip]
const SAFE_QUEEN_CHECK: ScorePair = S(51,-3);
#[rustfmt::skip]
const KING_ATTACKER_WEIGHT: [ScorePair; 4] = [S(27,21), S(19,27), S(20,41), S(0,129)];
#[rustfmt::skip]
const KING_ATTACKS: [ScorePair; 14] = [S(-69,80), S(-64,-12), S(-58,-3), S(-54,10), S(-23,10), S(23,-24), S(79,-29), S(166,-78), S(284,-129), S(255,-141), S(463,-291), S(197,-240), S(616,34), S(13,-183)];
#[rustfmt::skip]
const THREAT_BY_PAWN: [ScorePair; 6] = [S(0,0), S(0,0), S(0,0), S(0,0), S(0,0), S(0,0)];
#[rustfmt::skip]
const THREAT_BY_KNIGHT: [[ScorePair; 6]; 2] = [
    [S(0,67), S(46,0), S(169,125), S(94,216), S(59,1210), S(291,40)],
    [S(-1,43), S(6,81), S(99,93), S(73,170), S(-121,1033), S(-40,191)],
];
#[rustfmt::skip]
const THREAT_BY_BISHOP: [[ScorePair; 6]; 2] = [
    [S(-14,55), S(53,48), S(44,32), S(174,234), S(121,1014), S(247,-35)],
    [S(-16,16), S(43,43), S(-44,-32), S(92,278), S(-9,947), S(-59,168)],
];
#[rustfmt::skip]
const THREAT_BY_ROOK: [[ScorePair; 6]; 2] = [
    [S(-23,125), S(42,107), S(178,6), S(-104,-40), S(149,380), S(47,49)],
    [S(-27,53), S(30,26), S(86,-11), S(-187,-66), S(138,1096), S(-22,135)],
];
#[rustfmt::skip]
const THREAT_BY_QUEEN: [[ScorePair; 6]; 2] = [
    [S(10,53), S(55,46), S(163,-178), S(87,-7), S(9,53), S(182,24)],
    [S(-2,-1), S(15,83), S(-3,48), S(-59,91), S(-15,-58), S(126,96)],
];
#[rustfmt::skip]
const TEMPO: i32 = 60;

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
