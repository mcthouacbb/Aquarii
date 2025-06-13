use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{
    chess::{
        movegen::{self, MoveList},
        Board,
    },
    policy,
    tune::policy::trace,
};

pub struct Coefficient {
    mv_idx: u16,
    index: u16,
    value: f32,
}

pub struct Position {
    coeffs: Vec<Coefficient>,
    visit_dist: Vec<f32>,
}

pub struct Dataset {
    positions: Vec<Position>,
}

pub fn load_dataset(files: &[File]) -> Dataset {
    let mut positions = Vec::new();
    for file in files {
        load_data_file(&file, &mut positions);
    }
    Dataset {
        positions: positions,
    }
}

fn load_data_file(file: &File, positions: &mut Vec<Position>) {
    let reader = BufReader::new(file);
    let lines = reader
        .lines()
        .collect::<Result<Vec<String>, _>>()
        .expect("Cannot read file");

    for line in lines {
        let parts: Vec<&str> = line.split(" | ").collect();
        let fen = parts[0];
        let board = Board::from_fen(fen).expect("Invalid fen string in policy data");
        let mut pos = Position {
            coeffs: Vec::new(),
            visit_dist: Vec::with_capacity(parts.len() - 1),
        };

        for str in parts.iter().skip(1) {
            let frac = str
                .parse::<f32>()
                .expect("Could not parse visit distribution");
            pos.visit_dist.push(frac);
        }

        // somehow get coeffs out of policy
        let mut moves = MoveList::new();
        movegen::movegen(&board, &mut moves);

        for (mv_idx, mv) in moves.iter().enumerate() {
            let coeffs = trace::compute_coeffs(&board, *mv);
            for c in coeffs {
                pos.coeffs.push(Coefficient {
                    mv_idx: mv_idx as u16,
                    index: c.0 as u16,
                    value: c.1,
                });
            }
        }

        positions.push(pos);

        if positions.len() % 65536 == 0 {
            println!("Loaded {} positions", positions.len());
        }
    }
    println!("Finished loading {} positions", positions.len());
}
