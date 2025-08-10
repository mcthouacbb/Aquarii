use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind},
    types::{Bitboard, Color, Piece, PieceType, Square},
};

// heavily inspired by Motors tuner
pub trait PolicyValueType:
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

impl PolicyValueType for f32 {}

pub trait PolicyValues {
    type Value: PolicyValueType;

    fn cap_bonus(pt: PieceType) -> Self::Value;
    fn pawn_protected_penalty(pt: PieceType) -> Self::Value;
    fn pawn_threat_evasion(pt: PieceType) -> Self::Value;
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
const CAP_BONUS: [f32; 5] = [1.600, 2.524, 2.755, 2.757, 3.398];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.325, 2.296, 1.939, 3.109, 3.428];
#[rustfmt::skip]
const PAWN_THREAT_EVASION: [f32; 5] = [0.232, 2.465, 2.186, 2.223, 2.692];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(122.763, 148.464), S(59.300, 139.267), S(84.348, 132.548), S(85.857, 152.938), S(64.127, 134.113), S(54.133, 118.460), S(32.991, 126.930), S(78.808, 155.814),
        S(31.486, 54.747), S(39.871, 44.055), S(38.763, 47.839), S(61.212, 54.278), S(61.743, 36.496), S(62.195, 26.767), S(50.641, 28.551), S(40.183, 52.290),
        S(-20.792, -29.375), S(28.109, -33.373), S(20.007, -27.680), S(34.095, -17.034), S(43.539, -26.330), S(33.579, -34.181), S(45.378, -45.002), S(-5.973, -30.646),
        S(-36.028, -93.322), S(-20.944, -72.813), S(-1.641, -70.937), S(19.384, -58.096), S(9.850, -58.868), S(5.533, -68.610), S(-4.579, -71.714), S(-24.083, -84.629),
        S(-14.942, -118.351), S(-26.565, -75.332), S(-1.304, -86.860), S(-16.352, -57.218), S(-2.707, -61.183), S(-9.291, -72.096), S(9.324, -78.408), S(5.752, -104.145),
        S(-24.299, -97.382), S(-22.308, -67.974), S(-35.358, -59.972), S(-63.544, -30.077), S(-48.387, -31.714), S(-0.953, -58.234), S(15.628, -68.058), S(-18.224, -83.918),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(17.210, -69.335), S(-7.484, -25.948), S(-60.290, 3.101), S(13.464, -14.819), S(2.344, -8.026), S(-21.796, -19.593), S(-16.466, -20.208), S(21.722, -66.623),
        S(-29.039, -17.264), S(-19.778, -1.205), S(-7.587, 10.892), S(-18.647, 13.810), S(-38.056, 22.167), S(3.444, 5.068), S(-21.987, -7.613), S(-3.069, -35.272),
        S(-17.465, -3.002), S(11.082, 8.034), S(23.546, 18.315), S(12.836, 25.563), S(13.577, 20.944), S(25.699, 13.070), S(32.075, -1.407), S(12.816, -13.457),
        S(9.573, -6.283), S(14.452, 18.979), S(29.258, 23.101), S(40.725, 23.063), S(44.935, 22.569), S(37.923, 26.399), S(25.324, 15.007), S(42.647, -15.462),
        S(-5.647, -4.311), S(14.339, 12.271), S(23.232, 27.114), S(39.865, 19.826), S(45.009, 20.619), S(37.400, 21.300), S(32.470, 11.793), S(9.500, -15.148),
        S(-40.068, -10.850), S(5.429, 2.094), S(32.557, 2.048), S(34.492, 15.881), S(42.877, 15.161), S(38.725, 0.560), S(32.997, -4.171), S(-28.208, -10.497),
        S(-53.437, -22.412), S(-31.057, -8.567), S(1.361, -0.215), S(-2.328, 5.918), S(6.657, 3.055), S(20.015, -3.052), S(0.881, -16.133), S(-28.048, -21.848),
        S(-90.803, -10.854), S(-54.680, -50.861), S(-54.125, -9.962), S(-32.717, -15.630), S(-30.693, -17.797), S(-14.854, -22.845), S(-51.688, -39.064), S(-59.703, -35.069),
    ],
    [
        S(3.591, -20.966), S(-63.212, 6.634), S(-3.458, -14.352), S(-12.173, -7.230), S(-24.898, -7.012), S(-1.379, -18.046), S(-69.898, 1.257), S(6.033, -22.787),
        S(-36.244, -14.159), S(-14.582, -3.145), S(-21.409, 1.304), S(-77.178, 9.232), S(-57.543, 5.125), S(-28.160, 0.127), S(-25.169, 0.328), S(-10.310, -27.239),
        S(-24.663, -1.772), S(2.002, 0.662), S(-20.272, 13.441), S(6.054, 4.695), S(7.286, 5.456), S(-5.129, 12.552), S(19.741, -3.058), S(-6.942, -4.215),
        S(-10.113, -5.575), S(1.879, 10.084), S(11.296, 9.426), S(10.268, 17.822), S(14.037, 15.879), S(15.580, 10.063), S(8.956, 4.700), S(7.949, -14.330),
        S(-3.178, -9.924), S(-2.243, 10.499), S(4.758, 18.546), S(28.498, 13.433), S(26.524, 11.414), S(11.790, 15.472), S(4.245, 7.928), S(14.188, -21.531),
        S(5.621, -11.517), S(19.076, -0.227), S(20.984, 9.999), S(13.749, 10.000), S(13.307, 18.024), S(16.756, 9.227), S(24.978, -5.646), S(8.903, -11.923),
        S(-6.386, -3.459), S(22.443, -11.824), S(18.844, -6.978), S(-5.545, 3.984), S(6.345, 2.921), S(28.576, -6.050), S(38.716, -12.777), S(6.181, -21.310),
        S(-23.634, -13.972), S(-14.987, -4.889), S(-12.004, -24.743), S(-25.829, -6.935), S(-16.581, -6.985), S(-16.827, -12.688), S(-14.479, -12.233), S(-31.020, -21.784),
    ],
    [
        S(24.463, 14.155), S(3.756, 17.355), S(-22.428, 27.122), S(-16.387, 20.144), S(-18.592, 16.911), S(-27.090, 16.807), S(6.747, 11.107), S(51.122, 2.206),
        S(0.012, 20.478), S(-0.461, 19.795), S(9.804, 15.432), S(5.078, 9.527), S(-3.007, 6.295), S(-4.844, 6.643), S(-3.087, 8.651), S(22.601, 4.803),
        S(-13.064, 15.523), S(-7.654, 11.578), S(-5.963, 5.241), S(2.185, -3.487), S(7.696, -12.034), S(-2.556, -8.042), S(21.317, -7.342), S(18.808, -5.249),
        S(-19.609, 12.191), S(-12.203, 6.910), S(0.322, 2.051), S(4.758, -3.606), S(6.801, -14.435), S(5.425, -13.720), S(14.617, -12.326), S(20.691, -10.575),
        S(-22.712, 8.690), S(-22.044, 5.336), S(-2.515, -1.717), S(5.041, -4.957), S(11.551, -12.457), S(-1.866, -9.835), S(14.545, -17.879), S(5.112, -10.706),
        S(-29.012, 7.152), S(-12.205, -2.517), S(-3.960, -5.499), S(0.200, -8.841), S(10.999, -13.576), S(7.296, -16.140), S(34.322, -28.973), S(19.270, -22.682),
        S(-44.023, 5.457), S(-26.776, -0.113), S(-7.320, -3.749), S(-2.226, -4.849), S(0.818, -13.251), S(4.989, -16.669), S(21.878, -24.909), S(-16.415, -10.384),
        S(4.512, -11.209), S(-11.493, 0.226), S(2.908, 0.593), S(17.365, -5.348), S(16.906, -12.996), S(11.621, -17.335), S(21.937, -19.419), S(14.779, -22.326),
    ],
    [
        S(-11.831, 0.988), S(-34.272, 18.000), S(-40.764, 27.158), S(-58.696, 34.751), S(-43.039, 28.851), S(-57.116, 30.532), S(-15.673, 8.253), S(13.219, -4.046),
        S(-20.324, -1.291), S(-23.403, 14.425), S(-20.039, 17.279), S(-56.558, 38.952), S(-58.529, 47.867), S(-37.288, 25.603), S(-32.257, 20.883), S(3.331, 1.167),
        S(-10.225, -8.827), S(0.008, -4.158), S(-17.795, 14.807), S(-4.838, 8.631), S(-6.576, 17.881), S(0.168, 15.715), S(21.379, -4.143), S(14.396, -9.834),
        S(-13.102, -2.170), S(-4.937, 5.762), S(-6.581, 9.010), S(-2.595, 11.434), S(6.865, 15.032), S(2.164, 15.864), S(17.274, 7.023), S(18.805, -8.908),
        S(-4.659, -6.537), S(-3.404, 4.712), S(5.184, 2.059), S(15.089, 4.395), S(14.689, 4.664), S(16.223, 1.299), S(10.457, 4.699), S(19.510, -11.734),
        S(-5.306, -5.664), S(11.447, -8.736), S(20.137, -10.119), S(11.943, -1.333), S(18.816, 0.337), S(24.708, -6.186), S(38.085, -20.478), S(22.798, -20.626),
        S(-18.543, -4.569), S(8.994, -15.823), S(21.644, -25.419), S(15.598, -7.776), S(18.720, -11.574), S(30.721, -22.017), S(30.111, -18.613), S(19.457, -33.004),
        S(-9.804, -8.630), S(-35.514, 2.908), S(-9.733, -14.639), S(25.558, -76.654), S(3.123, -24.250), S(-12.322, -9.601), S(-5.864, -10.359), S(-22.259, -18.999),
    ],
    [
        S(27.001, -23.553), S(24.309, -12.901), S(23.803, -8.566), S(-10.144, -0.300), S(-13.171, -0.307), S(32.060, 0.557), S(46.523, -4.247), S(54.933, -22.919),
        S(8.211, -11.154), S(12.579, 15.364), S(-0.280, 16.106), S(-9.481, 18.947), S(-19.048, 26.538), S(6.066, 30.837), S(44.993, 20.000), S(27.597, 3.555),
        S(-32.266, 3.383), S(-5.292, 23.233), S(-37.693, 33.781), S(-62.684, 46.984), S(-54.140, 52.702), S(0.933, 46.736), S(17.175, 36.145), S(-26.374, 16.236),
        S(-53.477, 1.143), S(-54.897, 26.984), S(-73.969, 41.573), S(-112.778, 57.115), S(-103.684, 57.454), S(-89.720, 54.726), S(-67.689, 39.726), S(-91.108, 18.546),
        S(-57.897, -4.180), S(-52.704, 16.411), S(-81.428, 35.321), S(-104.400, 49.305), S(-111.501, 51.734), S(-88.793, 41.733), S(-73.027, 26.020), S(-87.377, 6.055),
        S(-19.690, -19.011), S(-7.162, 1.514), S(-47.616, 18.199), S(-58.723, 27.952), S(-61.578, 30.841), S(-57.795, 25.701), S(-28.322, 7.463), S(-26.914, -15.556),
        S(30.051, -37.682), S(10.364, -14.703), S(-3.924, -3.160), S(-30.062, 4.288), S(-28.893, 6.881), S(-8.369, -2.210), S(18.462, -16.806), S(10.493, -33.123),
        S(-15.748, -66.323), S(58.425, -56.393), S(38.915, -43.034), S(-14.023, -38.743), S(56.303, -57.760), S(1.891, -44.726), S(47.404, -52.907), S(3.751, -71.362),
    ],
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.894, 0.912, 0.687, 0.408, 0.627],
    [0.129, -0.087, 0.769, 0.754, 0.718],
    [0.033, 0.579, -0.124, 0.617, 0.456],
    [0.267, 0.591, 0.579, 0.036, 1.010],
    [0.118, 0.371, 0.120, 0.191, -0.108],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.285, -1.576];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.863;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.654;

pub struct PolicyParams {}

impl PolicyValues for PolicyParams {
    type Value = f32;

    fn cap_bonus(pt: PieceType) -> Self::Value {
        CAP_BONUS[pt as usize]
    }

    fn pawn_protected_penalty(pt: PieceType) -> Self::Value {
        PAWN_PROTECTED_PENALTY[pt as usize]
    }

    fn pawn_threat_evasion(pt: PieceType) -> Self::Value {
        PAWN_THREAT_EVASION[pt as usize]
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

pub fn get_policy(board: &Board, mv: Move) -> f32 {
    get_policy_impl::<PolicyParams>(board, mv)
}

pub fn get_policy_impl<Params: PolicyValues>(board: &Board, mv: Move) -> Params::Value {
    let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let pawn_protected = attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
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

    let pawn_threat_evasion = if pawn_protected.has(mv.from_sq())
        && !pawn_protected.has(mv.to_sq())
        && moving_piece.piece_type() != PieceType::King
    {
        Params::pawn_threat_evasion(moving_piece.piece_type())
    } else {
        Params::Value::default()
    };

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

    cap_bonus + promo_bonus + pawn_threat_evasion + bad_see_penalty + check_bonus
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
