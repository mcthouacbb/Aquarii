use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign},
};

use crate::{
    chess::{attacks, see, Board, Move, MoveKind}, eval::piece_attacks, types::{Bitboard, Color, Piece, PieceType, Square}
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
    fn mobility(pt: PieceType, mob: u32, phase: i32) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

const CAP_BONUS: [f32; 5] = [1.543, 2.510, 2.694, 2.677, 3.202];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [0.632, 2.162, 1.989, 3.074, 3.331];
const PAWN_THREAT_EVASION: [f32; 5] = [0.238, 2.547, 2.157, 2.395, 2.789];
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(96.311, 154.176), S(65.919, 138.798), S(85.852, 152.793), S(81.588, 151.393), S(91.479, 135.434), S(44.495, 127.242), S(42.982, 122.053), S(76.100, 152.536),
        S(26.788, 56.849), S(26.099, 45.942), S(53.964, 59.136), S(68.066, 50.018), S(66.831, 36.647), S(75.793, 26.430), S(44.212, 23.988), S(38.771, 43.679),
        S(-40.118, -20.130), S(15.722, -30.402), S(14.423, -15.117), S(37.980, -16.547), S(46.890, -30.434), S(32.657, -39.231), S(39.990, -50.385), S(-9.341, -38.813),
        S(-45.045, -80.862), S(-26.521, -67.539), S(-8.463, -62.313), S(10.327, -51.611), S(3.412, -58.939), S(-0.147, -69.545), S(-1.735, -82.901), S(-19.997, -92.217),
        S(-21.754, -102.928), S(-31.159, -71.646), S(-3.692, -79.035), S(-21.752, -48.540), S(-6.129, -62.052), S(-11.274, -74.428), S(11.050, -87.991), S(10.675, -109.169),
        S(-22.946, -85.744), S(-20.154, -66.326), S(-22.152, -61.616), S(-56.435, -37.713), S(-42.497, -38.354), S(3.264, -63.878), S(23.620, -82.056), S(-4.251, -91.946),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(31.426, -62.892), S(23.345, -30.817), S(-22.435, -7.464), S(46.244, -20.504), S(34.240, -14.375), S(10.834, -29.696), S(7.954, -28.656), S(39.608, -54.477),
        S(-22.974, -12.297), S(8.761, -6.153), S(10.732, 3.185), S(3.525, 5.767), S(2.237, 2.493), S(40.477, -9.629), S(-2.647, -5.449), S(19.269, -28.429),
        S(3.814, -11.113), S(33.050, 0.705), S(45.764, 10.870), S(27.487, 22.462), S(52.805, 9.858), S(39.764, 8.991), S(54.639, -8.127), S(21.864, -21.477),
        S(4.941, -0.459), S(12.327, 13.647), S(30.250, 24.755), S(41.851, 22.973), S(43.280, 22.283), S(42.529, 21.057), S(26.588, 11.319), S(44.779, -13.167),
        S(-5.866, -4.403), S(9.479, 7.558), S(20.100, 26.309), S(32.371, 21.557), S(37.694, 19.799), S(32.264, 20.422), S(26.038, 8.729), S(9.417, -13.345),
        S(-44.945, -9.950), S(-7.797, 2.671), S(20.270, 2.348), S(18.532, 16.903), S(30.009, 16.941), S(21.262, -1.204), S(21.017, -8.514), S(-34.003, -8.328),
        S(-50.938, -17.573), S(-33.841, -11.724), S(-16.805, -3.140), S(-13.753, 2.623), S(-8.387, 2.351), S(1.250, -4.044), S(-13.599, -15.470), S(-29.583, -16.373),
        S(-83.201, -9.945), S(-56.315, -40.909), S(-64.707, -9.299), S(-38.395, -13.693), S(-43.069, -16.378), S(-28.552, -19.825), S(-49.316, -38.269), S(-82.078, -1.276),
    ],
    [
        S(4.368, -31.739), S(-65.478, -9.361), S(8.830, -26.316), S(-0.265, -11.969), S(-18.316, -12.745), S(0.227, -22.778), S(-49.176, -13.815), S(9.397, -35.909),
        S(-33.838, -23.660), S(1.946, -7.105), S(-15.533, -0.698), S(-49.116, 3.819), S(-25.069, 0.586), S(-20.439, -2.644), S(-6.353, -5.443), S(-13.430, -33.465),
        S(-30.801, -7.370), S(2.559, 0.947), S(6.990, 13.235), S(21.098, 5.901), S(11.001, 11.646), S(16.391, 12.822), S(18.433, -0.939), S(-15.399, -8.306),
        S(-9.652, -11.920), S(9.880, 8.804), S(9.831, 20.149), S(19.584, 28.384), S(22.693, 26.541), S(13.682, 18.954), S(20.082, 5.823), S(-1.056, -16.864),
        S(7.061, -18.477), S(-6.877, 12.849), S(3.072, 24.992), S(27.773, 26.707), S(26.564, 24.460), S(8.969, 21.017), S(2.965, 5.692), S(19.655, -28.901),
        S(1.424, -14.321), S(15.075, 0.253), S(13.400, 19.258), S(7.628, 20.237), S(5.413, 24.923), S(10.619, 17.977), S(19.080, -4.341), S(8.464, -17.407),
        S(-10.087, -13.397), S(11.109, -11.613), S(8.333, -3.273), S(-17.231, 7.341), S(-3.276, 6.333), S(18.014, -7.117), S(30.940, -13.342), S(-0.805, -24.518),
        S(-24.301, -29.910), S(-18.858, -11.298), S(-18.480, -33.002), S(-34.482, -8.284), S(-22.615, -17.191), S(-17.235, -22.026), S(-13.646, -20.976), S(-26.777, -36.664),
    ],
    [
        S(17.464, 12.869), S(-9.260, 18.229), S(-36.598, 29.959), S(-37.492, 23.304), S(-24.260, 15.946), S(-24.506, 13.900), S(-3.409, 10.901), S(45.221, 1.286),
        S(-2.071, 14.772), S(-0.447, 19.664), S(10.639, 21.222), S(15.741, 11.657), S(15.214, 5.768), S(16.498, 4.035), S(6.459, 5.586), S(35.013, -1.227),
        S(-0.416, 12.042), S(8.148, 10.667), S(16.547, 6.666), S(26.248, -2.273), S(34.165, -11.947), S(30.779, -9.114), S(41.396, -8.391), S(30.872, -8.937),
        S(-6.392, 10.542), S(4.176, 5.743), S(19.611, 4.765), S(19.907, -0.407), S(32.072, -12.456), S(29.245, -11.989), S(33.764, -11.910), S(30.195, -12.465),
        S(-13.663, 6.412), S(-17.290, 8.054), S(7.440, 1.887), S(17.038, -3.543), S(20.813, -8.271), S(9.413, -4.205), S(18.650, -11.138), S(10.110, -11.806),
        S(-34.483, 5.095), S(-12.577, -3.226), S(-5.928, -1.580), S(-3.026, -3.226), S(7.690, -8.586), S(1.652, -11.808), S(27.956, -22.761), S(6.898, -19.553),
        S(-45.535, 1.602), S(-32.067, -0.093), S(-10.626, -3.879), S(-8.964, -5.492), S(-6.171, -10.000), S(-6.759, -12.145), S(10.552, -22.501), S(-18.513, -13.070),
        S(-0.440, -13.901), S(-20.168, 1.231), S(-8.319, 4.139), S(5.724, -2.473), S(4.981, -9.865), S(2.918, -16.624), S(12.930, -18.668), S(8.593, -26.150),
    ],
    [
        S(-9.028, -5.109), S(-29.431, 10.478), S(-46.926, 30.281), S(-51.628, 28.510), S(-35.687, 21.926), S(-49.200, 22.215), S(-10.507, -1.333), S(6.386, -6.605),
        S(-10.208, -16.342), S(-13.919, 6.402), S(-22.494, 22.254), S(-56.449, 43.452), S(-66.324, 55.629), S(-30.148, 26.977), S(-17.994, 12.716), S(5.003, -5.682),
        S(-2.802, -21.646), S(11.367, -10.532), S(-4.646, 16.351), S(7.581, 11.207), S(3.961, 22.259), S(6.365, 18.440), S(32.239, -6.378), S(12.032, -8.281),
        S(-14.632, -6.408), S(8.250, -2.692), S(1.773, 10.591), S(10.493, 18.208), S(16.994, 19.656), S(14.540, 18.994), S(23.137, 6.899), S(20.295, -9.277),
        S(5.457, -20.161), S(-7.810, 5.567), S(15.113, 2.083), S(24.522, 8.432), S(24.516, 12.125), S(23.493, 3.609), S(12.863, 5.111), S(19.317, -14.895),
        S(-10.266, -14.214), S(6.949, -9.277), S(20.199, -3.558), S(10.079, 6.275), S(19.359, 4.000), S(24.447, -2.823), S(33.983, -16.846), S(15.677, -22.395),
        S(-21.103, -15.328), S(-3.329, -11.832), S(12.748, -20.494), S(6.497, -7.159), S(9.854, -9.235), S(21.600, -25.422), S(24.917, -34.238), S(10.376, -41.961),
        S(-12.189, -21.334), S(-42.750, -1.762), S(-23.556, -8.496), S(12.574, -64.381), S(-9.206, -20.524), S(-25.019, -6.737), S(-11.051, -20.752), S(-23.545, -24.037),
    ],
    [
        S(-8.117, -32.683), S(-4.483, -13.857), S(-12.692, -7.726), S(-63.499, 5.376), S(-50.322, 6.442), S(-24.320, 8.927), S(-6.164, 5.942), S(20.499, -28.695),
        S(-45.624, -6.010), S(-38.025, 25.137), S(-57.562, 24.312), S(-41.920, 22.742), S(-73.205, 36.038), S(-37.427, 39.570), S(-10.063, 30.947), S(-39.995, 13.091),
        S(-90.730, 7.659), S(-33.517, 27.077), S(-72.811, 38.989), S(-92.170, 50.912), S(-73.702, 53.593), S(-23.737, 47.417), S(-19.472, 40.164), S(-46.627, 17.779),
        S(-84.704, 5.425), S(-68.281, 28.621), S(-90.312, 44.079), S(-136.137, 58.289), S(-116.412, 57.354), S(-103.806, 53.975), S(-88.183, 40.595), S(-109.430, 19.191),
        S(-76.884, -6.271), S(-73.607, 18.519), S(-83.323, 34.065), S(-114.447, 50.988), S(-115.176, 51.040), S(-86.414, 39.266), S(-81.482, 25.209), S(-106.853, 9.163),
        S(-29.997, -20.200), S(-7.139, -0.962), S(-43.827, 17.152), S(-59.623, 29.421), S(-57.568, 29.610), S(-49.711, 20.364), S(-21.886, 4.190), S(-30.079, -13.862),
        S(30.871, -39.296), S(14.753, -17.861), S(6.080, -5.168), S(-20.015, 2.375), S(-19.966, 5.351), S(4.309, -5.163), S(27.244, -18.619), S(20.155, -35.378),
        S(-5.743, -68.565), S(63.930, -58.450), S(51.216, -44.725), S(-6.491, -38.547), S(58.581, -56.069), S(9.640, -42.926), S(56.242, -53.175), S(14.019, -72.511),
    ],
];
const MOBILITY: [[(f32, f32); 28]; 4] = [
    [S(12.967, -59.716),S(-5.868, -12.116),S(-6.977, -14.944),S(-3.315, -1.874),S(-0.829, 2.550),S(1.715, 7.364),S(4.434, 6.311),S(7.512, 6.052),S(11.340, 1.850),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),],
    [S(-0.612, 36.349),S(0.204, 13.132),S(-6.411, 16.883),S(-0.780, 11.030),S(3.046, 8.776),S(5.702, 6.273),S(5.803, 3.694),S(4.662, 0.917),S(3.960, -4.762),S(0.132, -6.574),S(-0.932, -10.419),S(-8.709, -12.956),S(-3.516, -19.797),S(-10.105, -18.876),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),],
    [S(5.411, 48.873),S(-4.370, 49.194),S(-36.494, 33.676),S(-20.445, 9.009),S(-8.440, 7.796),S(-3.728, 7.136),S(0.696, 7.012),S(6.542, 2.762),S(9.162, 0.707),S(13.768, -2.601),S(15.856, -3.799),S(16.917, -6.042),S(15.648, -7.430),S(15.131, -10.984),S(1.831, -9.290),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),S(0.000, 0.000),],
    [S(0.000, 0.000),S(0.000, 0.000),S(-70.570, 33.894),S(16.187, -54.341),S(14.066, -29.781),S(-4.704, 6.609),S(1.842, -8.350),S(3.513, -0.637),S(5.036, -5.973),S(5.621, -2.231),S(4.886, 1.495),S(3.685, 3.195),S(2.306, 4.247),S(1.988, 4.709),S(1.452, 1.912),S(-0.299, 1.989),S(0.262, 0.016),S(-4.251, 0.246),S(-6.053, 0.539),S(-5.720, -0.203),S(-9.889, -0.693),S(-13.766, 2.213),S(-16.470, -2.860),S(-20.775, 0.925),S(-14.028, -2.593),S(-30.613, 4.820),S(-14.465, -3.340),S(-22.023, 4.010),],
];
const PROMO_BONUS: [f32; 2] = [1.167, -1.952];
const BAD_SEE_PENALTY: f32 = -2.548;
const CHECK_BONUS: f32 = 0.504;

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
        (PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].0 * phase.min(24) as f32
            + PSQT_SCORE[pt as usize][sq.relative_sq(c).flip() as usize].1
                * (24 - phase.min(24)) as f32)
            / 24.0
    }

    fn mobility(pt: PieceType, mob: u32, phase: i32) -> Self::Value {
        let (mg, eg) = MOBILITY[pt as usize - PieceType::Knight as usize][mob as usize];
        (mg * phase.min(24) as f32 + eg * (24 - phase.min(24)) as f32) / 24.0
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
        + board.pieces(PieceType::Knight).popcount()) as i32;
    let psqt = if mv.kind() != MoveKind::Promotion {

        Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.to_sq(), phase)
            - Params::psqt_score(board.stm(), moving_piece.piece_type(), mv.from_sq(), phase)
    } else {
        Params::Value::default()
    };

    let mobility_score = if mv.kind() == MoveKind::None && moving_piece.piece_type() != PieceType::Pawn && moving_piece.piece_type() != PieceType::King {
        let attacks_before = piece_attacks(moving_piece.piece_type(), mv.from_sq(), board.occ());

        let occ_after = board.occ() | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
        let attacks_after = piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after);
        
        let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
        let mobility_area = !attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
        let mobility_before = (attacks_before & mobility_area).popcount();
        let mobility_after = (attacks_after & mobility_area).popcount();

        Params::mobility(moving_piece.piece_type(), mobility_after, phase) - Params::mobility(moving_piece.piece_type(), mobility_before, phase)
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
        + mobility_score / 50.0
}
