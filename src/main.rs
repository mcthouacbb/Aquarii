use std::{env, io, str::SplitWhitespace};

mod bench;
mod chess;
mod datagen;
mod eval;
mod perft;
mod policy;
mod position;
mod search;
mod tune;
mod types;

use bench::run_bench;
use chess::{
    movegen::{movegen, MoveList},
    Board, Move, MoveKind,
};
use position::Position;
use search::SearchLimits;
use types::Color;

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

fn parse_position(tokens: &mut SplitWhitespace, position: &mut Position) {
    match tokens.next() {
        Some("fen") => {
            let fen = tokens.clone().take(6).collect::<Vec<&str>>().join(" ");
            tokens.nth(5);
            if !position.parse_fen(fen.as_str()) {
                println!("info string invalid fen");
                return;
            }
        }
        Some("startpos") => {
            position.set_startpos();
        }
        _ => {
            println!("info string invalid position");
        }
    }

    if tokens.next() == Some("moves") {
        while let Some(mv_str) = tokens.next() {
            let Some(mv) = move_from_str(position.board(), mv_str) else {
                println!("invalid move {}", mv_str);
                return;
            };
            position.make_move(mv);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 && args[1] == "bench" {
        run_bench();
        return;
    }

    if args.len() == 2 && args[1] == "datagen" {
        datagen::run_datagen();
        return;
    }

    if args.len() >= 2 && args[1] == "tunepolicy" {
        tune::policy::main(&args[2..args.len()]);
        return;
    }

    if args.len() >= 2 && args[1] == "tuneeval" {
        tune::eval::main(&args[2..args.len()]);
        return;
    }

    let mut pos = Position::new();
    let mut searcher = search::MCTS::new(1000000);
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
                println!("option name Threads type spin default 1 min 1 max 1");
                println!("option name Hash type spin default 1 min 1 max 1");
                println!("uciok");
            }
            Some("ucinewgame") => {
                // does nothing for now
                searcher.new_game();
            }
            Some("isready") => {
                println!("readyok");
            }
            Some("position") => {
                parse_position(&mut tokens, &mut pos);
            }
            Some("bench") => {
                run_bench();
            }
            Some("d") => {
                println!("{}", pos.board());
            }
            Some("go") => {
                let mut limits = SearchLimits::new();
                loop {
                    match tokens.next() {
                        Some("infinite") => {
                            limits = SearchLimits::new();
                            break;
                        }
                        Some("nodes") => {
                            if let Some(nodes_str) = tokens.next() {
                                if let Ok(nodes) = nodes_str.parse::<i32>() {
                                    limits.max_nodes = nodes;
                                }
                            }
                        }
                        Some("movetime") => {
                            if let Some(time_str) = tokens.next() {
                                if let Ok(time) = time_str.parse::<i32>() {
                                    limits.max_time = time;
                                }
                            }
                        }
                        Some("depth") => {
                            if let Some(depth_str) = tokens.next() {
                                if let Ok(depth) = depth_str.parse::<i32>() {
                                    limits.max_depth = depth;
                                }
                            }
                        }
                        Some("wtime") => {
                            if let Some(time_str) = tokens.next() {
                                if let Ok(time) = time_str.parse::<i32>() {
                                    if pos.board().stm() == Color::White {
                                        limits.use_clock = true;
                                        limits.time = time;
                                    }
                                }
                            }
                        }
                        Some("btime") => {
                            if let Some(time_str) = tokens.next() {
                                if let Ok(time) = time_str.parse::<i32>() {
                                    if pos.board().stm() == Color::Black {
                                        limits.use_clock = true;
                                        limits.time = time;
                                    }
                                }
                            }
                        }
                        Some("winc") => {
                            if let Some(inc_str) = tokens.next() {
                                if let Ok(inc) = inc_str.parse::<i32>() {
                                    if pos.board().stm() == Color::White {
                                        limits.use_clock = true;
                                        limits.inc = inc;
                                    }
                                }
                            }
                        }
                        Some("binc") => {
                            if let Some(inc_str) = tokens.next() {
                                if let Ok(inc) = inc_str.parse::<i32>() {
                                    if pos.board().stm() == Color::Black {
                                        limits.use_clock = true;
                                        limits.inc = inc;
                                    }
                                }
                            }
                        }
                        _ => {
                            break;
                        }
                    }
                }
                let results: search::SearchResults = searcher.run(limits, true, &pos);
                println!("bestmove {}", results.best_move);
            }
            Some("tree") => {
                searcher.display_tree();
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
