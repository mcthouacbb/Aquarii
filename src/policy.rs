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
    fn threat(defended: bool, moving: PieceType, threatened: PieceType) -> Self::Value;
    fn promo_bonus(pt: PieceType) -> Self::Value;
    fn bad_see_penalty() -> Self::Value;
    fn check_bonus() -> Self::Value;
}

#[allow(non_snake_case)]
const fn S(mg: f32, eg: f32) -> (f32, f32) {
    (mg, eg)
}

#[rustfmt::skip]
const CAP_BONUS: [f32; 5] = [1.597, 2.654, 2.796, 2.758, 3.604];
#[rustfmt::skip]
const PAWN_PROTECTED_PENALTY: [f32; 5] = [-0.372, 2.268, 1.916, 2.968, 3.292];
#[rustfmt::skip]
const THREAT_EVASION: [[f32; 5]; 5] = [
    [0.351, 2.625, 2.322, 2.360, 2.982],
    [0.319, 0.161, 1.278, 1.885, 2.403],
    [0.174, 0.525, 0.281, 1.821, 2.439],
    [0.213, 0.575, 0.641, 0.522, 2.399],
    [0.043, 0.321, 0.482, 0.576, 0.686],
];
#[rustfmt::skip]
const PSQT_SCORE: [[(f32, f32); 64]; 6] = [
    [
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
        S(131.723, 150.745), S(67.334, 141.712), S(93.688, 133.897), S(96.427, 153.996), S(74.618, 135.051), S(64.710, 120.151), S(40.467, 128.675), S(83.532, 158.409),
        S(34.726, 58.080), S(44.050, 46.512), S(42.725, 49.875), S(66.564, 55.927), S(67.446, 37.341), S(66.561, 28.375), S(54.913, 30.585), S(43.541, 54.800),
        S(-19.954, -27.436), S(29.531, -31.907), S(21.728, -26.574), S(37.548, -16.480), S(46.945, -26.062), S(35.454, -33.374), S(46.815, -43.974), S(-5.394, -29.400),
        S(-38.258, -92.138), S(-20.148, -72.417), S(-1.112, -70.803), S(20.996, -57.218), S(11.258, -58.392), S(5.580, -68.606), S(-4.207, -71.277), S(-26.207, -83.910),
        S(-17.746, -118.219), S(-27.209, -75.874), S(-1.272, -87.033), S(-15.820, -57.338), S(-2.074, -61.614), S(-10.111, -72.518), S(8.804, -78.592), S(3.287, -104.758),
        S(-29.747, -98.185), S(-25.925, -69.250), S(-38.088, -61.520), S(-65.741, -32.070), S(-50.472, -32.840), S(-4.644, -59.461), S(11.819, -69.275), S(-23.384, -85.953),
        S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000), S(0.000, 0.000),
    ],
    [
        S(24.992, -71.351), S(-1.112, -28.335), S(-49.642, 0.296), S(24.826, -17.872), S(12.459, -11.103), S(-9.751, -22.121), S(-10.077, -22.423), S(30.222, -69.535),
        S(-26.263, -19.501), S(-16.990, -2.756), S(-2.748, 8.733), S(-10.551, 12.239), S(-28.550, 19.874), S(9.369, 3.568), S(-16.072, -9.348), S(2.245, -37.172),
        S(-15.030, -4.640), S(16.230, 6.931), S(29.128, 16.453), S(20.894, 24.425), S(23.819, 18.246), S(33.765, 11.602), S(39.058, -2.883), S(17.450, -14.948),
        S(8.263, -7.198), S(17.747, 17.550), S(32.304, 22.325), S(45.963, 22.749), S(50.823, 22.235), S(43.804, 25.157), S(30.060, 14.338), S(44.346, -16.814),
        S(-6.090, -4.120), S(16.267, 11.045), S(24.864, 27.272), S(41.895, 20.781), S(47.516, 21.922), S(39.627, 21.640), S(35.997, 10.817), S(8.844, -14.661),
        S(-40.676, -10.091), S(4.350, 2.565), S(31.388, 5.040), S(33.962, 17.273), S(43.121, 16.480), S(37.646, 2.851), S(32.761, -3.530), S(-27.946, -10.673),
        S(-56.779, -21.343), S(-34.339, -7.896), S(-2.606, 1.886), S(-4.664, 8.783), S(5.001, 4.722), S(17.710, -1.169), S(-0.643, -16.024), S(-31.945, -20.864),
        S(-96.560, -9.401), S(-58.941, -48.014), S(-56.493, -8.922), S(-35.443, -13.740), S(-34.499, -16.948), S(-17.588, -21.864), S(-55.353, -38.005), S(-63.002, -34.010),
    ],
    [
        S(4.222, -22.231), S(-62.889, 7.135), S(-2.927, -14.986), S(-10.032, -8.556), S(-21.353, -7.831), S(2.605, -19.104), S(-67.178, 0.352), S(5.901, -23.300),
        S(-39.191, -14.937), S(-11.938, -4.005), S(-20.975, 0.441), S(-73.312, 8.575), S(-51.469, 3.664), S(-25.852, -0.215), S(-22.669, -0.953), S(-8.645, -28.217),
        S(-24.277, -1.814), S(2.497, -0.189), S(-16.460, 12.852), S(9.494, 4.370), S(14.674, 4.410), S(0.789, 11.539), S(24.796, -3.412), S(-6.090, -4.764),
        S(-11.367, -6.433), S(4.106, 10.248), S(15.114, 8.562), S(17.359, 17.415), S(19.350, 15.476), S(21.037, 9.758), S(12.251, 4.318), S(10.920, -15.397),
        S(-4.401, -9.883), S(0.124, 9.858), S(7.670, 19.316), S(32.274, 13.407), S(31.071, 11.749), S(13.934, 15.714), S(8.055, 8.180), S(12.988, -22.254),
        S(5.504, -12.198), S(19.243, 0.304), S(22.582, 10.511), S(15.315, 11.223), S(13.960, 19.271), S(19.280, 9.850), S(24.191, -5.746), S(9.081, -11.894),
        S(-9.013, -3.374), S(20.976, -11.488), S(17.582, -7.004), S(-7.121, 5.036), S(5.242, 4.390), S(26.401, -5.272), S(38.376, -11.692), S(2.040, -21.789),
        S(-26.406, -14.209), S(-17.351, -4.570), S(-16.755, -23.061), S(-27.809, -6.420), S(-20.793, -7.081), S(-21.091, -10.829), S(-18.286, -12.368), S(-32.447, -22.620),
    ],
    [
        S(27.323, 13.783), S(6.458, 16.953), S(-15.282, 26.163), S(-10.545, 19.947), S(-11.155, 16.281), S(-21.533, 16.600), S(10.012, 10.960), S(52.733, 2.027),
        S(1.828, 20.800), S(4.853, 18.906), S(16.899, 13.940), S(13.387, 7.789), S(6.794, 4.570), S(3.089, 4.946), S(5.849, 6.977), S(24.695, 4.753),
        S(-10.463, 14.996), S(-3.558, 11.230), S(4.032, 3.669), S(15.804, -6.194), S(22.181, -14.978), S(11.507, -10.877), S(30.963, -9.051), S(25.326, -6.506),
        S(-20.216, 12.357), S(-7.693, 6.030), S(7.772, 0.804), S(14.864, -5.195), S(19.072, -16.671), S(15.241, -15.596), S(20.623, -13.220), S(21.458, -10.709),
        S(-23.391, 7.886), S(-21.487, 5.303), S(1.712, -2.250), S(10.114, -5.813), S(15.698, -12.697), S(4.927, -10.682), S(15.889, -17.399), S(4.679, -11.233),
        S(-32.286, 7.249), S(-14.165, -2.029), S(-4.471, -5.161), S(0.350, -8.320), S(11.309, -12.812), S(6.441, -14.947), S(33.932, -27.876), S(16.545, -22.414),
        S(-48.664, 5.777), S(-30.305, 0.741), S(-9.374, -3.132), S(-4.901, -4.473), S(-0.712, -12.569), S(2.004, -15.483), S(18.302, -24.292), S(-21.630, -9.880),
        S(-1.723, -10.071), S(-15.905, 1.555), S(-0.497, 1.932), S(13.938, -3.715), S(13.466, -11.275), S(7.288, -15.217), S(17.290, -17.895), S(7.935, -20.816),
    ],
    [
        S(-5.476, -0.717), S(-25.943, 16.076), S(-31.057, 24.685), S(-45.896, 31.398), S(-28.652, 24.574), S(-46.069, 28.035), S(-8.623, 6.970), S(15.319, -2.828),
        S(-23.226, 0.462), S(-14.967, 11.148), S(-12.236, 14.809), S(-42.643, 34.275), S(-40.966, 42.456), S(-24.599, 21.776), S(-24.988, 18.529), S(6.339, 0.609),
        S(-11.599, -7.019), S(5.345, -5.200), S(-6.723, 11.008), S(11.564, 2.716), S(8.320, 12.903), S(14.243, 11.547), S(31.384, -6.409), S(19.060, -10.938),
        S(-15.396, -0.543), S(-1.433, 5.015), S(1.578, 6.094), S(9.206, 8.359), S(15.312, 13.273), S(10.958, 13.820), S(24.373, 5.759), S(19.775, -8.148),
        S(-5.648, -7.633), S(-3.632, 4.970), S(7.346, 2.162), S(20.510, 3.338), S(18.121, 5.195), S(18.672, 1.542), S(12.775, 4.622), S(17.958, -10.529),
        S(-10.535, -4.023), S(9.146, -7.115), S(16.845, -7.144), S(10.255, 1.808), S(17.171, 2.351), S(21.343, -2.266), S(34.609, -17.089), S(18.198, -17.970),
        S(-25.669, -1.624), S(2.127, -12.749), S(15.487, -22.012), S(8.545, -3.303), S(11.494, -7.510), S(23.815, -18.596), S(23.647, -14.542), S(10.909, -28.820),
        S(-21.968, -1.985), S(-43.180, 6.320), S(-18.654, -9.762), S(15.361, -70.437), S(-5.943, -19.764), S(-21.718, -5.374), S(-14.232, -6.195), S(-30.998, -15.353),
    ],
    [
        S(33.869, -22.841), S(30.541, -12.392), S(27.642, -7.678), S(-5.082, 0.240), S(-6.401, -0.125), S(37.833, 0.984), S(53.132, -3.841), S(60.261, -22.078),
        S(15.204, -10.758), S(18.492, 15.372), S(7.185, 16.024), S(-3.868, 18.853), S(-13.792, 26.553), S(11.963, 30.871), S(50.698, 19.955), S(34.477, 3.509),
        S(-25.159, 3.226), S(0.622, 22.967), S(-32.278, 33.311), S(-57.068, 46.618), S(-48.965, 52.388), S(5.557, 46.424), S(22.221, 36.182), S(-19.678, 16.078),
        S(-48.238, 1.406), S(-49.301, 26.708), S(-67.752, 41.150), S(-108.727, 56.834), S(-99.213, 57.072), S(-85.391, 54.417), S(-63.236, 39.604), S(-86.551, 18.383),
        S(-52.065, -4.101), S(-47.263, 15.834), S(-76.682, 34.990), S(-100.368, 48.904), S(-107.561, 51.074), S(-85.245, 41.191), S(-68.352, 25.293), S(-83.259, 5.709),
        S(-13.929, -19.433), S(-2.007, 0.830), S(-43.675, 17.854), S(-55.940, 27.620), S(-59.214, 30.515), S(-54.981, 25.483), S(-24.624, 6.728), S(-22.256, -16.504),
        S(34.422, -37.821), S(13.527, -14.909), S(-1.576, -3.488), S(-30.317, 4.400), S(-29.007, 6.777), S(-8.572, -2.800), S(20.752, -17.594), S(12.570, -33.941),
        S(-13.538, -65.305), S(60.242, -56.489), S(37.832, -42.802), S(-14.399, -38.433), S(53.844, -57.669), S(1.646, -44.627), S(45.901, -53.861), S(4.671, -71.922),
    ],
];
#[rustfmt::skip]
const THREAT: [[[f32; 5]; 5]; 2] = [
    [
        [-0.961, 0.780, 0.765, 0.282, 0.684],
        [0.431, 0.212, 0.866, 0.706, 0.691],
        [0.184, 0.778, 0.115, 0.648, 0.701],
        [0.463, 0.734, 0.758, 0.121, 1.221],
        [0.297, 0.504, 0.482, 0.453, 0.036],
    ],
    [
        [-0.886, 0.822, 0.665, 0.549, 0.702],
        [-0.027, -0.066, 0.641, 0.556, 0.680],
        [-0.034, 0.511, -0.206, 0.711, 0.437],
        [0.035, 0.225, 0.316, -0.126, 0.960],
        [0.001, 0.116, -0.069, -0.062, -0.507],
    ],
];
#[rustfmt::skip]
const PROMO_BONUS: [f32; 2] = [1.229, -1.576];
#[rustfmt::skip]
const BAD_SEE_PENALTY: f32 = -2.938;
#[rustfmt::skip]
const CHECK_BONUS: f32 = 0.649;

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

    fn threat(defended: bool, moving: PieceType, threatened: PieceType) -> Self::Value {
        THREAT[defended as usize][moving as usize - PieceType::Pawn as usize][threatened as usize]
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
    attacked_by_2: Bitboard,
    defended: Bitboard,
}

