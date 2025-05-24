use std::ops;

use crate::{
    chess::{attacks, Board},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

#[derive(Clone, Copy)]
struct ScorePair(i32);

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

#[allow(non_snake_case)]
const fn S(mg: i32, eg: i32) -> ScorePair {
    ScorePair::new(mg, eg)
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [
    S(68, 107), S(272, 393), S(281, 351), S(360, 624), S(786, 1190), S(0, 0)
];

#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    // pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S(  48,  187), S(  65,  176), S(  42,  175), S(  73,  123), S(  52,  121), S(  38,  137), S( -26,  178), S( -41,  193),
        S(   4,   40), S(   3,   45), S(  24,    3), S(  29,  -31), S(  35,  -35), S(  65,  -17), S(  46,   23), S(  16,   25),
        S( -16,   26), S(  -7,   21), S(  -7,    0), S(  -6,  -15), S(  12,  -16), S(  14,  -12), S(  11,   10), S(   4,    4),
        S( -20,   10), S( -10,   14), S(  -8,   -6), S(   4,  -11), S(   5,  -11), S(   6,   -8), S(   6,    5), S(  -5,   -6),
        S( -21,    6), S(  -6,    9), S( -11,   -6), S(  -5,   -2), S(  10,   -4), S(  -1,   -5), S(  27,    1), S(   1,   -9),
        S( -15,    8), S(  -3,   12), S(  -7,   -4), S(  -5,   -1), S(   7,    5), S(  25,   -3), S(  39,   -1), S(  -1,  -10),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
    // knight
    [
        S( -97,  -33), S(-102,   -7), S( -56,    3), S( -20,   -6), S(   8,   -3), S( -37,  -25), S( -76,   -4), S( -55,  -53),
        S(  -6,   -4), S(   1,    3), S(  24,   -2), S(  37,   -4), S(  27,  -10), S(  78,  -24), S(   9,   -2), S(  35,  -20),
        S(   0,   -5), S(  21,   -2), S(  24,   11), S(  39,   12), S(  69,   -1), S(  80,  -11), S(  45,  -11), S(  34,  -14),
        S(   5,    9), S(   7,   12), S(  20,   21), S(  47,   21), S(  31,   20), S(  45,   18), S(  19,   12), S(  41,    0),
        S(  -5,   15), S(  -3,    6), S(   3,   21), S(  14,   22), S(  17,   26), S(  17,   14), S(  21,    6), S(   9,   10),
        S( -23,   -1), S( -14,   -1), S( -11,    4), S(  -9,   18), S(   5,   15), S(  -8,   -1), S(   7,   -3), S(  -7,    0),
        S( -24,    1), S( -21,    4), S( -15,   -5), S(   1,   -2), S(  -1,   -2), S(  -2,   -6), S(  -4,   -4), S(  -4,   13),
        S( -61,   14), S( -11,   -8), S( -32,   -5), S( -18,   -3), S( -13,    0), S(  -9,   -9), S(  -9,   -3), S( -31,    5),
    ],
    // bishop
    [
        S( -15,    0), S( -53,    9), S( -41,    4), S( -84,   15), S( -65,    9), S( -52,   -2), S( -31,    0), S( -41,   -6),
        S( -12,  -10), S(  -2,   -6), S( -11,   -3), S( -19,   -1), S(   4,  -10), S(  -2,   -7), S(  -3,   -3), S(   2,  -13),
        S(  -4,    7), S(  10,   -3), S(   8,    1), S(  19,   -9), S(  11,   -4), S(  47,   -1), S(  27,   -3), S(  25,    4),
        S( -10,    4), S(   3,    6), S(  10,    1), S(  26,   14), S(  20,    5), S(  16,    7), S(   7,    3), S(  -7,    6),
        S(  -3,   -1), S( -10,    6), S(  -1,   11), S(  18,    9), S(  15,    8), S(  -2,    5), S(  -7,    6), S(  15,  -11),
        S(   4,    1), S(   9,    5), S(   3,    6), S(   7,    8), S(   8,   11), S(   5,    6), S(   8,   -5), S(  20,   -4),
        S(  24,    8), S(   7,   -7), S(  16,  -11), S(  -2,    0), S(   3,    1), S(  16,   -7), S(  24,   -2), S(  21,   -6),
        S(  10,   -2), S(  24,    3), S(   7,   -7), S(  -7,   -2), S(  -1,   -5), S(  -2,    4), S(  19,  -11), S(  26,  -13),
    ],
    // rook
    [
        S(   6,   13), S(  -6,   19), S(  -9,   27), S(  -8,   22), S(  13,   14), S(  35,   10), S(  34,   10), S(  48,    5),
        S( -13,   16), S( -12,   24), S(   2,   26), S(  17,   16), S(   7,   17), S(  35,    8), S(  35,    4), S(  59,   -6),
        S( -24,   14), S(   4,   12), S(  -2,   14), S(   1,   11), S(  28,   -1), S(  43,   -8), S(  84,  -14), S(  59,  -18),
        S( -25,   15), S(  -9,   10), S( -13,   17), S(  -9,   12), S(  -1,   -1), S(  10,   -6), S(  26,   -6), S(  22,  -11),
        S( -30,    6), S( -33,    8), S( -27,    7), S( -20,    5), S( -19,    2), S( -21,    1), S(   7,   -8), S(  -3,  -11),
        S( -33,    0), S( -30,   -2), S( -27,   -4), S( -24,   -3), S( -15,   -8), S(  -8,  -15), S(  24,  -31), S(   4,  -27),
        S( -31,   -7), S( -28,   -5), S( -18,   -6), S( -16,   -8), S( -10,  -14), S(  -1,  -19), S(  11,  -25), S( -14,  -18),
        S( -14,   -3), S( -13,   -7), S( -10,   -1), S(  -3,   -8), S(   4,  -15), S(   2,   -8), S(  10,  -18), S(  -8,  -16),
    ],
    // queen
    [
        S( -10,  -12), S( -34,   15), S( -15,   36), S(  11,   26), S(  13,   24), S(  25,   16), S(  64,  -37), S(  24,   -8),
        S(  -3,  -21), S( -32,   12), S( -31,   44), S( -40,   63), S( -35,   81), S(   3,   39), S(  -8,   27), S(  49,   11),
        S(   0,  -15), S(  -9,   -3), S( -14,   31), S(  -7,   41), S(   4,   50), S(  50,   31), S(  57,    0), S(  62,    2),
        S( -14,   -5), S( -13,    7), S( -18,   19), S( -17,   36), S( -17,   53), S(  -1,   43), S(   9,   39), S(  17,   25),
        S(  -8,  -12), S( -19,   11), S( -18,   13), S( -13,   28), S( -14,   29), S( -13,   26), S(   0,   18), S(  10,   14),
        S( -10,  -25), S(  -8,  -11), S( -15,    4), S( -14,    3), S( -11,    7), S(  -5,    2), S(   7,  -13), S(  11,  -21),
        S(   0,  -32), S(  -7,  -31), S(  -1,  -33), S(   2,  -30), S(   0,  -24), S(   8,  -46), S(  16,  -71), S(  34,  -88),
        S(  -7,  -35), S(  -5,  -35), S(  -1,  -38), S(   5,  -27), S(   2,  -40), S( -10,  -37), S(  11,  -58), S(  13,  -64),
    ],
    // king
    [
        S(  68, -106), S(  61,  -57), S(  79,  -43), S( -51,    3), S( -10,  -13), S(  42,  -14), S( 102,  -24), S( 220, -133),
        S( -59,  -13), S(  -6,   15), S( -38,   26), S(  54,   12), S(  10,   28), S(   7,   41), S(  49,   29), S(  28,   -1),
        S( -70,    0), S(  37,   20), S( -23,   38), S( -50,   49), S( -12,   49), S(  64,   40), S(  42,   38), S(   5,    9),
        S( -38,   -9), S( -48,   25), S( -65,   45), S(-110,   57), S(-101,   57), S( -63,   51), S( -57,   40), S( -84,   15),
        S( -38,  -18), S( -45,   13), S( -76,   37), S(-110,   53), S(-104,   52), S( -66,   37), S( -67,   25), S( -97,   10),
        S(   6,  -28), S(  21,   -3), S( -37,   19), S( -52,   31), S( -44,   30), S( -42,   21), S(   2,    2), S( -15,   -9),
        S(  91,  -44), S(  48,  -18), S(  35,   -6), S(   1,    4), S(  -3,    8), S(  16,   -2), S(  62,  -20), S(  69,  -37),
        S(  79,  -81), S( 106,  -62), S(  82,  -42), S(  -8,  -25), S(  49,  -44), S(  15,  -27), S(  85,  -54), S(  84,  -83),
    ],
];

#[rustfmt::skip]
const MOBILITY: [[ScorePair; 28]; 4] = [
	[S(  -3,  -30), S( -37,  -46), S( -16,  -16), S(  -8,    0), S(   2,    8), S(   6,   18), S(  13,   21), S(  21,   26), S(  30,   19), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0)],
	[S( -10,  -46), S( -30,  -60), S( -18,  -32), S( -11,  -13), S(  -3,   -4), S(   2,    7), S(   3,   15), S(   6,   19), S(   6,   22), S(   8,   23), S(  10,   23), S(  14,   18), S(  12,   25), S(  18,    4), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0)],
	[S( -13,  -46), S( -30,  -69), S( -14,  -53), S(  -3,  -31), S(   0,  -17), S(  -2,   -6), S(  -1,    1), S(   2,    7), S(   3,   11), S(   6,   17), S(   3,   27), S(   4,   34), S(   6,   38), S(   9,   39), S(  16,   36), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0)],
	[S(  -2,    6), S( -35,  -72), S( -61, -116), S( -18, -202), S( -23,  -63), S( -15,  -11), S(  -6,  -24), S(  -3,   -5), S(  -2,   12), S(   0,   21), S(   3,   24), S(   6,   27), S(   7,   37), S(  10,   36), S(  11,   42), S(  12,   44), S(  13,   45), S(  16,   46), S(  15,   46), S(  21,   39), S(  26,   29), S(  32,   12), S(  29,   18), S(  37,   -7), S(  36,   -8), S(   7,   -3), S( -12,   -9), S(-118,   16)]
];

