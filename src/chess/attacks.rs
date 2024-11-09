use crate::types::{Bitboard, Color, Square};

const KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut result = [Bitboard::NONE; 64];
    let mut sq: usize = 0;
    while sq < 64 {
        let bb = Bitboard::from_square(Square::from_raw(sq as u8));
        result[sq] = result[sq].bit_or(bb.north_west().north());
        result[sq] = result[sq].bit_or(bb.north_west().west());
        result[sq] = result[sq].bit_or(bb.south_west().south());
        result[sq] = result[sq].bit_or(bb.south_west().west());
        result[sq] = result[sq].bit_or(bb.north_east().north());
        result[sq] = result[sq].bit_or(bb.north_east().east());
        result[sq] = result[sq].bit_or(bb.south_east().south());
        result[sq] = result[sq].bit_or(bb.south_east().east());
        sq += 1;
    }
    result
};

const KING_ATTACKS: [Bitboard; 64] = {
    let mut result = [Bitboard::NONE; 64];
    let mut sq: usize = 0;
    while sq < 64 {
        let bb = Bitboard::from_square(Square::from_raw(sq as u8));
        result[sq] = result[sq].bit_or(bb.north());
        result[sq] = result[sq].bit_or(bb.south());
        result[sq] = result[sq].bit_or(bb.east());
        result[sq] = result[sq].bit_or(bb.west());
        result[sq] = result[sq].bit_or(bb.north_east());
        result[sq] = result[sq].bit_or(bb.north_west());
        result[sq] = result[sq].bit_or(bb.south_east());
        result[sq] = result[sq].bit_or(bb.south_west());
        sq += 1;
    }
    result
};

enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

const RAYS: [[Bitboard; 64]; 8] = {
    let mut result: [[Bitboard; 64]; 8] = [[Bitboard::NONE; 64]; 8];
    let mut sq: usize = 0;
    while sq < 64 {
        let bb = Bitboard::from_square(Square::from_raw(sq as u8));
        let mut tmp = bb;
        while tmp.any() {
            tmp = tmp.north();
            result[Direction::North as usize][sq] =
                result[Direction::North as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.south();
            result[Direction::South as usize][sq] =
                result[Direction::South as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.east();
            result[Direction::East as usize][sq] = result[Direction::East as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.west();
            result[Direction::West as usize][sq] = result[Direction::West as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.north_east();
            result[Direction::NorthEast as usize][sq] =
                result[Direction::NorthEast as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.north_west();
            result[Direction::NorthWest as usize][sq] =
                result[Direction::NorthWest as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.south_east();
            result[Direction::SouthEast as usize][sq] =
                result[Direction::SouthEast as usize][sq].bit_or(tmp);
        }

        tmp = bb;
        while tmp.any() {
            tmp = tmp.south_west();
            result[Direction::SouthWest as usize][sq] =
                result[Direction::SouthWest as usize][sq].bit_or(tmp);
        }

        sq += 1;
    }

    result
};

const LINE_BETWEEN: [[Bitboard; 64]; 64] = {
    let mut result = [[Bitboard::NONE; 64]; 64];

    let mut sq1 = 0usize;
    while sq1 < 64 {
        let mut sq2 = 0usize;
        while sq2 < 64 {
            if RAYS[Direction::North as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::North as usize][sq1]
                    .bit_and(RAYS[Direction::South as usize][sq2]);
            }
            if RAYS[Direction::South as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::South as usize][sq1]
                    .bit_and(RAYS[Direction::North as usize][sq2]);
            }
            if RAYS[Direction::East as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::East as usize][sq1]
                    .bit_and(RAYS[Direction::West as usize][sq2]);
            }
            if RAYS[Direction::West as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::West as usize][sq1]
                    .bit_and(RAYS[Direction::East as usize][sq2]);
            }
            if RAYS[Direction::NorthEast as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::NorthEast as usize][sq1]
                    .bit_and(RAYS[Direction::SouthWest as usize][sq2]);
            }
            if RAYS[Direction::NorthWest as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::NorthWest as usize][sq1]
                    .bit_and(RAYS[Direction::SouthEast as usize][sq2]);
            }
            if RAYS[Direction::SouthEast as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::SouthEast as usize][sq1]
                    .bit_and(RAYS[Direction::NorthWest as usize][sq2]);
            }
            if RAYS[Direction::SouthWest as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::SouthWest as usize][sq1]
                    .bit_and(RAYS[Direction::NorthEast as usize][sq2]);
            }
            sq2 += 1;
        }
        sq1 += 1;
    }

    result
};

