use std::{fs::File, io::Write, thread, time::Instant};

use rand::{seq::IndexedRandom, Rng};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use crate::{
    chess::{
        movegen::{self, MoveList},
        Move,
    },
    position::Position,
    search::{SearchLimits, MCTS},
    tree::{GameResult, Score},
    types::Color,
};

#[derive(Debug, Clone, Default)]
struct DataPoint {
    fen: String,
    visit_dist: Vec<(Move, f32)>,
    score: f32,
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
    wdl: WDL,
}

pub fn run_datagen(num_threads: i32, gen_value: bool) {
    println!(
        "Running {} datagen with {} threads",
        if gen_value { "value" } else { "policy" },
        num_threads
    );
    let mut handles = Vec::new();
    for i in 0..num_threads {
        handles.push(thread::spawn(move || {
            datagen_thread(i, gen_value);
        }));
    }
    for handle in handles {
        let _ = handle.join();
    }
}

pub fn datagen_thread(thread_id: i32, gen_value: bool) {
    let mut search = MCTS::new();
    let seed = rand::rng().next_u64();
    println!("Thread {} RNG seed: {}", thread_id, seed);

    let filename = if gen_value {
        format!("datagen{}.value.txt", thread_id)
    } else {
        format!("datagen{}.policy.txt", thread_id)
    };
    let mut data_file = File::create(filename).expect("Unable to create data file");

    let mut rng = XorShiftRng::seed_from_u64(seed);
    let mut games = 0;
    let mut positions = 0;
    let mut total_positions = 0;
    let mut start_time = Instant::now();
    loop {
        let game = run_game(&mut search, &mut rng);
        let (num_positions, data) = if gen_value {
            serialize_value(&game, &mut rng)
        } else {
            serialize_policy(&game)
        };
        data_file
            .write_all(data.as_bytes())
            .expect("Unable to write data");

        games += 1;
        positions += num_positions;
        total_positions += num_positions;
        if games % 32 == 0 {
            println!(
                "{} datagen: Thread {} wrote {} total games and {} total positions. {} positions in last 32 games in {} seconds",
                if gen_value { "value" } else { "policy" },
                thread_id,
                games,
                total_positions,
                positions,
                start_time.elapsed().as_secs_f32()
            );
            start_time = Instant::now();
            positions = 0;
        }
    }
}

fn serialize_value(game: &Game, rng: &mut XorShiftRng) -> (i32, String) {
    let mut value = String::new();
    let mut num_positions = 0;
    let selected: Vec<_> = game.points.choose_multiple(rng, 10).cloned().collect();
    for pt in &selected {
        value += format!("{} | {} | {}\n", pt.fen, pt.score, game.wdl.as_f32()).as_str();

        num_positions += 1;
    }
    (num_positions, value)
}

fn serialize_policy(game: &Game) -> (i32, String) {
    let mut policy = String::new();
    let mut num_positions = 0;
    for pt in &game.points {
        policy += pt.fen.as_str();
        for (_mv, frac) in &pt.visit_dist {
            policy += format!(" | {}", frac).as_str();
        }
        policy += "\n";

        num_positions += 1;
    }
    (num_positions, policy)
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
    } else if pos.is_drawn(0) {
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

    let mut game = Game::default();

    loop {
        let results = search.run(limits, false, &pos);
        let mut datapt_score = match results.score {
            Score::Win(_) => 1.0,
            Score::Draw => 0.5,
            Score::Loss(_) => 0.0,
            Score::Normal(wdl) => wdl,
        };
        if pos.board().stm() == Color::Black {
            datapt_score = 1.0 - datapt_score;
        }

        game.points.push(DataPoint {
            fen: pos.board().to_fen(),
            visit_dist: results.visit_dist,
            score: datapt_score,
        });

        pos.make_move(results.best_move);
        let game_result = game_result(&pos);
        match game_result {
            GameResult::Drawn => {
                game.wdl = WDL::Draw;
                break;
            }
            GameResult::Mated => {
                if pos.board().stm() == Color::White {
                    game.wdl = WDL::BlackWin;
                } else {
                    game.wdl = WDL::WhiteWin;
                }
                break;
            }
            GameResult::NonTerminal => {}
        }
    }
    game
}