const PAWN_PHALANX: [ScorePair; 8] = [S(   0,    0), S(   4,   -2), S(  13,    6), S(  23,   16), S(  49,   56), S( 121,  186), S( -97,  403), S(   0,    0)];
const DEFENDED_PAWN: [ScorePair; 8] = [S(   0,    0), S(   0,    0), S(  17,   10), S(  12,    8), S(  13,   15), S(  26,   36), S( 150,   27), S(   0,    0)];

pub fn piece_attacks(pt: PieceType, sq: Square, occ: Bitboard) -> Bitboard {
    match pt {
        PieceType::Knight => attacks::knight_attacks(sq),
        PieceType::Bishop => attacks::bishop_attacks(sq, occ),
        PieceType::Rook => attacks::rook_attacks(sq, occ),
        PieceType::Queen => attacks::queen_attacks(sq, occ),
        _ => unreachable!(),
    }
}

fn evaluate_piece(board: &Board, pt: PieceType, color: Color) -> ScorePair {
    let mut eval = S(0, 0);

    let opp_pawns = board.colored_pieces(Piece::new(!color, pt));
    let mobility_area = !attacks::pawn_attacks_bb(!color, opp_pawns);

    let mut pieces = board.colored_pieces(Piece::new(color, pt));

    while pieces.any() {
        let sq = pieces.poplsb();

        let attacks = piece_attacks(pt, sq, board.occ());
        let mobility = (attacks & mobility_area).popcount();
        eval += MOBILITY[pt as usize - PieceType::Knight as usize][mobility as usize];
    }
    eval
}

