use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait PolicyScoreType:
    Debug
    + Default
    + Clone
    + PartialEq
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Mul<f32, Output = Self>
    + Div<f32, Output = Self>
{
}

impl PolicyScoreType for f32 {}

pub trait PolicyValues {
    type Value: PolicyScoreType;

    fn cap_bonus(pt: PieceType) -> Self::Value;
    fn pawn_protected_penalty(pt: PieceType) -> Self::Value;
    fn threat_evasion(threat: PieceType, moving: PieceType) -> Self::Value;
    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value;
    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

#[rustfmt::skip]
const CAP_BONUS: [f32; 5] = [1.582, 2.623, 2.688, 2.700, 3.526];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.339, 2.240, 1.880, 2.951, 3.375];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 5] = [
    [0.328, 2.673, 2.260, 2.342, 2.997],
    [0.250, 0.088, 1.374, 1.897, 2.458],
    [0.290, 0.552, 0.226, 1.797, 2.351],
    [0.245, 0.561, 0.584, 0.477, 2.465],
    [0.052, 0.354, 0.500, 0.598, 0.757],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(134.988, 148.247), S(69.586, 139.257), S(97.743, 131.580), S(99.844, 151.927), S(77.673, 133.317), S(69.106, 117.704), S(43.426, 127.254), S(87.339, 156.636),
        S(38.007, 56.511), S(46.933, 45.135), S(46.069, 48.726), S(69.646, 54.991), S(70.793, 36.626), S(69.884, 27.342), S(57.745, 29.491), S(46.272, 53.765),
        S(-18.470, -27.662), S(31.104, -31.645), S(23.442, -26.007), S(39.226, -15.554), S(48.923, -25.036), S(37.469, -32.976), S(48.426, -43.547), S(-4.191, -28.919),
        S(-37.674, -91.144), S(-19.811, -71.032), S(-1.242, -68.703), S(21.176, -54.667), S(11.326, -56.041), S(6.105, -66.796), S(-3.526, -69.709), S(-25.979, -82.089),
        S(-17.711, -116.252), S(-27.247, -73.506), S(-2.027, -83.981), S(-16.219, -53.747), S(-2.519, -58.589), S(-10.271, -70.121), S(8.385, -76.290), S(2.822, -102.424),
        S(-30.759, -95.074), S(-26.991, -65.775), S(-39.429, -57.485), S(-66.827, -27.448), S(-51.624, -28.703), S(-5.391, -56.083), S(10.606, -65.755), S(-24.521, -82.326),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(25.272, -71.280), S(-1.582, -27.033), S(-50.538, 1.551), S(23.734, -16.546), S(10.379, -9.832), S(-12.944, -21.005), S(-10.239, -21.600), S(29.963, -68.862),
        S(-28.385, -18.090), S(-16.423, -1.778), S(-6.096, 10.493), S(-14.984, 13.634), S(-32.908, 21.295), S(6.100, 4.803), S(-16.292, -8.816), S(-1.941, -35.922),
        S(-15.685, -3.970), S(13.702, 7.703), S(26.705, 18.532), S(17.598, 25.869), S(19.213, 19.963), S(29.604, 13.302), S(35.302, -1.911), S(15.029, -13.835),
        S(9.507, -6.814), S(15.816, 19.654), S(31.898, 23.376), S(42.822, 24.382), S(47.543, 23.411), S(41.973, 26.366), S(26.659, 15.459), S(43.218, -16.530),
        S(-4.903, -5.400), S(15.394, 12.027), S(24.556, 27.874), S(41.615, 20.702), S(46.459, 22.099), S(38.642, 22.266), S(33.971, 11.583), S(10.102, -16.066),
        S(-38.966, -11.384), S(5.590, 2.034), S(32.690, 3.427), S(34.783, 16.963), S(43.382, 16.267), S(39.176, 1.695), S(33.576, -4.284), S(-26.623, -11.707),
        S(-54.736, -22.762), S(-32.660, -9.183), S(-0.382, 0.700), S(-3.281, 7.556), S(6.129, 3.380), S(18.890, -2.150), S(0.507, -16.139), S(-29.459, -22.510),
        S(-93.370, -11.188), S(-58.797, -49.113), S(-54.777, -10.484), S(-33.593, -15.484), S(-32.169, -18.485), S(-15.948, -23.198), S(-55.580, -38.104), S(-61.010, -35.983),
    ],
    [
        S(6.562, -23.056), S(-61.928, 7.213), S(-2.471, -14.979), S(-8.849, -8.288), S(-21.450, -7.893), S(2.488, -19.167), S(-66.838, 0.476), S(7.692, -23.976),
        S(-38.743, -15.150), S(-11.735, -4.018), S(-21.262, 1.145), S(-73.164, 8.549), S(-54.590, 4.709), S(-25.862, -0.344), S(-22.398, -0.907), S(-8.466, -28.608),
        S(-24.376, -1.938), S(2.771, 0.169), S(-16.013, 12.989), S(8.641, 4.927), S(11.892, 5.072), S(-2.418, 13.005), S(23.544, -3.169), S(-6.925, -4.691),
        S(-11.234, -6.446), S(2.811, 10.664), S(14.217, 9.364), S(16.574, 17.887), S(18.968, 15.993), S(19.078, 10.553), S(9.879, 5.291), S(10.616, -15.814),
        S(-4.169, -10.240), S(-0.416, 10.028), S(6.337, 19.616), S(31.751, 14.004), S(29.938, 11.883), S(12.795, 15.990), S(7.476, 8.073), S(12.208, -22.295),
        S(5.115, -12.662), S(18.936, 0.137), S(22.842, 10.325), S(15.037, 10.995), S(14.024, 19.151), S(19.310, 9.394), S(24.620, -6.024), S(8.650, -12.482),
        S(-8.882, -3.679), S(21.535, -12.284), S(18.010, -7.153), S(-6.392, 4.574), S(6.060, 3.719), S(27.269, -5.465), S(38.473, -12.155), S(2.299, -22.160),
        S(-25.431, -14.630), S(-17.027, -5.078), S(-16.402, -23.292), S(-26.632, -7.108), S(-19.493, -7.431), S(-20.798, -11.119), S(-17.878, -12.614), S(-31.579, -23.348),
    ],
    [
        S(25.592, 13.490), S(3.985, 16.853), S(-18.353, 26.105), S(-13.817, 19.773), S(-15.238, 16.476), S(-25.094, 16.586), S(6.998, 10.665), S(50.547, 1.660),
        S(0.136, 19.993), S(1.351, 19.482), S(13.216, 14.764), S(8.590, 9.122), S(1.456, 5.972), S(-3.577, 6.520), S(0.464, 7.859), S(22.226, 4.724),
        S(-11.196, 15.136), S(-6.283, 11.804), S(-0.850, 4.957), S(7.980, -4.041), S(14.115, -12.919), S(2.449, -8.481), S(25.343, -7.796), S(22.188, -5.788),
        S(-18.678, 12.227), S(-8.663, 6.589), S(4.690, 1.901), S(10.658, -4.036), S(13.589, -15.245), S(9.629, -13.851), S(17.187, -12.158), S(21.430, -10.548),
        S(-21.718, 8.353), S(-20.468, 5.515), S(0.889, -1.621), S(8.536, -5.014), S(13.807, -12.066), S(2.535, -10.048), S(15.484, -17.187), S(5.204, -10.732),
        S(-28.982, 6.839), S(-12.021, -2.216), S(-2.852, -5.400), S(1.169, -8.091), S(11.852, -12.846), S(7.296, -15.196), S(35.537, -28.807), S(19.311, -22.838),
        S(-45.758, 5.439), S(-27.625, -0.060), S(-7.277, -3.629), S(-3.446, -4.529), S(0.913, -12.933), S(3.649, -16.051), S(20.255, -24.839), S(-19.314, -10.281),
        S(-0.163, -10.753), S(-13.767, 0.155), S(1.499, 0.510), S(15.458, -5.010), S(15.234, -12.583), S(8.890, -16.591), S(18.981, -19.317), S(8.935, -21.409),
    ],
    [
        S(-7.649, 0.240), S(-27.594, 16.449), S(-34.485, 25.674), S(-55.460, 35.070), S(-37.353, 28.049), S(-51.868, 30.087), S(-10.911, 7.256), S(13.707, -2.586),
        S(-22.004, -0.050), S(-16.460, 12.153), S(-15.881, 17.137), S(-48.775, 37.297), S(-49.125, 46.064), S(-32.402, 25.210), S(-29.879, 20.676), S(6.437, 0.230),
        S(-10.101, -7.632), S(4.756, -4.816), S(-9.283, 12.998), S(4.257, 6.791), S(1.736, 16.181), S(8.110, 14.681), S(29.071, -5.907), S(17.747, -10.836),
        S(-14.299, -0.629), S(-0.688, 5.080), S(-0.809, 8.028), S(6.471, 9.975), S(13.542, 14.568), S(7.766, 15.787), S(22.112, 6.895), S(19.958, -8.890),
        S(-4.350, -7.092), S(-3.013, 5.345), S(7.791, 2.508), S(20.397, 4.295), S(18.298, 5.649), S(18.235, 2.040), S(13.079, 4.213), S(18.347, -10.904),
        S(-9.144, -4.024), S(11.209, -8.155), S(19.135, -7.871), S(12.115, 0.853), S(18.519, 2.214), S(23.164, -2.846), S(36.501, -17.963), S(19.293, -18.552),
        S(-22.780, -3.068), S(5.413, -14.400), S(18.599, -23.765), S(12.116, -5.137), S(15.059, -9.294), S(26.843, -19.584), S(26.041, -15.533), S(13.484, -30.181),
        S(-18.784, -4.417), S(-39.816, 4.229), S(-14.888, -12.179), S(18.602, -72.965), S(-2.123, -21.693), S(-18.426, -6.916), S(-11.833, -7.942), S(-28.790, -16.902),
    ],
    [
        S(32.143, -23.012), S(29.803, -12.888), S(26.232, -8.094), S(-4.787, -0.384), S(-8.825, -0.194), S(38.151, 0.380), S(51.339, -4.171), S(59.972, -22.561),
        S(13.699, -11.102), S(16.987, 15.072), S(5.577, 15.862), S(-4.917, 18.533), S(-14.746, 26.400), S(10.982, 30.475), S(50.389, 19.377), S(34.670, 3.021),
        S(-26.830, 3.018), S(-0.683, 22.742), S(-33.316, 33.157), S(-58.169, 46.123), S(-49.722, 51.906), S(5.177, 46.211), S(22.031, 35.831), S(-20.386, 15.551),
        S(-48.446, 0.961), S(-50.359, 26.394), S(-68.164, 40.815), S(-108.964, 56.659), S(-99.344, 57.068), S(-85.752, 53.951), S(-63.152, 39.116), S(-85.917, 17.839),
        S(-52.433, -4.493), S(-47.444, 15.324), S(-77.309, 34.495), S(-100.880, 48.740), S(-107.804, 50.966), S(-84.978, 40.651), S(-68.052, 25.002), S(-82.843, 5.172),
        S(-13.372, -19.943), S(-1.880, 0.374), S(-43.707, 17.628), S(-56.289, 27.311), S(-59.453, 30.228), S(-54.588, 24.944), S(-23.842, 6.269), S(-20.590, -16.971),
        S(35.572, -38.377), S(14.222, -15.281), S(-1.515, -4.015), S(-30.619, 4.219), S(-29.434, 6.678), S(-8.294, -2.759), S(21.969, -18.426), S(14.456, -34.492),
        S(-10.478, -66.162), S(60.571, -56.735), S(37.933, -42.859), S(-14.809, -38.444), S(52.519, -57.587), S(1.891, -44.877), S(46.863, -53.934), S(7.634, -72.790),
    ],
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.915, 0.763, 0.660, 0.350, 0.633],
    [0.099, -0.021, 0.719, 0.666, 0.735],
    [0.059, 0.627, -0.053, 0.629, 0.519],
    [0.259, 0.541, 0.515, 0.041, 1.065],
    [0.082, 0.283, 0.141, 0.133, -0.049],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.239, -1.630];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.944;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.684;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        CAP_BONUS[pt as usize]
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        PAWN_PROTECTED_PENALTY[pt as usize]
    }

    fn threat_evasion(threat: PieceType, moving: PieceType) -> Self::Value {
        THREAT_EVASION[threat as usize][moving as usize]
    }

    fn psqt_score(c: Color, pt: PieceType, sq: Square, phase: i32) -> Self::Value {
        (PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].0 * phase as f32
            + PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].1 * (24 - phase) as f32)
            / 24.0
    }

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[moving as usize - PieceType::Pawn as usize][threatened as usize]
    }

    fn promo_bonus(pt: PieceType) -> Self::Value {
        match pt {
            PieceType::Queen => PROMO_BONUS[0],
            _ => PROMO_BONUS[1],
        }
    }

    fn bad_see_penalty() -> Self::Value {
        BAD_SEE_PENALTY
    }

    fn check_bonus() -> Self::Value {
        CHECK_BONUS
    }
}

