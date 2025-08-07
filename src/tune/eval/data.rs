use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use rand::seq::SliceRandom;

use crate::{chess::Board, tune::eval::trace, types::Color};

pub struct Coefficient {
    pub index: u16,
    pub value: f32,
}

pub struct Position {
    pub coeffs: Vec<Coefficient>,
    pub score: f32,
    pub wdl: f32,
}

pub struct Dataset {
    pub positions: Vec<Position>,
}

pub fn load_dataset(files: &[File]) -> Dataset {
    let mut positions = Vec::new();
    for file in files {
        load_data_file(&file, &mut positions);
    }
    positions.shuffle(&mut rand::rng());
    println!("Finished shuffling positions");
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
        if board.checkers().any() {
            continue;
        }
        let mut pos = Position {
            coeffs: Vec::new(),
            score: 0.0,
            wdl: 0.0,
        };

        pos.score = parts[1].parse::<f32>().expect("Could not parse score");
        pos.wdl = parts[2].parse::<f32>().expect("Could not parse score");

        // make stm relative
        if board.stm() == Color::Black {
            pos.score = 1.0 - pos.score;
            pos.wdl = 1.0 - pos.wdl;
        }

        let coeffs = trace::compute_coeffs(&board);

        for c in coeffs {
            pos.coeffs.push(Coefficient {
                index: c.0 as u16,
                value: c.1,
            });
        }

        positions.push(pos);

        if positions.len() % 65536 == 0 {
            println!("Loaded {} positions", positions.len());
        }
    }
    println!("Finished loading {} positions", positions.len());
}
