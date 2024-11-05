mod types;
mod chess;

use types::{Bitboard, PieceType, Color, Piece, Square};
use chess::Board;

fn main() {
    println!("{} {} {} {} {} {}", PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen, PieceType::King);
    println!("{} {}", Color::White, Color::Black);
    let piece = Piece::new(Color::Black, PieceType::Queen);
    println!("{} {} {}", piece, piece.color(), piece.piece_type());
    println!("{} {}", !Color::White, !Color::Black);

    let mut bb = Bitboard::FILE_C | Bitboard::FILE_E;
    let mut bb2 = !bb ^ Bitboard::RANK_3;

    println!("{}\n{}\n", bb, bb2);
    while bb.any() {
        let sq = bb.poplsb();
        println!("{}", sq);
    }

    while bb2.any() {
        let sq = bb2.poplsb();
        println!("{}", sq);
    }

    let sq = Square::from(2);
    println!("{}", sq);

    let mut board = Board::startpos();
    println!("{}\n", board);

    board = Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1").unwrap();
    println!("{}", board);
}
