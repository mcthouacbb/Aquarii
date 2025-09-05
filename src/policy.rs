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
    fn passed_pawn_push(rank: u8, phase: i32) -> Self::Value;
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
const CAP_BONUS: [f32; 5] = [1.538, 2.685, 2.728, 2.662, 3.520];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.424, 2.239, 1.860, 2.976, 3.379];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 5] = [
    [0.327, 2.583, 2.247, 2.410, 3.015],
    [0.330, 0.099, 1.295, 1.856, 2.376],
    [0.264, 0.551, 0.154, 1.892, 2.351],
    [0.028, 0.480, 0.631, 0.549, 2.304],
    [-0.035, 0.309, 0.468, 0.481, 0.802],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(120.067, 100.285), S(61.335, 98.489), S(88.587, 91.215), S(95.405, 108.854), S(71.196, 96.694), S(66.869, 85.232), S(40.348, 92.384), S(79.828, 110.848),
        S(39.640, 40.648), S(46.756, 36.732), S(45.272, 38.951), S(68.527, 42.452), S(69.266, 28.217), S(68.238, 23.342), S(57.182, 25.629), S(46.767, 40.231),
        S(-15.784, -11.988), S(31.673, -13.746), S(24.426, -9.661), S(39.765, -1.784), S(49.474, -9.372), S(37.530, -14.009), S(47.983, -23.011), S(-2.593, -11.616),
        S(-37.336, -53.726), S(-18.695, -34.977), S(0.645, -33.837), S(22.734, -21.816), S(12.836, -23.163), S(6.962, -30.914), S(-3.841, -32.491), S(-26.445, -45.467),
        S(-18.379, -71.102), S(-26.194, -29.813), S(-0.038, -40.968), S(-13.626, -11.514), S(-0.060, -16.735), S(-9.515, -27.750), S(8.068, -31.981), S(1.416, -58.708),
        S(-31.472, -48.934), S(-25.894, -20.891), S(-37.296, -12.681), S(-64.568, 15.842), S(-49.315, 14.824), S(-4.543, -12.345), S(9.979, -20.415), S(-26.127, -37.536),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(24.441, -70.013), S(-2.333, -27.016), S(-51.529, 1.857), S(23.023, -17.013), S(8.920, -9.959), S(-13.185, -21.128), S(-12.335, -21.502), S(29.050, -68.511),
        S(-28.522, -18.270), S(-16.862, -2.025), S(-6.715, 10.193), S(-15.357, 13.443), S(-33.683, 20.855), S(5.352, 4.708), S(-16.635, -9.498), S(-2.362, -36.420),
        S(-16.296, -4.136), S(13.619, 7.291), S(26.557, 18.168), S(16.987, 25.849), S(19.357, 19.441), S(28.724, 13.279), S(35.458, -2.674), S(14.335, -14.322),
        S(9.473, -7.059), S(15.587, 19.429), S(32.071, 22.804), S(42.832, 24.245), S(47.569, 23.019), S(41.923, 25.745), S(26.633, 15.383), S(43.334, -16.775),
        S(-4.909, -5.548), S(15.438, 11.634), S(24.390, 27.308), S(41.696, 20.426), S(46.437, 21.790), S(38.326, 21.726), S(34.294, 11.228), S(10.038, -16.613),
        S(-39.383, -11.011), S(5.580, 2.034), S(32.128, 3.504), S(34.737, 16.820), S(43.343, 15.973), S(38.782, 1.813), S(33.666, -4.345), S(-27.094, -11.636),
        S(-55.040, -22.943), S(-32.720, -9.458), S(-0.591, 0.537), S(-3.747, 7.492), S(5.844, 3.517), S(18.742, -2.432), S(0.610, -16.436), S(-29.553, -22.807),
        S(-92.834, -11.843), S(-59.525, -48.096), S(-54.912, -10.558), S(-33.887, -15.697), S(-32.422, -18.827), S(-16.054, -23.542), S(-56.610, -37.194), S(-61.156, -36.011),
    ],
    [
        S(6.193, -22.842), S(-61.167, 6.887), S(-2.852, -15.027), S(-8.606, -8.589), S(-22.048, -7.739), S(3.167, -19.071), S(-67.591, 0.616), S(7.919, -24.029),
        S(-38.546, -15.304), S(-12.242, -4.202), S(-20.989, 0.882), S(-74.276, 8.781), S(-54.250, 4.554), S(-26.707, -0.187), S(-22.016, -1.042), S(-8.854, -28.669),
        S(-24.528, -1.770), S(3.141, 0.441), S(-16.553, 13.102), S(8.963, 4.753), S(11.418, 4.961), S(-1.966, 12.530), S(23.131, -3.232), S(-6.541, -4.559),
        S(-10.718, -6.753), S(2.351, 10.808), S(14.731, 9.275), S(16.454, 17.856), S(19.367, 15.954), S(18.895, 10.236), S(9.930, 5.235), S(10.506, -15.669),
        S(-4.701, -10.249), S(0.061, 10.024), S(6.173, 19.744), S(32.155, 13.639), S(29.951, 11.827), S(13.141, 16.167), S(7.446, 7.940), S(12.605, -22.222),
        S(5.461, -12.559), S(18.659, 0.005), S(23.193, 10.033), S(14.593, 11.125), S(14.214, 19.315), S(19.314, 9.458), S(24.969, -6.079), S(8.325, -11.972),
        S(-9.467, -3.684), S(21.528, -11.981), S(18.066, -7.493), S(-6.178, 4.878), S(5.771, 4.026), S(27.561, -5.283), S(38.302, -11.766), S(2.427, -22.393),
        S(-25.146, -14.831), S(-17.055, -5.080), S(-16.421, -22.218), S(-26.749, -6.884), S(-18.936, -7.582), S(-21.528, -10.259), S(-17.760, -12.546), S(-31.767, -23.475),
    ],
    [
        S(25.456, 13.327), S(3.354, 16.812), S(-18.859, 25.985), S(-14.847, 19.802), S(-15.825, 16.149), S(-26.057, 16.259), S(6.451, 10.516), S(50.222, 1.573),
        S(0.051, 20.008), S(1.347, 18.976), S(13.124, 14.366), S(8.321, 8.944), S(0.962, 5.632), S(-3.644, 5.875), S(-0.268, 7.508), S(21.972, 4.397),
        S(-10.900, 14.623), S(-6.162, 11.301), S(-0.889, 4.689), S(8.265, -4.376), S(14.386, -13.112), S(2.626, -8.910), S(25.529, -8.477), S(22.256, -6.141),
        S(-18.753, 11.984), S(-8.554, 6.488), S(5.000, 1.738), S(11.066, -4.142), S(13.797, -15.748), S(9.905, -14.014), S(17.511, -12.886), S(21.539, -10.668),
        S(-21.597, 8.363), S(-20.466, 5.150), S(1.085, -1.791), S(8.651, -5.218), S(13.736, -12.132), S(2.804, -10.320), S(15.447, -17.537), S(5.189, -10.735),
        S(-28.987, 6.910), S(-11.875, -2.550), S(-2.692, -5.454), S(1.150, -8.326), S(11.974, -12.953), S(7.358, -15.357), S(35.604, -28.875), S(19.247, -22.866),
        S(-45.913, 5.553), S(-27.325, 0.251), S(-7.348, -3.767), S(-3.264, -4.495), S(0.893, -13.165), S(3.689, -16.182), S(20.346, -25.128), S(-19.572, -10.349),
        S(-0.415, -10.129), S(-13.859, 0.266), S(1.161, 0.640), S(15.338, -4.797), S(15.136, -12.844), S(8.728, -16.285), S(18.807, -19.268), S(8.543, -20.691),
    ],
    [
        S(-7.943, -0.222), S(-27.586, 15.878), S(-34.584, 25.091), S(-55.727, 34.737), S(-38.163, 28.184), S(-53.245, 29.833), S(-11.711, 6.955), S(13.278, -3.424),
        S(-21.771, -0.233), S(-16.421, 11.810), S(-15.571, 16.649), S(-49.099, 36.876), S(-49.185, 45.394), S(-33.184, 25.065), S(-30.426, 20.512), S(6.177, -0.015),
        S(-10.081, -7.601), S(4.902, -4.844), S(-9.219, 12.686), S(4.524, 6.618), S(1.629, 15.768), S(8.323, 14.165), S(28.740, -5.917), S(17.877, -11.239),
        S(-14.063, -0.758), S(-0.419, 4.988), S(-0.511, 7.961), S(6.874, 9.588), S(13.656, 13.904), S(7.898, 15.191), S(22.424, 6.490), S(20.093, -8.801),
        S(-4.517, -7.129), S(-2.721, 5.274), S(7.978, 2.372), S(20.558, 3.980), S(18.396, 5.173), S(18.576, 1.570), S(13.288, 3.815), S(18.376, -11.022),
        S(-8.975, -3.918), S(11.171, -8.103), S(19.347, -7.936), S(12.355, 0.876), S(18.886, 2.266), S(23.260, -2.744), S(36.729, -18.201), S(19.490, -18.389),
        S(-22.859, -3.031), S(5.594, -14.422), S(18.449, -23.696), S(11.881, -5.062), S(14.940, -9.207), S(26.791, -19.530), S(26.057, -15.509), S(13.326, -30.175),
        S(-18.707, -4.001), S(-39.976, 4.084), S(-14.935, -11.910), S(18.176, -71.258), S(-2.224, -21.644), S(-18.493, -6.579), S(-11.737, -7.629), S(-28.931, -16.657),
    ],
    [
        S(30.412, -20.774), S(32.606, -11.322), S(37.892, -9.044), S(9.617, -2.139), S(7.110, -2.179), S(50.195, -0.973), S(57.744, -3.834), S(56.332, -19.923),
        S(23.380, -12.544), S(32.207, 12.890), S(23.508, 13.194), S(14.918, 15.507), S(4.060, 23.504), S(27.915, 28.144), S(65.846, 17.342), S(49.053, 0.842),
        S(-12.729, 1.018), S(15.270, 20.379), S(-17.458, 31.113), S(-42.285, 44.237), S(-33.155, 49.779), S(20.706, 44.024), S(37.973, 33.446), S(-7.646, 14.087),
        S(-35.727, -1.295), S(-35.474, 23.865), S(-55.126, 38.716), S(-95.059, 54.623), S(-86.825, 55.158), S(-73.223, 52.233), S(-50.539, 37.013), S(-73.623, 15.858),
        S(-46.229, -5.433), S(-40.805, 14.557), S(-71.523, 33.893), S(-94.160, 47.787), S(-102.192, 50.309), S(-81.258, 40.702), S(-64.025, 24.462), S(-79.315, 4.630),
        S(-14.785, -19.362), S(-3.202, 1.321), S(-45.082, 18.300), S(-58.127, 28.182), S(-61.224, 31.352), S(-56.883, 25.861), S(-26.535, 7.498), S(-23.524, -16.004),
        S(32.063, -37.164), S(10.877, -14.163), S(-5.030, -2.588), S(-34.498, 5.642), S(-33.247, 7.976), S(-12.217, -1.472), S(18.098, -16.956), S(10.620, -33.158),
        S(-14.659, -64.785), S(56.965, -55.626), S(34.172, -41.331), S(-18.785, -36.976), S(48.132, -55.583), S(-2.236, -43.242), S(42.748, -52.526), S(3.808, -71.474),
    ],
];
#[rustfmt::skip]
const PASSED_PAWN_PUSH: [(f32, f32); 8] = [
    S(0.000, 0.000), S(0.000, 0.000), S(-0.310, 0.299), S(-0.182, 0.731), S(-0.049, 0.998), S(0.208, 0.995), S(0.544, 0.967), S(-0.834, 2.371)
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.914, 0.773, 0.729, 0.182, 0.758],
    [0.101, 0.023, 0.725, 0.612, 0.683],
    [0.004, 0.600, -0.005, 0.646, 0.571],
    [0.258, 0.503, 0.531, -0.027, 1.028],
    [0.063, 0.227, 0.114, 0.141, -0.131],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [0.229, -2.504];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.908;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.722;

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

    fn passed_pawn_push(rank: u8, phase: i32) -> Self::Value {
        (PASSED_PAWN_PUSH[rank as usize].0 * phase as f32
            + PASSED_PAWN_PUSH[rank as usize].1 * (24 - phase) as f32)
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

    let their_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
    let mut pawn_score = Params::Value::default();
    if moving_piece.piece_type() == PieceType::Pawn && !data.attacked().has(mv.to_sq()) {
        let sq = mv.to_sq();
        let rank = sq.relative_sq(board.stm()).rank();
        let stoppers = their_pawns & attacks::passed_pawn_span(board.stm(), mv.to_sq());
        if stoppers.empty() {
            pawn_score += Params::passed_pawn_push(rank, phase);
        }
    }

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

    cap_bonus + promo_bonus + threat_evasion + bad_see_penalty + check_bonus + pawn_score
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
