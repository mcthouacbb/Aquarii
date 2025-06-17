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
    fn king_ring_attacks(moving: PieceType, attacks: u32, phase: i32) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

const CAP_BONUS: [f32; 5] = [1.573, 2.543, 2.711, 2.684, 3.210];
const PAWN_PROTECTED_PENALTY: [f32; 5] = [0.672, 2.209, 1.990, 3.182, 3.380];
const PAWN_THREAT_EVASION: [f32; 5] = [0.333, 2.381, 2.060, 2.215, 2.761];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(122.610, 146.407), S(89.757, 131.912), S(110.994, 148.606), S(103.520, 151.031), S(114.022, 136.902), S(63.256, 130.369), S(63.196, 126.215), S(96.551, 157.783),
        S(43.108, 49.253), S(41.408, 39.379), S(69.031, 55.492), S(82.749, 49.645), S(80.850, 37.912), S(89.045, 28.882), S(56.883, 28.622), S(50.649, 49.054),
        S(-32.468, -28.094), S(22.801, -36.880), S(21.213, -19.150), S(44.433, -17.058), S(52.997, -29.130), S(38.071, -36.954), S(44.895, -46.362), S(-5.326, -33.946),
        S(-46.008, -89.554), S(-26.876, -74.797), S(-9.107, -67.176), S(9.690, -53.159), S(2.022, -58.222), S(-2.139, -67.646), S(-4.372, -79.732), S(-24.066, -88.221),
        S(-26.543, -113.327), S(-35.519, -80.242), S(-8.173, -85.465), S(-26.460, -53.250), S(-11.531, -63.856), S(-17.576, -73.356), S(4.266, -86.039), S(2.483, -107.202),
        S(-33.851, -97.245), S(-30.946, -76.006), S(-32.888, -69.025), S(-66.050, -44.274), S(-53.500, -41.460), S(-9.164, -64.010), S(10.189, -81.290), S(-18.454, -90.920),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(9.037, -72.895), S(0.306, -31.301), S(-53.987, -1.676), S(13.792, -14.946), S(3.067, -9.100), S(-26.007, -23.605), S(-12.648, -28.486), S(12.340, -64.423),
        S(-41.361, -14.184), S(-11.750, -3.230), S(-19.401, 13.735), S(-22.260, 13.773), S(-24.534, 11.381), S(-1.613, 3.669), S(-21.447, -3.988), S(-2.836, -30.172),
        S(-9.351, -10.155), S(4.987, 10.446), S(25.942, 16.232), S(4.831, 27.870), S(13.351, 20.280), S(30.263, 11.280), S(26.044, 2.397), S(4.412, -19.025),
        S(2.069, -0.614), S(10.512, 18.131), S(27.896, 25.895), S(40.166, 24.689), S(42.660, 24.161), S(33.443, 23.939), S(25.259, 14.673), S(39.863, -14.241),
        S(-3.320, -8.889), S(10.351, 8.719), S(26.203, 26.563), S(38.876, 21.446), S(44.202, 20.343), S(37.176, 20.147), S(27.668, 11.326), S(11.389, -18.477),
        S(-42.390, -12.624), S(3.624, 3.809), S(32.842, 3.295), S(31.032, 16.168), S(40.379, 17.181), S(34.882, 0.836), S(31.493, -7.402), S(-30.615, -8.861),
        S(-44.738, -25.073), S(-24.095, -14.769), S(-4.711, -1.803), S(-3.449, 5.261), S(3.102, 5.819), S(12.495, -1.314), S(-5.700, -18.189), S(-24.362, -22.574),
        S(-78.236, -25.174), S(-57.321, -41.912), S(-50.616, -11.936), S(-27.619, -15.762), S(-31.744, -18.894), S(-14.753, -21.924), S(-49.959, -39.067), S(-73.620, -17.045),
    ],
    [
        S(6.015, -20.092), S(-63.650, 0.589), S(10.290, -18.924), S(-5.551, -3.339), S(-22.918, -4.943), S(3.765, -15.177), S(-49.899, -4.591), S(13.102, -23.700),
        S(-34.052, -13.368), S(-13.742, -0.954), S(-22.920, 2.256), S(-56.372, 4.262), S(-50.530, 5.534), S(-27.992, -0.625), S(-18.862, 1.674), S(-19.932, -22.506),
        S(-25.449, -1.594), S(-0.668, 3.230), S(-6.632, 9.315), S(11.051, 1.680), S(2.995, 4.984), S(-1.823, 11.802), S(15.812, -0.100), S(-7.247, -2.384),
        S(-9.668, -4.110), S(3.867, 8.541), S(9.561, 12.651), S(8.384, 16.705), S(14.561, 15.950), S(11.534, 10.880), S(13.990, 7.085), S(-0.575, -10.180),
        S(3.369, -10.180), S(-4.386, 11.320), S(2.033, 17.066), S(26.750, 15.411), S(21.289, 13.375), S(10.220, 13.770), S(5.654, 4.402), S(16.484, -18.982),
        S(4.703, -8.270), S(17.617, 0.600), S(17.348, 13.037), S(9.444, 12.726), S(11.218, 18.034), S(12.303, 11.922), S(24.313, -3.433), S(8.648, -10.646),
        S(-9.324, -3.814), S(19.079, -11.366), S(13.574, -3.266), S(-9.248, 5.567), S(1.577, 4.143), S(26.078, -6.651), S(35.074, -12.950), S(1.718, -14.241),
        S(-15.927, -19.364), S(-17.881, -1.621), S(-14.683, -27.526), S(-29.746, -2.900), S(-15.483, -11.270), S(-17.051, -15.957), S(-8.308, -11.016), S(-21.738, -26.286),
    ],
    [
        S(12.356, 18.441), S(-9.734, 20.150), S(-35.641, 29.848), S(-35.378, 22.599), S(-27.075, 17.045), S(-26.754, 15.702), S(-4.986, 13.059), S(42.069, 5.958),
        S(-8.490, 17.160), S(-8.007, 19.424), S(-7.068, 20.815), S(-5.522, 11.733), S(-5.234, 5.990), S(-6.308, 5.896), S(-5.814, 6.794), S(28.289, 0.481),
        S(-12.200, 14.263), S(-8.630, 10.794), S(-6.641, 5.990), S(1.539, -3.243), S(7.935, -12.944), S(2.901, -8.808), S(18.744, -7.040), S(17.194, -6.204),
        S(-11.403, 11.089), S(-7.198, 4.137), S(3.407, 2.690), S(4.989, -3.904), S(14.383, -15.661), S(13.989, -15.094), S(19.216, -13.451), S(24.556, -12.548),
        S(-15.309, 6.630), S(-19.032, 4.753), S(-1.830, -1.105), S(12.220, -8.288), S(14.614, -12.875), S(1.203, -8.031), S(14.013, -13.889), S(8.635, -12.157),
        S(-27.500, 4.618), S(-8.623, -5.791), S(-2.719, -5.524), S(3.386, -9.107), S(11.617, -13.555), S(6.994, -16.837), S(33.297, -26.475), S(15.093, -21.378),
        S(-40.296, 3.363), S(-23.719, -2.090), S(-1.711, -7.504), S(0.890, -10.227), S(2.117, -14.499), S(3.147, -16.255), S(19.291, -24.545), S(-14.502, -11.382),
        S(3.508, -9.077), S(-9.759, 1.570), S(2.090, 3.180), S(16.858, -3.977), S(15.417, -10.577), S(11.184, -15.749), S(20.722, -16.889), S(10.670, -20.018),
    ],
    [
        S(-8.436, -1.230), S(-30.474, 13.979), S(-52.075, 34.785), S(-56.632, 32.184), S(-43.592, 26.797), S(-55.879, 27.204), S(-13.299, 2.827), S(6.376, -2.643),
        S(-9.986, -13.799), S(-20.675, 9.246), S(-32.186, 24.364), S(-72.258, 47.508), S(-82.665, 60.108), S(-42.864, 30.217), S(-26.587, 16.240), S(3.904, -2.848),
        S(-3.480, -18.714), S(4.793, -9.028), S(-15.070, 16.954), S(-7.258, 12.184), S(-11.628, 23.833), S(-6.754, 19.801), S(25.443, -5.289), S(12.662, -6.004),
        S(-13.804, -4.449), S(2.627, -1.431), S(-6.379, 9.832), S(-1.672, 17.244), S(5.365, 18.256), S(5.648, 19.009), S(18.148, 7.871), S(21.605, -7.763),
        S(7.764, -18.646), S(-6.103, 3.272), S(11.825, 0.023), S(19.808, 4.598), S(19.403, 8.760), S(19.547, 1.350), S(14.743, 3.303), S(22.288, -13.956),
        S(-3.645, -14.375), S(13.660, -13.114), S(23.465, -7.685), S(15.256, -0.834), S(23.735, -1.834), S(27.138, -5.745), S(39.577, -19.696), S(21.336, -21.552),
        S(-10.753, -16.703), S(7.414, -16.355), S(22.377, -26.156), S(17.426, -14.186), S(20.416, -15.636), S(31.563, -30.262), S(34.689, -37.587), S(19.296, -41.985),
        S(-0.286, -22.218), S(-29.761, -3.926), S(-9.641, -11.275), S(22.922, -65.355), S(3.811, -23.633), S(-12.246, -7.628), S(0.668, -20.770), S(-13.888, -23.486),
    ],
    [
        S(21.830, -36.270), S(36.651, -20.255), S(27.698, -13.989), S(-24.337, -0.582), S(-11.300, 0.894), S(15.547, 3.065), S(33.653, 0.097), S(58.848, -33.627),
        S(-10.221, -11.260), S(-3.857, 20.215), S(-24.825, 19.719), S(-10.037, 18.441), S(-40.983, 31.477), S(-5.624, 35.207), S(23.477, 26.045), S(-5.048, 8.185),
        S(-58.090, 2.820), S(-5.112, 23.213), S(-45.998, 35.370), S(-66.928, 47.654), S(-48.583, 50.263), S(3.427, 44.175), S(8.237, 36.144), S(-16.510, 13.581),
        S(-63.136, 2.781), S(-49.307, 26.018), S(-71.863, 41.622), S(-120.151, 56.493), S(-103.157, 56.220), S(-85.706, 51.676), S(-68.746, 37.872), S(-88.560, 16.573),
        S(-63.023, -7.978), S(-62.178, 17.266), S(-73.716, 32.925), S(-107.391, 50.128), S(-109.034, 50.413), S(-78.424, 38.160), S(-73.366, 24.118), S(-96.067, 8.210),
        S(-23.202, -20.262), S(-2.938, -0.983), S(-41.796, 17.209), S(-58.766, 29.691), S(-57.677, 30.250), S(-48.492, 20.534), S(-19.849, 4.245), S(-26.722, -13.814),
        S(33.403, -38.765), S(15.594, -17.174), S(5.197, -4.150), S(-24.475, 3.875), S(-24.380, 6.890), S(-0.623, -3.537), S(25.675, -18.244), S(19.349, -34.945),
        S(-6.516, -66.071), S(62.709, -57.402), S(45.340, -42.953), S(-11.789, -36.128), S(48.508, -53.109), S(4.376, -40.986), S(48.943, -51.765), S(11.793, -71.224),
    ],
];
const THREAT: [[f32; 5]; 4] = [
    [0.136, -0.043, 0.737, 0.638, 0.694],
    [0.059, 0.646, -0.176, 0.703, 0.675],
    [0.256, 0.572, 0.577, -0.046, 1.038],
    [0.141, 0.411, 0.102, 0.092, 0.092],
];
const KING_RING_ATTACKS: [[(f32, f32); 9]; 4] = [[(0.0, 0.0); 9]; 4];
const PROMO_BONUS: [f32; 2] = [1.052, -1.822];
const BAD_SEE_PENALTY: f32 = -2.560;
const CHECK_BONUS: f32 = 0.559;

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

    fn threat(moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[moving as usize - PieceType::Knight as usize][threatened as usize]
    }

    fn king_ring_attacks(moving: PieceType, attacks: u32, phase: i32) -> Self::Value {
        let (mg, eg) = KING_RING_ATTACKS[moving as usize - PieceType::Knight as usize][attacks as usize];
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

    let mut threat_score = Params::Value::default();
    let mut king_attack_score = Params::Value::default();
    if mv.kind() == MoveKind::None
        && moving_piece.piece_type() != PieceType::King
        && moving_piece.piece_type() != PieceType::Pawn
    {
        let occ_after =
            board.occ() | Bitboard::from_square(mv.to_sq()) & !Bitboard::from_square(mv.from_sq());
        let attacks_after =
            attacks::piece_attacks(moving_piece.piece_type(), mv.to_sq(), occ_after);

        let mut threats =
            attacks_after & board.colors(!board.stm()) & !board.pieces(PieceType::King);
        while threats.any() {
            let threat = threats.poplsb();
            threat_score += Params::threat(
                moving_piece.piece_type(),
                board.piece_at(threat).unwrap().piece_type(),
            );
        }

        let opp_king_ring = {
            let king_sq = board.king_sq(!board.stm());
            let attacks = attacks::king_attacks(king_sq);
            (attacks | attacks::pawn_pushes_bb(!board.stm(), attacks))
                & !Bitboard::from_square(king_sq)
        };

        let king_ring_attacks = attacks_after & opp_king_ring;
        if king_ring_attacks.any() {
            king_attack_score +=
                Params::king_ring_attacks(moving_piece.piece_type(), king_ring_attacks.popcount(), phase);
        }
    }

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
        + king_attack_score
}
