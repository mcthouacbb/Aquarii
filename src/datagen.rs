use std::{
    fs::{self, File},
    io::{self, BufReader, ErrorKind, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Instant,
};

use rand::{
    seq::{IndexedRandom, SliceRandom},
    Rng,
};
use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;
use viriformat::{
    chess::{
        board::{Board as VfBoard, GameOutcome},
        chessmove::{Move as VfMove, MoveFlags as VfMoveFlags},
        piece::PieceType as VfPieceType,
        types::Square as VfSquare,
    },
    dataformat::{Game as VfGame, WDL as VfWDL},
};

use crate::{
    chess::{
        movegen::{self, MoveList},
        Board, Move, MoveKind,
    },
    position::Position,
    score::{sigmoid, sigmoid_inv, GameResult, Score},
    search::{SearchLimits, MCTS},
    types::{Color, PieceType, Square},
};

#[derive(Debug, Clone)]
struct DataPoint {
    fen: String,
    visit_dist: Vec<(Move, f32)>,
    best_move: Move,
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

const NUM_THREADS: i32 = 8;

pub fn run_datagen() {
    let mut handles = Vec::new();
    let stop = Arc::new(AtomicBool::new(false));
    for i in 0..NUM_THREADS {
        let stop_ref = stop.clone();
        handles.push(thread::spawn(move || {
            datagen_thread(i, &stop_ref);
        }));
    }

    let stop_ref = stop.clone();
    ctrlc::set_handler(move || stop_ref.store(true, Ordering::SeqCst))
        .expect("Error setting Ctrl+C handler");

    for handle in handles {
        let _ = handle.join();
    }

    let mut combined_value_file = File::create_new("datagen.value.bin").unwrap();
    for i in 0..NUM_THREADS {
        let mut thread_value_file = File::open(format!("datagen{}.value.bin", i)).unwrap();
        io::copy(&mut thread_value_file, &mut combined_value_file).unwrap();
    }

    println!("Finished combining value files into datagen.value.bin");

    for i in 0..NUM_THREADS {
        fs::remove_file(format!("datagen{}.value.bin", i)).unwrap();
    }

    let mut combined_policy_file = File::create_new("datagen.policy.txt").unwrap();
    for i in 0..NUM_THREADS {
        let mut thread_policy_file = File::open(format!("datagen{}.policy.txt", i)).unwrap();
        io::copy(&mut thread_policy_file, &mut combined_policy_file).unwrap();
    }

    println!("Finished combining policy files into datagen.policy.txt");

    for i in 0..NUM_THREADS {
        fs::remove_file(format!("datagen{}.policy.txt", i)).unwrap();
    }
}

pub fn extract_fens(in_filename: &str, out_filename: &str) {
    let value_file = match File::open(in_filename) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening file '{}': {}", in_filename, err);
            return;
        }
    };

    let mut reader = BufReader::new(value_file);

    let mut games = Vec::new();

    loop {
        match VfGame::deserialise_from(&mut reader, Vec::new()) {
            Ok(game) => {
                games.push(game);
            }
            Err(err) => {
                if err.kind() == ErrorKind::UnexpectedEof {
                    break;
                } else {
                    println!("Encountered viriformat error: {}", err);
                }
            }
        }
    }

    const PPG: usize = 10;

    println!(
        "Finished loading {} games from file '{}'",
        games.len(),
        in_filename
    );
    println!("Extracting {} positions per game", PPG);

    let mut positions = Vec::new();
    let mut rng = XorShiftRng::seed_from_u64(rand::rng().next_u64());

    for game in games {
        let mut game_positions = Vec::new();

        let mut position = Board::from_fen(game.initial_position().to_string().as_str()).unwrap();
        for &(mv, score) in &game.moves {
            game_positions.push((position.to_fen(), score, wdl_from_vf(game.outcome())));
            position.make_move(move_from_vf(mv));
        }

        positions.extend(
            game_positions
                .choose_multiple(&mut rng, PPG)
                .map(|pos| pos.clone()),
        );
    }

    println!("Finished extracing {} positions", positions.len());
    println!("Shuffling positions");

    positions.shuffle(&mut rng);

    let mut fens_file = match File::create_new(out_filename) {
        Ok(file) => file,
        Err(err) => {
            println!("Error opening output file '{}': {}", out_filename, err);
            return;
        }
    };

    for (fen, score, wdl) in &positions {
        let result = write!(
            fens_file,
            "{} | {} | {}\n",
            fen,
            sigmoid(score.get() as f32, 400.0),
            wdl.as_f32()
        );
        if let Err(err) = result {
            println!("Error writing to output file '{}': {}", out_filename, err);
            return;
        }
    }

    println!(
        "Finished writing {} positions to file '{}'",
        positions.len(),
        out_filename
    );
}

