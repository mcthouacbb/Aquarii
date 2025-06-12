use rand::Rng;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use crate::{chess::{movegen::{self, MoveList}, Move}, position::Position, search::{GameResult, MateScore, Score, SearchLimits, MCTS}, types::Color};

#[derive(Debug, Clone, Default)]
struct DataPoint {
	fen: String,
	visit_dist: Vec<(Move, f32)>,
	score: f32
}

#[derive(Debug, Clone, Copy, Default)]
enum WDL {
	WhiteWin,
	#[default]
	Draw,
	BlackWin,
}

impl WDL {
	fn as_f32(self) -> f32 {
		match self {
			Self::WhiteWin => 1.0,
			Self::Draw => 0.5,
			Self::BlackWin => 0.0,
		}
	}
}

#[derive(Debug, Clone, Default)]
struct Game {
	points: Vec<DataPoint>,
	wdl: WDL
}

pub fn run_datagen() {
	let mut search = MCTS::new(10000);
	// temporary
	let seed = 9792801834308900943;//rand::rng().next_u64();
	println!("RNG seed: {}", seed);

	let mut rng = XorShiftRng::seed_from_u64(seed);
	loop {
		let game = run_game(&mut search, &mut rng);
		println!("{}", serialize_value(&game));

		// temporary
		break;
	}
}

fn serialize_value(game: &Game) -> String {
	let mut result = String::new();
	for pt in &game.points {
		result += format!("{} | {} | {}\n", pt.fen, pt.score, game.wdl.as_f32()).as_str();
	}
	result
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

fn run_game(search: &mut MCTS, rng: &mut XorShiftRng) -> Game {
	let mut limits = SearchLimits::new();
	limits.max_nodes = 5000;

	let mut pos = init_opening(rng);

	println!("opening position: {}", pos.board().to_fen());

	let mut game = Game::default();
	
	loop {
		let results = search.run(limits, false, &pos);
		let score_str = match results.score {
			Score::Mate(mate_score) => match mate_score {
				MateScore::Loss(ply) => format!("mate -{}", ply),
				MateScore::Win(ply) => format!("mate {}", ply),
			},
			Score::Normal(wdl) => format!("wdl {}", wdl),
		};
		let mut datapt_score = match results.score {
			Score::Mate(mate_score) => match mate_score {
				MateScore::Loss(_) => 0.0,
				MateScore::Win(_) => 1.0,
			},
			Score::Normal(wdl) => wdl,
		};
		if pos.board().stm() == Color::Black {
			datapt_score = 1.0 - datapt_score;
		}
		println!("best move: {}, score: {}", results.best_move, score_str);
		// println!("visit dist: {:?}", results.visit_dist);

		game.points.push(DataPoint { fen: pos.board().to_fen(), visit_dist: results.visit_dist, score: datapt_score});

		pos.make_move(results.best_move);
		let game_result = game_result(&pos);
		match game_result {
			GameResult::Drawn => {
				println!("Draw");
				game.wdl = WDL::Draw;
				break;
			}
			GameResult::Mated => {
				if pos.board().stm() == Color::White {
					println!("Black win");
					game.wdl = WDL::BlackWin;
				} else {
					println!("White win");
					game.wdl = WDL::WhiteWin;
				}
				break;
			}
			GameResult::NonTerminal => {
				
			}
		}
	}
	game
}
