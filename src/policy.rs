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
    fn direct_check_bonus() -> Self::Value;
    fn discovered_check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

#[rustfmt::skip]
const CAP_BONUS: [f32; 5] = [1.561, 2.601, 2.682, 2.748, 3.539];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.426, 2.197, 1.907, 2.943, 3.250];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 5] = [
    [0.330, 2.611, 2.301, 2.342, 2.975],
    [0.296, 0.014, 1.232, 1.905, 2.408],
    [0.233, 0.636, 0.174, 1.849, 2.350],
    [0.004, 0.502, 0.511, 0.548, 2.374],
    [0.028, 0.339, 0.530, 0.567, 0.745],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(120.710, 100.381), S(61.784, 98.864), S(88.164, 91.492), S(95.329, 108.386), S(70.780, 95.653), S(64.824, 84.895), S(40.128, 91.718), S(79.652, 109.965),
        S(39.836, 41.067), S(46.639, 37.487), S(45.390, 39.808), S(68.668, 43.299), S(69.611, 28.120), S(68.190, 23.491), S(57.465, 25.418), S(47.052, 39.887),
        S(-15.593, -11.286), S(31.769, -12.911), S(24.449, -8.576), S(40.032, -0.610), S(49.513, -8.998), S(37.653, -13.660), S(48.029, -22.833), S(-2.346, -11.757),
        S(-37.264, -52.921), S(-18.683, -33.914), S(0.757, -32.798), S(22.982, -20.592), S(12.959, -22.193), S(7.124, -30.259), S(-3.787, -32.075), S(-26.214, -45.538),
        S(-18.248, -70.250), S(-25.875, -28.736), S(0.091, -39.918), S(-13.589, -10.556), S(-0.010, -15.932), S(-9.360, -26.945), S(8.191, -31.668), S(1.640, -58.595),
        S(-31.182, -48.198), S(-25.521, -19.723), S(-37.092, -11.558), S(-64.547, 17.029), S(-49.121, 15.479), S(-4.607, -11.648), S(10.165, -20.248), S(-26.239, -37.921),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(24.685, -70.812), S(-3.674, -26.582), S(-51.539, 1.672), S(23.287, -16.922), S(9.512, -9.888), S(-13.510, -20.907), S(-13.157, -21.294), S(29.634, -68.884),
        S(-28.549, -18.395), S(-16.827, -1.982), S(-6.464, 10.219), S(-14.834, 13.681), S(-33.068, 21.275), S(5.818, 4.969), S(-16.338, -9.243), S(-2.011, -36.440),
        S(-15.505, -4.346), S(14.043, 7.239), S(26.886, 18.204), S(17.538, 25.930), S(20.075, 19.758), S(29.525, 13.295), S(36.008, -2.673), S(14.822, -14.356),
        S(9.532, -7.047), S(15.804, 19.431), S(32.264, 22.947), S(42.959, 24.446), S(47.754, 23.342), S(42.105, 25.938), S(26.809, 15.744), S(43.337, -16.663),
        S(-4.748, -5.472), S(15.641, 11.729), S(24.615, 27.642), S(41.857, 20.495), S(46.703, 21.862), S(38.591, 21.898), S(34.359, 11.336), S(10.254, -16.634),
        S(-39.308, -11.187), S(5.602, 1.664), S(32.319, 3.723), S(34.758, 16.649), S(43.568, 15.888), S(38.900, 1.841), S(33.667, -4.595), S(-26.964, -11.562),
        S(-54.734, -23.229), S(-32.674, -9.370), S(-0.518, 0.488), S(-3.668, 7.495), S(5.969, 3.651), S(18.779, -2.262), S(0.508, -16.517), S(-29.499, -22.847),
        S(-92.979, -11.664), S(-59.391, -47.944), S(-54.617, -10.504), S(-33.676, -15.721), S(-32.281, -18.667), S(-15.808, -23.423), S(-56.471, -37.237), S(-60.932, -35.795),
    ],
    [
        S(6.115, -22.752), S(-61.196, 6.911), S(-2.894, -15.121), S(-8.931, -8.437), S(-22.629, -7.626), S(3.295, -19.267), S(-67.649, 0.415), S(7.583, -24.168),
        S(-38.436, -15.407), S(-11.988, -4.233), S(-21.051, 0.770), S(-74.127, 8.789), S(-54.212, 4.690), S(-26.166, -0.121), S(-22.054, -1.090), S(-8.577, -28.614),
        S(-24.520, -1.874), S(3.021, 0.251), S(-16.319, 12.991), S(8.998, 4.679), S(11.555, 4.841), S(-1.974, 12.654), S(23.198, -3.235), S(-6.634, -4.572),
        S(-10.931, -6.612), S(2.443, 10.918), S(14.629, 9.183), S(16.518, 17.919), S(19.224, 15.951), S(18.831, 10.363), S(10.006, 5.129), S(10.417, -15.432),
        S(-4.664, -10.250), S(-0.081, 10.045), S(6.061, 19.719), S(32.199, 13.671), S(30.071, 11.815), S(13.040, 16.122), S(7.341, 7.682), S(12.406, -22.380),
        S(5.237, -12.554), S(18.596, 0.072), S(23.029, 9.889), S(14.583, 11.118), S(13.955, 19.178), S(19.233, 9.281), S(24.940, -6.159), S(8.430, -12.059),
        S(-9.344, -3.725), S(21.360, -12.105), S(17.928, -7.471), S(-6.310, 4.571), S(5.703, 4.090), S(27.384, -5.497), S(38.239, -11.982), S(2.490, -22.571),
        S(-25.066, -14.911), S(-17.108, -5.186), S(-16.583, -22.380), S(-26.843, -6.973), S(-19.252, -7.607), S(-21.585, -10.169), S(-18.075, -12.628), S(-31.683, -23.489),
    ],
    [
        S(25.123, 13.479), S(3.001, 17.342), S(-19.087, 26.355), S(-14.743, 20.038), S(-15.339, 16.008), S(-26.018, 16.723), S(5.879, 10.944), S(49.943, 2.239),
        S(-0.108, 20.310), S(1.378, 19.471), S(13.280, 14.669), S(8.464, 9.225), S(1.748, 5.386), S(-3.734, 6.222), S(0.210, 7.792), S(21.595, 4.703),
        S(-11.159, 14.970), S(-5.967, 11.530), S(-0.960, 4.795), S(8.301, -4.035), S(14.514, -12.895), S(2.597, -8.621), S(25.580, -8.315), S(22.166, -5.833),
        S(-18.657, 12.263), S(-8.782, 6.796), S(4.742, 2.004), S(10.948, -3.923), S(13.979, -15.342), S(9.785, -13.745), S(17.588, -12.489), S(21.447, -10.424),
        S(-21.712, 8.812), S(-20.292, 5.645), S(1.016, -1.426), S(8.808, -4.931), S(13.915, -11.894), S(2.711, -9.830), S(15.428, -17.239), S(4.929, -10.559),
        S(-29.107, 7.166), S(-11.962, -2.052), S(-2.716, -5.200), S(1.200, -7.926), S(12.021, -12.782), S(7.216, -15.152), S(35.479, -28.521), S(19.224, -22.557),
        S(-45.986, 5.912), S(-27.627, 0.296), S(-7.280, -3.440), S(-3.461, -4.356), S(0.849, -12.806), S(3.545, -15.946), S(20.286, -24.877), S(-19.606, -9.938),
        S(-0.286, -9.644), S(-14.073, 0.340), S(1.187, 0.980), S(15.225, -4.549), S(15.084, -12.608), S(8.712, -15.957), S(18.661, -19.144), S(8.691, -20.272),
    ],
    [
        S(-8.030, -0.546), S(-27.938, 15.732), S(-34.699, 24.768), S(-56.055, 34.630), S(-38.645, 28.053), S(-53.375, 29.692), S(-11.401, 6.591), S(13.188, -3.755),
        S(-22.062, -0.220), S(-16.759, 11.671), S(-16.091, 16.487), S(-49.369, 36.597), S(-49.264, 44.529), S(-33.282, 25.007), S(-31.030, 20.371), S(5.958, 0.086),
        S(-10.267, -7.816), S(4.783, -4.993), S(-9.286, 12.495), S(4.335, 6.244), S(1.592, 15.647), S(8.024, 13.822), S(28.623, -6.078), S(17.675, -11.278),
        S(-14.403, -1.041), S(-0.842, 4.861), S(-0.631, 7.692), S(6.653, 9.373), S(13.617, 13.538), S(7.717, 14.900), S(22.134, 6.148), S(19.864, -9.039),
        S(-4.671, -7.225), S(-3.070, 4.937), S(7.764, 2.148), S(20.363, 3.560), S(18.315, 5.015), S(18.392, 1.534), S(13.042, 3.735), S(18.323, -10.982),
        S(-9.390, -4.168), S(11.131, -8.198), S(19.154, -8.140), S(12.223, 0.741), S(18.730, 2.033), S(23.040, -3.091), S(36.511, -18.322), S(19.199, -18.666),
        S(-22.982, -3.060), S(5.250, -14.726), S(18.262, -23.806), S(11.797, -5.152), S(14.783, -9.311), S(26.802, -19.429), S(25.949, -15.815), S(13.298, -30.304),
        S(-18.870, -4.428), S(-39.962, 4.040), S(-15.213, -12.106), S(17.928, -71.536), S(-2.405, -21.637), S(-18.411, -6.594), S(-11.934, -7.783), S(-29.140, -17.030),
    ],
    [
        S(31.189, -20.795), S(33.014, -11.186), S(38.068, -9.071), S(10.111, -2.149), S(7.705, -2.231), S(50.657, -0.784), S(59.067, -3.828), S(57.294, -19.753),
        S(23.939, -12.374), S(32.587, 13.063), S(24.212, 13.228), S(15.177, 15.523), S(4.324, 23.700), S(28.573, 28.110), S(67.112, 17.323), S(49.057, 0.898),
        S(-12.842, 1.045), S(15.107, 20.632), S(-17.279, 30.989), S(-41.089, 44.303), S(-32.026, 50.032), S(21.944, 43.944), S(38.135, 33.647), S(-7.405, 14.252),
        S(-35.391, -1.000), S(-35.086, 24.087), S(-54.190, 38.965), S(-95.420, 54.845), S(-86.967, 55.493), S(-73.004, 52.356), S(-50.397, 37.106), S(-73.837, 16.158),
        S(-47.008, -5.207), S(-42.155, 14.789), S(-72.228, 34.095), S(-94.403, 48.058), S(-102.617, 50.644), S(-81.490, 40.840), S(-64.365, 24.737), S(-79.588, 5.011),
        S(-15.121, -18.937), S(-3.787, 1.483), S(-45.379, 18.613), S(-58.422, 28.438), S(-61.600, 31.410), S(-57.040, 26.085), S(-26.732, 7.708), S(-23.707, -15.859),
        S(31.495, -36.842), S(10.633, -13.907), S(-5.326, -2.122), S(-34.781, 5.885), S(-33.455, 8.401), S(-12.419, -1.208), S(17.965, -16.703), S(10.134, -33.100),
        S(-14.803, -64.500), S(56.558, -55.320), S(33.586, -41.223), S(-19.163, -36.784), S(48.033, -55.241), S(-2.501, -43.276), S(42.701, -52.222), S(3.537, -71.421),
    ],
];
#[rustfmt::skip]
const PASSED_PAWN_PUSH: [(f32, f32); 8] = [
    S(0.000, 0.000), S(0.000, 0.000), S(-0.351, 0.227), S(-0.248, 0.704), S(0.015, 0.951), S(0.294, 0.941), S(0.535, 0.856), S(-0.838, 2.444)
];
#[rustfmt::skip]
const THREAT: [[f32; 5]; 5] = [
    [-0.896, 0.784, 0.760, 0.279, 0.666],
    [0.082, -0.057, 0.755, 0.620, 0.710],
    [0.029, 0.585, -0.104, 0.673, 0.542],
    [0.276, 0.474, 0.522, 0.008, 1.152],
    [0.063, 0.250, 0.091, 0.097, -0.175],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [0.225, -2.545];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.909;
#[rustfmt::skip]
const DIRECT_CHECK_BONUS: f32 = 0.669;
#[rustfmt::skip]
const DISCOVERED_CHECK_BONUS: f32 = 0.819;

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

    fn direct_check_bonus() -> Self::Value {
        DIRECT_CHECK_BONUS
    }

    fn discovered_check_bonus() -> Self::Value {
        DISCOVERED_CHECK_BONUS
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

    let direct_check_bonus = if board.gives_direct_check(mv) {
        Params::direct_check_bonus()
    } else {
        Params::Value::default()
    };

    let discovered_check_bonus = if board.gives_discovered_check(mv) {
        Params::discovered_check_bonus()
    } else {
        Params::Value::default()
    };

    cap_bonus
        + promo_bonus
        + threat_evasion
        + bad_see_penalty
        + direct_check_bonus
        + discovered_check_bonus
        + pawn_score
        - pawn_protected_penalty
        + psqt / 50.0
        + threat_score
}