#[derive(Debug, Clone)]
pub struct PolicyData {
    attacked: Bitboard,
    attacked_by: [Bitboard; 6],
}

impl PolicyData {
    pub fn new(board: &Board) -> Self {
        let mut result: PolicyData = Self {
            attacked: Bitboard::NONE,
            attacked_by: [Bitboard::NONE; 6],
        };

        let stm = board.stm();

        result.add_attacks(
            PieceType::Pawn,
            attacks::pawn_attacks_bb(
                !stm,
                board.colored_pieces(Piece::new(!stm, PieceType::Pawn)),
            ),
        );

        result.add_attacks(PieceType::King, attacks::king_attacks(board.king_sq(!stm)));

        for pt in [
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            let mut bb = board.colored_pieces(Piece::new(!stm, pt));
            while bb.any() {
                let sq = bb.poplsb();
                let attacks = attacks::piece_attacks(pt, sq, board.occ());
                result.add_attacks(pt, attacks);
            }
        }

        result
    }

    fn add_attacks(&mut self, pt: PieceType, attacks: Bitboard) {
        self.attacked |= attacks;
        self.attacked_by[pt as usize] |= attacks;
    }

    fn attacked(&self) -> Bitboard {
        self.attacked
    }

    fn attacked_by(&self, pt: PieceType) -> Bitboard {
        self.attacked_by[pt as usize]
    }
}

