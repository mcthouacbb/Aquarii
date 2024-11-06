mod types;
mod chess;

use chess::Board;

fn main() {
    let mut board = Board::startpos();
    println!("{}\n", board);

    board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    println!("{}", board);
}