fn evaluate_pawns(board: &Board, color: Color) -> ScorePair {
    let mut eval = S(0, 0);
    let our_pawns = board.colored_pieces(Piece::new(color, PieceType::Pawn));
    
    let mut phalanx = our_pawns & our_pawns.west();
    while phalanx.any() {
        eval += PAWN_PHALANX[phalanx.poplsb().relative_sq(color).rank() as usize];
    }

    let mut defended = our_pawns & attacks::pawn_attacks_bb(color, our_pawns);
    while defended.any() {
        eval += DEFENDED_PAWN[defended.poplsb().relative_sq(color).rank() as usize]
    }
    eval
}

pub fn eval(board: &Board) -> i32 {
    let stm = board.stm();
    let mut eval = S(0, 0);
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
            let sq = stm_bb.poplsb().relative_sq(stm).flip();
            eval += MATERIAL[pt as usize];
            eval += PSQT[pt as usize][sq.value() as usize]
        }

        while nstm_bb.any() {
            let sq = nstm_bb.poplsb().relative_sq(!stm).flip();
            eval -= MATERIAL[pt as usize];
            eval -= PSQT[pt as usize][sq.value() as usize];
        }
    }

    eval += evaluate_piece(board, PieceType::Knight, stm) - evaluate_piece(board, PieceType::Knight, !stm);
    eval += evaluate_piece(board, PieceType::Bishop, stm) - evaluate_piece(board, PieceType::Bishop, !stm);
    eval += evaluate_piece(board, PieceType::Rook, stm) - evaluate_piece(board, PieceType::Rook, !stm);
    eval += evaluate_piece(board, PieceType::Queen, stm) - evaluate_piece(board, PieceType::Queen, !stm);
    eval += evaluate_pawns(board, stm) - evaluate_pawns(board, !stm);

    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount()) as i32;

    (eval.mg() * phase.min(24) + eval.eg() * (24 - phase.min(24))) / 24
}
