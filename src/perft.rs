use crate::chess::{
    movegen::{movegen, MoveList},
    Board,
};
use std::time::Instant;

fn perft<const ROOT: bool>(board: &Board, depth: i32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0u64;

    let mut moves = MoveList::new();
    movegen(board, &mut moves);
    if !ROOT && depth == 1 {
        return moves.len() as u64;
    }

    for mv in moves {
        let mut new_board = board.clone();
        new_board.make_move(mv);
        let sub_nodes = perft::<false>(&new_board, depth - 1);
        if ROOT {
            println!("{}: {}", mv, sub_nodes);
        }
        nodes += sub_nodes
    }

    if ROOT {
        println!("total nodes: {}", nodes);
    }

    nodes
}

fn test_zobrist_key(board: &Board, depth: i32) {
    assert!(board.zkey() == board.recompute_zkey());

    if depth == 0 {
        return;
    }

    let mut moves = MoveList::new();
    movegen(board, &mut moves);

    for mv in moves {
        let mut new_board = board.clone();
        new_board.make_move(mv);
        test_zobrist_key(&new_board, depth - 1);
    }
}

struct PerftTest {
    fen: &'static str,
    depths: [u64; 6],
}

pub fn run_perft_tests() {
    #[rustfmt::skip]
    let perft_tests = [
        PerftTest { fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", depths: [20, 400, 8902, 197281, 4865609, 119060324] },
        PerftTest { fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", depths: [48, 2039, 97862, 4085603, 193690690, 8031647685] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/4K2R w K - 0 1", depths: [15, 66, 1197, 7059, 133987, 764643] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1", depths: [16, 71, 1287, 7626, 145232, 846648] },
        PerftTest { fen: "4k2r/8/8/8/8/8/8/4K3 w k - 0 1", depths: [5, 75, 459, 8290, 47635, 899442] },
        PerftTest { fen: "r3k3/8/8/8/8/8/8/4K3 w q - 0 1", depths: [5, 80, 493, 8897, 52710, 1001523] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1", depths: [26, 112, 3189, 17945, 532933, 2788982] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1", depths: [5, 130, 782, 22180, 118882, 3517770] },
        PerftTest { fen: "8/8/8/8/8/8/6k1/4K2R w K - 0 1", depths: [12, 38, 564, 2219, 37735, 185867] },
        PerftTest { fen: "8/8/8/8/8/8/1k6/R3K3 w Q - 0 1", depths: [15, 65, 1018, 4573, 80619, 413018] },
        PerftTest { fen: "4k2r/6K1/8/8/8/8/8/8 w k - 0 1", depths: [3, 32, 134, 2073, 10485, 179869] },
        PerftTest { fen: "r3k3/1K6/8/8/8/8/8/8 w q - 0 1", depths: [4, 49, 243, 3991, 20780, 367724] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", depths: [26, 568, 13744, 314346, 7594526, 179862938] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1", depths: [25, 567, 14095, 328965, 8153719, 195629489] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1", depths: [25, 548, 13502, 312835, 7736373, 184411439] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1", depths: [25, 547, 13579, 316214, 7878456, 189224276] },
        PerftTest { fen: "1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1", depths: [26, 583, 14252, 334705, 8198901, 198328929] },
        PerftTest { fen: "2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1", depths: [25, 560, 13592, 317324, 7710115, 185959088] },
        PerftTest { fen: "r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1", depths: [25, 560, 13607, 320792, 7848606, 190755813] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/4K2R b K - 0 1", depths: [5, 75, 459, 8290, 47635, 899442] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/R3K3 b Q - 0 1", depths: [5, 80, 493, 8897, 52710, 1001523] },
        PerftTest { fen: "4k2r/8/8/8/8/8/8/4K3 b k - 0 1", depths: [15, 66, 1197, 7059, 133987, 764643] },
        PerftTest { fen: "r3k3/8/8/8/8/8/8/4K3 b q - 0 1", depths: [16, 71, 1287, 7626, 145232, 846648] },
        PerftTest { fen: "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1", depths: [5, 130, 782, 22180, 118882, 3517770] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1", depths: [26, 112, 3189, 17945, 532933, 2788982] },
        PerftTest { fen: "8/8/8/8/8/8/6k1/4K2R b K - 0 1", depths: [3, 32, 134, 2073, 10485, 179869] },
        PerftTest { fen: "8/8/8/8/8/8/1k6/R3K3 b Q - 0 1", depths: [4, 49, 243, 3991, 20780, 367724] },
        PerftTest { fen: "4k2r/6K1/8/8/8/8/8/8 b k - 0 1", depths: [12, 38, 564, 2219, 37735, 185867] },
        PerftTest { fen: "r3k3/1K6/8/8/8/8/8/8 b q - 0 1", depths: [15, 65, 1018, 4573, 80619, 413018] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", depths: [26, 568, 13744, 314346, 7594526, 179862938] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1", depths: [26, 583, 14252, 334705, 8198901, 198328929] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1", depths: [25, 560, 13592, 317324, 7710115, 185959088] },
        PerftTest { fen: "r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1", depths: [25, 560, 13607, 320792, 7848606, 190755813] },
        PerftTest { fen: "1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1", depths: [25, 567, 14095, 328965, 8153719, 195629489] },
        PerftTest { fen: "2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1", depths: [25, 548, 13502, 312835, 7736373, 184411439] },
        PerftTest { fen: "r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1", depths: [25, 547, 13579, 316214, 7878456, 189224276] },
        PerftTest { fen: "8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1", depths: [14, 195, 2760, 38675, 570726, 8107539] },
        PerftTest { fen: "8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1", depths: [11, 156, 1636, 20534, 223507, 2594412] },
        PerftTest { fen: "8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1", depths: [19, 289, 4442, 73584, 1198299, 19870403] },
        PerftTest { fen: "K7/8/2n5/1n6/8/8/8/k6N w - - 0 1", depths: [3, 51, 345, 5301, 38348, 588695] },
        PerftTest { fen: "k7/8/2N5/1N6/8/8/8/K6n w - - 0 1", depths: [17, 54, 835, 5910, 92250, 688780] },
        PerftTest { fen: "8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1", depths: [15, 193, 2816, 40039, 582642, 8503277] },
        PerftTest { fen: "8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1", depths: [16, 180, 2290, 24640, 288141, 3147566] },
        PerftTest { fen: "8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1", depths: [4, 68, 1118, 16199, 281190, 4405103] },
        PerftTest { fen: "K7/8/2n5/1n6/8/8/8/k6N b - - 0 1", depths: [17, 54, 835, 5910, 92250, 688780] },
        PerftTest { fen: "k7/8/2N5/1N6/8/8/8/K6n b - - 0 1", depths: [3, 51, 345, 5301, 38348, 588695] },
        PerftTest { fen: "B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1", depths: [17, 278, 4607, 76778, 1320507, 22823890] },
        PerftTest { fen: "8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1", depths: [21, 316, 5744, 93338, 1713368, 28861171] },
        PerftTest { fen: "k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1", depths: [21, 144, 3242, 32955, 787524, 7881673] },
        PerftTest { fen: "K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1", depths: [7, 143, 1416, 31787, 310862, 7382896] },
        PerftTest { fen: "B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1", depths: [6, 106, 1829, 31151, 530585, 9250746] },
        PerftTest { fen: "8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1", depths: [17, 309, 5133, 93603, 1591064, 29027891] },
        PerftTest { fen: "k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1", depths: [7, 143, 1416, 31787, 310862, 7382896] },
        PerftTest { fen: "K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1", depths: [21, 144, 3242, 32955, 787524, 7881673] },
        PerftTest { fen: "7k/RR6/8/8/8/8/rr6/7K w - - 0 1", depths: [19, 275, 5300, 104342, 2161211, 44956585] },
        PerftTest { fen: "R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1", depths: [36, 1027, 29215, 771461, 20506480, 525169084] },
        PerftTest { fen: "7k/RR6/8/8/8/8/rr6/7K b - - 0 1", depths: [19, 275, 5300, 104342, 2161211, 44956585] },
        PerftTest { fen: "R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1", depths: [36, 1027, 29227, 771368, 20521342, 524966748] },
        PerftTest { fen: "6kq/8/8/8/8/8/8/7K w - - 0 1", depths: [2, 36, 143, 3637, 14893, 391507] },
        PerftTest { fen: "6KQ/8/8/8/8/8/8/7k b - - 0 1", depths: [2, 36, 143, 3637, 14893, 391507] },
        PerftTest { fen: "K7/8/8/3Q4/4q3/8/8/7k w - - 0 1", depths: [6, 35, 495, 8349, 166741, 3370175] },
        PerftTest { fen: "6qk/8/8/8/8/8/8/7K b - - 0 1", depths: [22, 43, 1015, 4167, 105749, 419369] },
        PerftTest { fen: "6KQ/8/8/8/8/8/8/7k b - - 0 1", depths: [2, 36, 143, 3637, 14893, 391507] },
        PerftTest { fen: "K7/8/8/3Q4/4q3/8/8/7k b - - 0 1", depths: [6, 35, 495, 8349, 166741, 3370175] },
        PerftTest { fen: "8/8/8/8/8/K7/P7/k7 w - - 0 1", depths: [3, 7, 43, 199, 1347, 6249] },
        PerftTest { fen: "8/8/8/8/8/7K/7P/7k w - - 0 1", depths: [3, 7, 43, 199, 1347, 6249] },
        PerftTest { fen: "K7/p7/k7/8/8/8/8/8 w - - 0 1", depths: [1, 3, 12, 80, 342, 2343] },
        PerftTest { fen: "7K/7p/7k/8/8/8/8/8 w - - 0 1", depths: [1, 3, 12, 80, 342, 2343] },
        PerftTest { fen: "8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1", depths: [7, 35, 210, 1091, 7028, 34834] },
        PerftTest { fen: "8/8/8/8/8/K7/P7/k7 b - - 0 1", depths: [1, 3, 12, 80, 342, 2343] },
        PerftTest { fen: "8/8/8/8/8/7K/7P/7k b - - 0 1", depths: [1, 3, 12, 80, 342, 2343] },
        PerftTest { fen: "K7/p7/k7/8/8/8/8/8 b - - 0 1", depths: [3, 7, 43, 199, 1347, 6249] },
        PerftTest { fen: "7K/7p/7k/8/8/8/8/8 b - - 0 1", depths: [3, 7, 43, 199, 1347, 6249] },
        PerftTest { fen: "8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1", depths: [5, 35, 182, 1091, 5408, 34822] },
        PerftTest { fen: "8/8/8/8/8/4k3/4P3/4K3 w - - 0 1", depths: [2, 8, 44, 282, 1814, 11848] },
        PerftTest { fen: "4k3/4p3/4K3/8/8/8/8/8 b - - 0 1", depths: [2, 8, 44, 282, 1814, 11848] },
        PerftTest { fen: "8/8/7k/7p/7P/7K/8/8 w - - 0 1", depths: [3, 9, 57, 360, 1969, 10724] },
        PerftTest { fen: "8/8/k7/p7/P7/K7/8/8 w - - 0 1", depths: [3, 9, 57, 360, 1969, 10724] },
        PerftTest { fen: "8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1", depths: [5, 25, 180, 1294, 8296, 53138] },
        PerftTest { fen: "8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1", depths: [8, 61, 483, 3213, 23599, 157093] },
        PerftTest { fen: "8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1", depths: [8, 61, 411, 3213, 21637, 158065] },
        PerftTest { fen: "k7/8/3p4/8/3P4/8/8/7K w - - 0 1", depths: [4, 15, 90, 534, 3450, 20960] },
        PerftTest { fen: "8/8/7k/7p/7P/7K/8/8 b - - 0 1", depths: [3, 9, 57, 360, 1969, 10724] },
        PerftTest { fen: "8/8/k7/p7/P7/K7/8/8 b - - 0 1", depths: [3, 9, 57, 360, 1969, 10724] },
        PerftTest { fen: "8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1", depths: [5, 25, 180, 1294, 8296, 53138] },
        PerftTest { fen: "8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1", depths: [8, 61, 411, 3213, 21637, 158065] },
        PerftTest { fen: "8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1", depths: [8, 61, 483, 3213, 23599, 157093] },
        PerftTest { fen: "k7/8/3p4/8/3P4/8/8/7K b - - 0 1", depths: [4, 15, 89, 537, 3309, 21104] },
        PerftTest { fen: "7k/3p4/8/8/3P4/8/8/K7 w - - 0 1", depths: [4, 19, 117, 720, 4661, 32191] },
        PerftTest { fen: "7k/8/8/3p4/8/8/3P4/K7 w - - 0 1", depths: [5, 19, 116, 716, 4786, 30980] },
        PerftTest { fen: "k7/8/8/7p/6P1/8/8/K7 w - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "k7/8/7p/8/8/6P1/8/K7 w - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/8/8/6p1/7P/8/8/K7 w - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "k7/8/6p1/8/8/7P/8/K7 w - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/8/8/3p4/4p3/8/8/7K w - - 0 1", depths: [3, 15, 84, 573, 3013, 22886] },
        PerftTest { fen: "k7/8/3p4/8/8/4P3/8/7K w - - 0 1", depths: [4, 16, 101, 637, 4271, 28662] },
        PerftTest { fen: "7k/3p4/8/8/3P4/8/8/K7 b - - 0 1", depths: [5, 19, 117, 720, 5014, 32167] },
        PerftTest { fen: "7k/8/8/3p4/8/8/3P4/K7 b - - 0 1", depths: [4, 19, 117, 712, 4658, 30749] },
        PerftTest { fen: "k7/8/8/7p/6P1/8/8/K7 b - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "k7/8/7p/8/8/6P1/8/K7 b - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/8/8/6p1/7P/8/8/K7 b - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "k7/8/6p1/8/8/7P/8/K7 b - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/8/8/3p4/4p3/8/8/7K b - - 0 1", depths: [5, 15, 102, 569, 4337, 22579] },
        PerftTest { fen: "k7/8/3p4/8/8/4P3/8/7K b - - 0 1", depths: [4, 16, 101, 637, 4271, 28662] },
        PerftTest { fen: "7k/8/8/p7/1P6/8/8/7K w - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "7k/8/p7/8/8/1P6/8/7K w - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "7k/8/8/1p6/P7/8/8/7K w - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "7k/8/1p6/8/8/P7/8/7K w - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/7p/8/8/8/8/6P1/K7 w - - 0 1", depths: [5, 25, 161, 1035, 7574, 55338] },
        PerftTest { fen: "k7/6p1/8/8/8/8/7P/K7 w - - 0 1", depths: [5, 25, 161, 1035, 7574, 55338] },
        PerftTest { fen: "3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1", depths: [7, 49, 378, 2902, 24122, 199002] },
        PerftTest { fen: "7k/8/8/p7/1P6/8/8/7K b - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "7k/8/p7/8/8/1P6/8/7K b - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "7k/8/8/1p6/P7/8/8/7K b - - 0 1", depths: [5, 22, 139, 877, 6112, 41874] },
        PerftTest { fen: "7k/8/1p6/8/8/P7/8/7K b - - 0 1", depths: [4, 16, 101, 637, 4354, 29679] },
        PerftTest { fen: "k7/7p/8/8/8/8/6P1/K7 b - - 0 1", depths: [5, 25, 161, 1035, 7574, 55338] },
        PerftTest { fen: "k7/6p1/8/8/8/8/7P/K7 b - - 0 1", depths: [5, 25, 161, 1035, 7574, 55338] },
        PerftTest { fen: "3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1", depths: [7, 49, 378, 2902, 24122, 199002] },
        PerftTest { fen: "8/Pk6/8/8/8/8/6Kp/8 w - - 0 1", depths: [11, 97, 887, 8048, 90606, 1030499] },
        PerftTest { fen: "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1", depths: [24, 421, 7421, 124608, 2193768, 37665329] },
        PerftTest { fen: "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1", depths: [18, 270, 4699, 79355, 1533145, 28859283] },
        PerftTest { fen: "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1", depths: [24, 496, 9483, 182838, 3605103, 71179139] },
        PerftTest { fen: "8/Pk6/8/8/8/8/6Kp/8 b - - 0 1", depths: [11, 97, 887, 8048, 90606, 1030499] },
        PerftTest { fen: "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1", depths: [24, 421, 7421, 124608, 2193768, 37665329] },
        PerftTest { fen: "8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1", depths: [18, 270, 4699, 79355, 1533145, 28859283] },
        PerftTest { fen: "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1", depths: [24, 496, 9483, 182838, 3605103, 71179139] },
    ];

    let mut passed = 0;
    let mut failed = 0;

    let start = Instant::now();
    let mut total_nodes = 0u64;

    for test in perft_tests {
        let board = Board::from_fen(test.fen).unwrap();
        println!("fen: {}", test.fen);
        for d in 1..=6 {
            // skip the ones that take really long
            if test.depths[(d - 1) as usize] > 100_000_000 {
                continue;
            }
            let nodes = perft::<false>(&board, d);
            total_nodes += nodes;
            if test.depths[(d - 1) as usize] == nodes {
                passed += 1;
                println!("    passed depth {}", d);
            } else {
                failed += 1;
                println!(
                    "    failed depth {} expected {} got {}",
                    d,
                    test.depths[(d - 1) as usize],
                    nodes
                );
            }
        }
    }

    println!("nodes: {}", total_nodes);
    println!("time: {}", start.elapsed().as_secs_f64());
    println!(
        "nps: {}",
        total_nodes as f64 / start.elapsed().as_secs_f64()
    );
    println!("passed {} out of {}", passed, failed + passed);
}
