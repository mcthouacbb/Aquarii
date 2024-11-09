mod chess;
mod types;

use chess::{movegen::movegen, Board, Move};

fn perft<const ROOT: bool>(board: &Board, depth: i32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0u64;

    // !TODO
    let moves: Vec<Move> = movegen(board);

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

fn main() {
    let mut board = Board::startpos();
    println!("{}\n", board);

    perft::<true>(&board, 1);

    board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
        .unwrap();
    println!("{}", board);

    perft::<true>(&board, 1);
}