impl PolicyData {
    pub fn new(board: &Board) -> Self {
        let mut result: PolicyData = Self {
            attacked: Bitboard::NONE,
            attacked_by: [Bitboard::NONE; 6],
            attacked_by_2: Bitboard::NONE,
            defended: Bitboard::NONE,
        };

        let stm = board.stm();

        result.add_defenses(attacks::pawn_attacks_bb(
            stm,
            board.colored_pieces(Piece::new(stm, PieceType::Pawn)),
        ));

        result.add_defenses(attacks::king_attacks(board.king_sq(stm)));

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
            let mut bb = board.colored_pieces(Piece::new(stm, pt));
            while bb.any() {
                let sq = bb.poplsb();
                let attacks = attacks::piece_attacks(pt, sq, board.occ());
                result.add_defenses(attacks);
            }

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
        self.attacked_by_2 |= self.attacked & attacks;
        self.attacked |= attacks;
        self.attacked_by[pt as usize] |= attacks;
    }

    fn add_defenses(&mut self, attacks: Bitboard) {
        self.defended |= attacks;
    }

    fn attacked(&self) -> Bitboard {
        self.attacked
    }

    fn attacked_by(&self, pt: PieceType) -> Bitboard {
        self.attacked_by[pt as usize]
    }

    fn attacked_by_2(&self) -> Bitboard {
        self.attacked_by_2
    }

    fn defended(&self) -> Bitboard {
        self.defended
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
            let defended_bb =
                data.attacked_by_2() | pawn_protected | (data.attacked() & !data.defended());
            let mut score = Params::Value::default();
            while threats.any() {
                let threat = threats.poplsb();
                let defended = defended_bb.has(threat);
                score += Params::threat(
                    defended,
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