const LINE_THROUGH: [[Bitboard; 64]; 64] = {
    let mut result = [[Bitboard::NONE; 64]; 64];

    let mut sq1 = 0usize;
    while sq1 < 64 {
        let mut sq2 = 0usize;
        while sq2 < 64 {
            if RAYS[Direction::North as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::North as usize][sq1]
                    .bit_or(RAYS[Direction::South as usize][sq2]);
            }
            if RAYS[Direction::South as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::South as usize][sq1]
                    .bit_or(RAYS[Direction::North as usize][sq2]);
            }
            if RAYS[Direction::East as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] =
                    RAYS[Direction::East as usize][sq1].bit_or(RAYS[Direction::West as usize][sq2]);
            }
            if RAYS[Direction::West as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] =
                    RAYS[Direction::West as usize][sq1].bit_or(RAYS[Direction::East as usize][sq2]);
            }
            if RAYS[Direction::NorthEast as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::NorthEast as usize][sq1]
                    .bit_or(RAYS[Direction::SouthWest as usize][sq2]);
            }
            if RAYS[Direction::NorthWest as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::NorthWest as usize][sq1]
                    .bit_or(RAYS[Direction::SouthEast as usize][sq2]);
            }
            if RAYS[Direction::SouthEast as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::SouthEast as usize][sq1]
                    .bit_or(RAYS[Direction::NorthWest as usize][sq2]);
            }
            if RAYS[Direction::SouthWest as usize][sq1].has(Square::from_raw(sq2 as u8)) {
                result[sq1][sq2] = RAYS[Direction::SouthWest as usize][sq1]
                    .bit_or(RAYS[Direction::NorthEast as usize][sq2]);
            }
            sq2 += 1;
        }
        sq1 += 1;
    }

    result
};

fn ray_bb(sq: Square, dir: Direction) -> Bitboard {
    RAYS[dir as usize][sq.value() as usize]
}

pub fn line_between(sq1: Square, sq2: Square) -> Bitboard {
    LINE_BETWEEN[sq1.value() as usize][sq2.value() as usize]
}

pub fn line_through(sq1: Square, sq2: Square) -> Bitboard {
    LINE_THROUGH[sq1.value() as usize][sq2.value() as usize]
}

pub fn pawn_pushes_bb(c: Color, bb: Bitboard) -> Bitboard {
    if c == Color::White {
        bb.north()
    } else {
        bb.south()
    }
}

pub fn pawn_east_attacks_bb(c: Color, bb: Bitboard) -> Bitboard {
    if c == Color::White {
        bb.north_east()
    } else {
        bb.south_east()
    }
}

pub fn pawn_west_attacks_bb(c: Color, bb: Bitboard) -> Bitboard {
    if c == Color::White {
        bb.north_west()
    } else {
        bb.south_west()
    }
}

pub fn pawn_attacks_bb(c: Color, bb: Bitboard) -> Bitboard {
    pawn_west_attacks_bb(c, bb) | pawn_east_attacks_bb(c, bb)
}

pub fn knight_attacks(sq: Square) -> Bitboard {
    KNIGHT_ATTACKS[sq.value() as usize]
}

pub fn king_attacks(sq: Square) -> Bitboard {
    KING_ATTACKS[sq.value() as usize]
}

pub fn bishop_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::NONE;
    let ray = ray_bb(sq, Direction::NorthEast);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.lsb(), Direction::NorthEast);
    }

    let ray = ray_bb(sq, Direction::NorthWest);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.lsb(), Direction::NorthWest);
    }

    let ray = ray_bb(sq, Direction::SouthEast);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.msb(), Direction::SouthEast);
    }

    let ray = ray_bb(sq, Direction::SouthWest);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.msb(), Direction::SouthWest);
    }

    attacks
}

pub fn rook_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::NONE;
    let ray = ray_bb(sq, Direction::North);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.lsb(), Direction::North);
    }

    let ray = ray_bb(sq, Direction::South);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.msb(), Direction::South);
    }

    let ray = ray_bb(sq, Direction::East);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.lsb(), Direction::East);
    }

    let ray = ray_bb(sq, Direction::West);
    attacks |= ray;
    let blockers = ray & occ;
    if blockers.any() {
        attacks &= !ray_bb(blockers.msb(), Direction::West);
    }

    attacks
}

pub fn queen_attacks(sq: Square, occ: Bitboard) -> Bitboard {
    rook_attacks(sq, occ) | bishop_attacks(sq, occ)
}
