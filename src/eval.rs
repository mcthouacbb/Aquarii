use crate::{
    chess::Board,
    types::{Piece, PieceType},
};

#[derive(Clone, Copy)]
struct ScorePair {
    mg: i16,
    eg: i16,
}

#[allow(non_snake_case)]
const fn S(mg: i16, eg: i16) -> ScorePair {
    ScorePair { mg: mg, eg: eg }
}

#[rustfmt::skip]
const MATERIAL: [ScorePair; 6] = [
    S(63, 119), S(267, 337), S(301, 360), S(381, 631), S(769, 1197), S(0, 0)
];

#[rustfmt::skip]
const PSQT: [[ScorePair; 64]; 6] = [
    // pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S(  73,  163), S(  95,  155), S(  73,  156), S( 102,  107), S(  86,  103), S(  69,  114), S(   3,  159), S( -22,  173),
        S(  -5,  104), S(   9,  112), S(  41,   78), S(  47,   57), S(  50,   49), S(  71,   34), S(  51,   81), S(   9,   79),
        S( -20,   36), S(   4,   25), S(   8,    5), S(  10,   -3), S(  31,  -12), S(  22,   -9), S(  26,   10), S(   3,   10),
        S( -30,   11), S(  -3,    8), S(  -4,   -9), S(  12,  -12), S(  12,  -14), S(   4,  -12), S(  13,   -1), S(  -9,   -8),
        S( -32,    5), S(  -7,    7), S(  -7,  -10), S(  -6,    2), S(   8,   -6), S(  -3,   -8), S(  27,   -3), S(  -2,  -12),
        S( -31,   10), S(  -7,   11), S( -11,   -3), S( -21,    3), S(  -1,    8), S(  13,   -3), S(  36,   -4), S( -10,  -11),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
    // knight
    [
        S(-142,  -75), S(-110,  -15), S( -46,   -1), S( -14,   -9), S(  17,   -6), S( -39,  -29), S( -92,   -8), S( -87,  -97),
        S( -11,  -22), S(   6,   -1), S(  31,    7), S(  48,    6), S(  32,   -1), S(  93,  -16), S(   5,   -5), S(  28,  -37),
        S(   4,   -6), S(  37,    9), S(  54,   25), S(  66,   26), S( 102,   11), S( 103,    6), S(  60,   -1), S(  29,  -16),
        S(   0,    5), S(  13,   26), S(  37,   37), S(  57,   39), S(  40,   40), S(  64,   34), S(  23,   25), S(  33,   -2),
        S( -13,    6), S(   2,   15), S(  16,   39), S(  17,   39), S(  26,   42), S(  21,   32), S(  20,   18), S(  -3,   -2),
        S( -32,   -9), S( -10,    9), S(   4,   19), S(   7,   32), S(  18,   31), S(   8,   15), S(  11,    4), S( -16,   -7),
        S( -45,  -18), S( -33,   -2), S( -17,    6), S(  -5,   10), S(  -4,    9), S(  -2,    4), S( -15,  -11), S( -18,   -8),
        S( -87,  -25), S( -34,  -38), S( -47,   -9), S( -33,   -6), S( -29,   -5), S( -16,  -15), S( -32,  -31), S( -58,  -37),
    ],
    // bishop
    [
        S( -26,   -9), S( -44,    2), S( -33,   -1), S( -74,   13), S( -63,    6), S( -45,   -3), S( -16,  -10), S( -53,  -12),
        S(  -9,  -22), S(  14,   -3), S(   8,    1), S(  -9,    4), S(  19,   -5), S(  19,   -7), S(  11,    1), S(   0,  -24),
        S(   0,    7), S(  23,    1), S(  24,   12), S(  47,    1), S(  33,    6), S(  64,    7), S(  41,    0), S(  29,    0),
        S(  -8,    2), S(   6,   18), S(  27,   13), S(  37,   26), S(  34,   19), S(  30,   16), S(   6,   15), S(  -8,    2),
        S( -14,   -2), S(  -2,   15), S(   4,   23), S(  24,   19), S(  21,   19), S(   7,   18), S(  -1,   13), S(  -6,  -12),
        S(  -4,   -2), S(   3,    8), S(   3,   16), S(   6,   15), S(   7,   19), S(   2,   16), S(   4,   -1), S(   8,  -12),
        S(  -2,   -8), S(  -1,   -7), S(  10,   -9), S( -11,    6), S(  -4,    8), S(   9,   -4), S(  15,   -2), S(   2,  -27),
        S( -23,  -24), S(  -4,   -7), S( -19,  -26), S( -28,   -5), S( -23,   -8), S( -24,   -8), S(   0,  -22), S( -13,  -37),
    ],
    // rook
    [
        S(  29,    9), S(  21,   16), S(  28,   25), S(  33,   21), S(  51,   12), S(  67,    2), S(  50,    4), S(  69,   -1),
        S(  11,    9), S(  10,   21), S(  29,   25), S(  49,   16), S(  35,   16), S(  63,    2), S(  50,   -2), S(  80,  -15),
        S(  -9,    9), S(  11,   12), S(  13,   14), S(  16,   12), S(  44,   -1), S(  45,   -7), S(  82,  -16), S(  60,  -20),
        S( -25,   11), S( -12,   10), S(  -9,   19), S(  -1,   15), S(   5,    0), S(   5,   -5), S(  13,   -9), S(  16,  -15),
        S( -43,    4), S( -41,    9), S( -31,   11), S( -19,   10), S( -19,    6), S( -34,    4), S( -11,   -9), S( -19,  -14),
        S( -50,    0), S( -41,    0), S( -33,   -1), S( -33,    4), S( -28,    0), S( -30,   -8), S(   3,  -28), S( -18,  -27),
        S( -53,   -5), S( -41,   -1), S( -26,   -1), S( -30,    1), S( -25,   -7), S( -24,  -11), S(  -7,  -20), S( -36,  -15),
        S( -34,  -10), S( -33,    0), S( -24,    7), S( -18,    6), S( -14,   -2), S( -24,   -7), S( -10,  -11), S( -33,  -18),
    ],
    // queen
    [
        S( -36,    2), S( -28,   15), S(   2,   32), S(  35,   18), S(  35,   15), S(  40,    8), S(  58,  -35), S(   5,   -4),
        S(   1,  -33), S( -21,    9), S( -14,   43), S( -21,   60), S( -16,   78), S(  21,   37), S(   1,   21), S(  44,   -3),
        S(   1,  -22), S(  -1,   -5), S(  -3,   37), S(  13,   38), S(  18,   52), S(  59,   32), S(  60,   -4), S(  57,  -17),
        S( -15,  -12), S( -11,   11), S(  -7,   25), S(  -8,   49), S(  -6,   61), S(   7,   47), S(   6,   33), S(  13,   12),
        S( -13,  -15), S( -15,   14), S( -17,   23), S(  -8,   42), S(  -9,   41), S( -10,   32), S(   1,   12), S(   4,   -1),
        S( -16,  -26), S(  -9,  -10), S( -14,   13), S( -15,   11), S( -12,   15), S(  -5,    6), S(   7,  -16), S(   1,  -28),
        S( -18,  -31), S( -13,  -27), S(  -2,  -30), S(  -3,  -20), S(  -5,  -17), S(   4,  -43), S(  10,  -71), S(  21, -101),
        S( -20,  -38), S( -30,  -30), S( -23,  -26), S(  -8,  -35), S( -16,  -31), S( -29,  -32), S(  -7,  -62), S( -14,  -62),
    ],
    // king
    [
        S(  64, -103), S(  40,  -53), S(  73,  -44), S( -68,    6), S( -12,  -14), S(  38,  -11), S(  87,  -19), S( 194, -126),
        S( -53,  -11), S( -14,   18), S( -57,   31), S(  50,   12), S(  -3,   33), S(   3,   45), S(  42,   34), S(  21,    4),
        S( -74,    5), S(  28,   23), S( -39,   42), S( -58,   53), S( -17,   52), S(  58,   44), S(  38,   43), S(   2,   14),
        S( -42,   -5), S( -52,   29), S( -68,   46), S(-112,   59), S( -99,   58), S( -62,   53), S( -62,   44), S( -85,   19),
        S( -36,  -17), S( -45,   14), S( -75,   37), S(-102,   52), S( -99,   51), S( -63,   38), S( -67,   27), S( -90,   10),
        S(   8,  -27), S(  23,   -4), S( -33,   17), S( -45,   29), S( -39,   28), S( -37,   20), S(   9,    0), S(  -8,  -12),
        S(  95,  -49), S(  55,  -22), S(  41,   -9), S(   7,    1), S(   6,    5), S(  24,   -5), S(  71,  -23), S(  80,  -41),
        S(  91,  -83), S( 114,  -64), S(  88,  -45), S( -11,  -27), S(  53,  -52), S(  14,  -29), S(  95,  -55), S(  96,  -83),
    ],
];

pub fn eval(board: &Board) -> i32 {
    let stm = board.stm();
    let mut eval_mg = 0;
    let mut eval_eg = 0;
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
            eval_mg += MATERIAL[pt as usize].mg as i32;
            eval_eg += MATERIAL[pt as usize].eg as i32;

            eval_mg += PSQT[pt as usize][sq.value() as usize].mg as i32;
            eval_eg += PSQT[pt as usize][sq.value() as usize].eg as i32;
        }

        while nstm_bb.any() {
            let sq = nstm_bb.poplsb().relative_sq(!stm).flip();
            eval_mg -= MATERIAL[pt as usize].mg as i32;
            eval_eg -= MATERIAL[pt as usize].eg as i32;

            eval_mg -= PSQT[pt as usize][sq.value() as usize].mg as i32;
            eval_eg -= PSQT[pt as usize][sq.value() as usize].eg as i32;
        }
    }

    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount()) as i32;

    (eval_mg * phase.min(24) + eval_eg * (24 - phase.min(24))) / 24
}