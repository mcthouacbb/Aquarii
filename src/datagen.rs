use rand::Rng;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use crate::{chess::movegen::{self, MoveList}, position::Position, search::{GameResult, SearchLimits, MCTS}, types::Color};


pub fn run_datagen() {
	let mut search = MCTS::new(10000);
	let seed = rand::rng().next_u64();
	println!("RNG seed: {}", seed);

	let mut rng = XorShiftRng::seed_from_u64(seed);
	loop {
		run_game(&mut search, &mut rng);

		// temporary
		break;
	}
}

fn game_result(pos: &Position) -> GameResult {
	let mut moves = MoveList::new();
	movegen::movegen(pos.board(), &mut moves);

	if moves.len() == 0 {
		if pos.board().checkers().any() {
			GameResult::Mated
		} else {
			GameResult::Drawn
		}
	} else if pos.is_drawn() {
		GameResult::Drawn
	} else {
		GameResult::NonTerminal
	}
}

fn init_opening(rng: &mut XorShiftRng) -> Position {
	'new_opening: loop {
		let mut pos = Position::new();
		for _ in 0..8 {
			let mut moves = MoveList::new();
			movegen::movegen(pos.board(), &mut moves);

			let idx = rng.random_range(0..moves.len());
			pos.make_move(moves[idx]);
			if game_result(&pos) != GameResult::NonTerminal {
				continue 'new_opening;
			}
		}
		return pos;
	}
}

fn run_game(search: &mut MCTS, rng: &mut XorShiftRng) {
	let mut limits = SearchLimits::new();
	limits.max_nodes = 5000;

	let mut pos = init_opening(rng);

	println!("opening position: {}", pos.board().to_fen());
	
	loop {
		let results = search.run(limits, false, &pos);
		println!("best move: {}", results.best_move);

		pos.make_move(results.best_move);
		let game_result = game_result(&pos);
		match game_result {
			GameResult::Drawn => {
				println!("Draw");
				break;
			}
			GameResult::Mated => {
				if pos.board().stm() == Color::White {
					println!("Black win");
				} else {
					println!("White win");
				}
				break;
			}
			GameResult::NonTerminal => {
				
			}
		}
	}
}