pub fn get_policy(board: &Board, mv: Move, data: &PolicyData) -> f32 {
    get_policy_impl::<PolicyParams>(board, mv, data)
}

pub fn get_policy_impl<Params: PolicyValues>(
    board: &Board,
    mv: Move,
    data: &PolicyData,
) -> Params::Value {
    let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let pawn_protected = data.attacked_by(PieceType::Pawn);
    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let captured_piece = board.piece_at(mv.to_sq());
    let cap_bonus = if let Some(captured) = captured_piece {
        Params::cap_bonus(captured.piece_type())
    } else {
        Params::Value::default()
    };

    let pawn_protected_penalty = if pawn_protected.has(mv.to_sq()) {
        Params::pawn_protected_penalty(moving_piece.piece_type())
    } else {
        Params::Value::default()
    };

    let mut threat_evasion = Params::Value::default();
    if moving_piece.piece_type() != PieceType::King && data.attacked().has(mv.from_sq()) {
        for pt in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ] {
            let from_threat = data.attacked_by(pt).has(mv.from_sq());
            let to_threat = data.attacked_by(pt).has(mv.to_sq());
            if from_threat && !to_threat {
                threat_evasion += Params::threat_evasion(pt, moving_piece.piece_type());
            }

            if from_threat || to_threat {
                break;
            }
        }
    }

    let moving_piece = board.piece_at(mv.from_sq()).unwrap();
    let phase = (4 * board.pieces(PieceType::Queen).popcount()
        + 2 * board.pieces(PieceType::Rook).popcount()
        + board.pieces(PieceType::Bishop).popcount()
        + board.pieces(PieceType::Knight).popcount())
    .min(24) as i32;
    let psqt = if mv.kind() != MoveKind::Promotion {
        Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.to_sq(), phase)
            - Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.from_sq(), phase)
    } else {
        Params::Value::default()
    };

    let threat_score =
        if mv.kind() == MoveKind::None && moving_piece.piece_type() != PieceType::King {
            let occ_after = board.occ()
                | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
            let attacks_after = if moving_piece.piece_type() != PieceType::Pawn {
                attacks::piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after)
            } else {
                attacks::pawn_attacks(board.stm(), mv.to_sq())
            };

            let mut threats =
                attacks_after & board.colors(!board.stm()) & !board.pieces(PieceType::King);
            let mut score = Params::Value::default();
            while threats.any() {
                let threat = threats.poplsb();
                score += Params::threat(
                    moving_piece.piece_type(),
                    board.piece_at(threat).unwrap().piece_type(),
                );
            }

            score
        } else {
            Params::Value::default()
        };

    let promo_bonus = if mv.kind() == MoveKind::Promotion {
        Params::promo_bonus(mv.promo_piece())
    } else {
        Params::Value::default()
    };

    let bad_see_penalty = if !see::see(board, mv, 0) && !pawn_protected.has(mv.to_sq()) {
        Params::bad_see_penalty()
    } else {
        Params::Value::default()
    };

    let check_bonus = if board.gives_direct_check(mv) {
        Params::check_bonus()
    } else {
        Params::Value::default()
    };

    cap_bonus + promo_bonus + threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