fn datagen_thread(thread_id: i32, stop: &Arc<AtomicBool>) {
    let mut search = MCTS::new();
    let seed = rand::rng().next_u64();
    println!("Thread {} RNG seed: {}", thread_id, seed);

    let value_filename = format!("datagen{}.value.bin", thread_id);
    let mut value_file = File::create(value_filename).expect("Unable to create value data file");

    let policy_filename = format!("datagen{}.policy.txt", thread_id);
    let mut policy_file = File::create(policy_filename).expect("Unable to create policy data file");

    let mut rng = XorShiftRng::seed_from_u64(seed);
    let mut games = 0;
    let mut positions = 0;
    let mut total_positions = 0;
    let mut start_time = Instant::now();
    while !stop.load(Ordering::SeqCst) {
        let game = run_game(&mut search, &mut rng);
        let (num_positions, value_data, policy_data) = serialize(&game);
        value_data.serialise_into(&mut value_file).unwrap();

        policy_file
            .write_all(policy_data.as_bytes())
            .expect("Unable to write policy data");

        games += 1;
        positions += num_positions;
        total_positions += num_positions;
        if games % 32 == 0 {
            println!(
                "Thread {} wrote {} total games and {} total positions. {} positions in last 32 games in {} seconds",
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

    println!(
        "Thread {} finished writing {} total games and {} total positions",
        thread_id, games, total_positions
    );
}

fn move_to_vf(mv: Move) -> VfMove {
    let from_sq = VfSquare::new(mv.from_sq().value()).unwrap();
    let to_sq = VfSquare::new(mv.to_sq().value()).unwrap();
    match mv.kind() {
        MoveKind::None => VfMove::new(from_sq, to_sq),
        MoveKind::Castle => VfMove::new_with_flags(from_sq, to_sq, VfMoveFlags::Castle),
        MoveKind::Enpassant => VfMove::new_with_flags(from_sq, to_sq, VfMoveFlags::EnPassant),
        MoveKind::Promotion => VfMove::new_with_promo(
            from_sq,
            to_sq,
            VfPieceType::new(mv.promo_piece() as u8).unwrap(),
        ),
    }
}

fn wdl_to_vf(wdl: WDL) -> GameOutcome {
    match wdl {
        WDL::WhiteWin => GameOutcome::WhiteWin(viriformat::chess::board::WinType::Mate),
        // DrawType might be wrong but this is not even used anyways so it doesn't matter
        WDL::Draw => GameOutcome::Draw(viriformat::chess::board::DrawType::Repetition),
        WDL::BlackWin => GameOutcome::BlackWin(viriformat::chess::board::WinType::Mate),
    }
}

fn move_from_vf(mv: VfMove) -> Move {
    let from_sq = Square::from_raw(mv.from().index() as u8);
    let to_sq = Square::from_raw(mv.to().index() as u8);

    if mv.is_castle() {
        Move::castle(from_sq, to_sq)
    } else if mv.is_ep() {
        Move::enpassant(from_sq, to_sq)
    } else if mv.is_promo() {
        Move::promo(
            from_sq,
            to_sq,
            PieceType::from_raw(mv.promotion_type().unwrap().index() as u8),
        )
    } else {
        Move::normal(from_sq, to_sq)
    }
}

fn wdl_from_vf(wdl: VfWDL) -> WDL {
    match wdl {
        VfWDL::Win => WDL::WhiteWin,
        VfWDL::Draw => WDL::Draw,
        VfWDL::Loss => WDL::BlackWin,
    }
}

fn serialize(game: &Game) -> (i32, viriformat::dataformat::Game, String) {
    let mut initial_pos = VfBoard::new();
    initial_pos
        .set_from_fen(&game.points[0].fen, false)
        .unwrap();

    let mut value = VfGame::new(&initial_pos);
    value.set_outcome(wdl_to_vf(game.wdl));

    let mut policy = String::new();
    let mut num_positions = 0;
    for pt in &game.points {
        value.add_move(
            move_to_vf(pt.best_move),
            sigmoid_inv(pt.score, 400.0) as i16,
        );

        policy += pt.fen.as_str();
        for (_mv, frac) in &pt.visit_dist {
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
            best_move: results.best_move,
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
