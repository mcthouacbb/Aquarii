use std::{io, str::SplitWhitespace};

mod chess;
mod perft;
mod types;

use chess::{
    movegen::{movegen, MoveList},
    Board, Move, MoveKind,
};
use rand::Rng;

fn move_from_str(board: &Board, mv_str: &str) -> Option<Move> {
    let parsed = mv_str.parse::<Move>().unwrap_or(Move::NULL);

    let mut moves = MoveList::new();
    movegen(board, &mut moves);
    for candidate in moves {
        if candidate.from_sq() == parsed.from_sq()
            && candidate.to_sq() == parsed.to_sq()
            && (candidate.kind() != MoveKind::Promotion
                || candidate.promo_piece() == parsed.promo_piece())
        {
            return Some(candidate);
        }
        if candidate.from_sq() == parsed.from_sq()
            && candidate.kind() == MoveKind::Castle
            && (candidate.to_sq() > candidate.from_sq()) == (parsed.to_sq() > parsed.from_sq())
        {
            return Some(candidate);
        }
    }
    None
}

fn parse_position(tokens: &mut SplitWhitespace, board: &mut Board) {
    match tokens.next() {
        Some("fen") => {
            let fen = tokens.clone().take(6).collect::<Vec<&str>>().join(" ");
            tokens.nth(5);
            if let Some(fen_board) = Board::from_fen(fen.as_str()) {
                *board = fen_board;
            } else {
                println!("info string invalid fen");
                return;
            }
        }
        Some("startpos") => {
            *board = Board::startpos();
        }
        _ => {
            println!("info string invalid position");
        }
    }

    if tokens.next() == Some("moves") {
        while let Some(mv_str) = tokens.next() {
            let Some(mv) = move_from_str(board, mv_str) else {
                println!("invalid move {}", mv_str);
                return;
            };
            board.make_move(mv);
        }
    }
}

fn main() {
    let mut board = Board::startpos();
    let mut rng = rand::thread_rng();
    loop {
        let mut cmd = String::new();

        io::stdin()
            .read_line(&mut cmd)
            .expect("Failed to read line");

        let mut tokens = cmd.split_whitespace();

        match tokens.next() {
            Some("uci") => {
                println!("id name Aquarii");
                println!("id author Mcthouacbb");
                println!("uciok");
            }
            Some("ucinewgame") => {
                // does nothing for now
            }
            Some("isready") => {
                println!("readyok");
            }
            Some("position") => {
                parse_position(&mut tokens, &mut board);
                println!("{}", board);
            }
            Some("go") => {
                let mut moves = MoveList::new();
                movegen(&board, &mut moves);
                let mv = moves[rng.gen_range(0..moves.len())];
                println!("bestmove {}", mv);
            }
            Some("quit") => {
                return;
            }
            _ => {
                println!("info string invalid command");
            }
        }
    }
}
