use std::{fs::File, io::Write, thread, time::Instant};

use rand::Rng;
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

use crate::{
    chess::{
        movegen::{self, MoveList},
        Move,
    },
    position::Position,
    search::{GameResult, MateScore, Score, SearchLimits, MCTS},
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

const NUM_THREADS: i32 = 2;

pub fn run_datagen() {
    let mut handles = Vec::new();
    for i in 0..NUM_THREADS {
        handles.push(thread::spawn(move || {
            datagen_thread(i);
        }));
    }
    for handle in handles {
        handle.join();
    }
}

pub fn datagen_thread(thread_id: i32) {
    let mut search = MCTS::new(10000);
    let seed = rand::rng().next_u64();
    println!("Thread {} RNG seed: {}", thread_id, seed);

    let value_filename = format!("datagen{}.value.txt", thread_id);
    let mut value_file = File::create(value_filename).expect("Unable to create value data file");

    let policy_filename = format!("datagen{}.policy.txt", thread_id);
    let mut policy_file = File::create(policy_filename).expect("Unable to create policy data file");

    let mut rng = XorShiftRng::seed_from_u64(seed);
    let mut games = 0;
    let mut positions = 0;
    let mut start_time = Instant::now();
    loop {
        let game = run_game(thread_id, &mut search, &mut rng);
        let (num_positions, value_data, policy_data) = serialize(&game);
        value_file
            .write_all(value_data.as_bytes())
            .expect("Unable to write value data");

        policy_file
            .write_all(policy_data.as_bytes())
            .expect("Unable to write policy data");

        games += 1;
        positions += num_positions;
        if games % 32 == 0 {
            println!(
                "Thread {} wrote {} total games {} positions in last 32 games in {} seconds",
                thread_id,
                games,
                positions,
                start_time.elapsed().as_secs_f32()
            );
            start_time = Instant::now();
            positions = 0;
        }
    }
}

fn serialize(game: &Game) -> (i32, String, String) {
    let mut value = String::new();
    let mut policy = String::new();
    let mut num_positions = 0;
    for pt in &game.points {
        value += format!("{} | {} | {}\n", pt.fen, pt.score, game.wdl.as_f32()).as_str();

        policy += pt.fen.as_str();
        for (mv, frac) in &pt.visit_dist {
            policy += format!(" | {}", frac).as_str();
        }
        policy += "\n";

        num_positions += 1;
    }
    (num_positions, value, policy)
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

fn run_game(thread_id: i32, search: &mut MCTS, rng: &mut XorShiftRng) -> Game {
    let mut limits = SearchLimits::new();
    limits.max_nodes = 5000;

    let mut pos = init_opening(rng);

    // println!("Thread {} opening position: {}", thread_id, pos.board().to_fen());

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
        // println!("best move: {}, score: {}", results.best_move, score_str);
        // println!("visit dist: {:?}", results.visit_dist);

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
