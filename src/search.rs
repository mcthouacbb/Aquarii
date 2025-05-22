use std::{mem::swap, ops::Range, time::Instant};

use arrayvec::ArrayVec;

use crate::{
    chess::{
        attacks,
        movegen::{movegen, MoveList},
        Move, MoveKind,
    },
    eval,
    position::Position,
    types::{Piece, PieceType},
};

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum GameResult {
    NonTerminal,
    Mated,
    Drawn,
}

#[derive(Clone)]
struct Node {
    first_child_idx: u32,
    child_count: u8,
    parent_move: Move,
    result: GameResult,
    policy: f32,
    wins: f32,
    visits: u32,
}

impl Node {
    fn new(mv: Move, policy: f32) -> Self {
        Node {
            first_child_idx: 0,
            child_count: 0,
            parent_move: mv,
            result: GameResult::NonTerminal,
            policy: policy,
            wins: 0.0,
            visits: 0,
        }
    }

    fn q(&self) -> f32 {
        self.wins / self.visits as f32
    }

    fn is_terminal(&self) -> bool {
        self.result != GameResult::NonTerminal
    }

    fn child_indices(&self) -> Range<u32> {
        self.first_child_idx..(self.first_child_idx + self.child_count as u32)
    }
}

fn sigmoid(x: f32, scale: f32) -> f32 {
    1.0 / (1.0 + (-x / scale).exp())
}

fn sigmoid_inv(x: f32, scale: f32) -> f32 {
    scale * (x / (1.0 - x)).ln()
}

fn softmax(vals: &mut ArrayVec<f32, 256>, max_val: f32) {
    let mut exp_sum = 0.0;
    for v in vals.iter_mut() {
        *v = (*v - max_val).exp();
        exp_sum += *v;
    }
    for v in vals.iter_mut() {
        *v /= exp_sum;
    }
}

#[derive(Copy, Clone)]
pub struct SearchLimits {
    pub use_clock: bool,
    pub time: i32,
    pub inc: i32,
    pub max_depth: i32,
    pub max_time: i32,
    pub max_nodes: i32,
}

#[derive(Copy, Clone)]
pub struct SearchResults {
    pub best_move: Move,
    pub nodes: u64,
}

impl SearchLimits {
    pub fn new() -> Self {
        Self {
            use_clock: false,
            time: -1,
            inc: -1,
            max_depth: -1,
            max_time: -1,
            max_nodes: -1,
        }
    }
}

pub struct MCTS {
    nodes: Vec<Node>,
    iters: u32,
    root_position: Position,
    position: Position,
    selection: Vec<u32>,
}

impl MCTS {
    const CUCT: f32 = std::f32::consts::SQRT_2;
    const EVAL_SCALE: f32 = 200.0;

    pub fn new(max_nodes: u32) -> Self {
        Self {
            nodes: Vec::with_capacity(max_nodes as usize),
            iters: 0,
            root_position: Position::new(),
            position: Position::new(),
            selection: Vec::new(),
        }
    }

    fn select_leaf(&mut self) {
        self.selection.clear();
        let mut node_idx = 0u32;
        self.selection.push(node_idx);

        loop {
            if self.nodes[node_idx as usize].child_count == 0
                && self.nodes[node_idx as usize].visits == 1
            {
                self.expand_node(node_idx);
            }
            let node = &self.nodes[node_idx as usize];
            if node.is_terminal() || node.child_count == 0 {
                break;
            } else {
                let mut best_uct = -1f32;
                let mut best_child_idx = 0u32;
                for child_idx in node.child_indices() {
                    let child = &self.nodes[child_idx as usize];
                    let q = if child.visits == 0 {
                        // TODO: inf root fpu
                        0.5
                    } else {
                        // 1 - child q because child q is from opposite perspective of current node
                        1.0 - child.q()
                    };
                    let policy = child.policy;
                    let expl = (node.visits as f32).sqrt() / (1 + child.visits) as f32;
                    let uct = q + Self::CUCT * policy * expl;

                    if uct > best_uct {
                        best_child_idx = child_idx;
                        best_uct = uct;
                    }
                }

                node_idx = best_child_idx;
                let child = &self.nodes[best_child_idx as usize];
                // println!("{}, {}", self.position.board().to_fen(), child.parent_move);
                self.position.make_move(child.parent_move);
                self.selection.push(node_idx);
            }
        }
    }

    fn get_policy(&self, mv: Move) -> f32 {
        let board = self.position.board();
        let opp_pawns = board.colored_pieces(Piece::new(!board.stm(), PieceType::Pawn));
        let pawn_protected = attacks::pawn_attacks_bb(!board.stm(), opp_pawns);
        let moving_piece = board.piece_at(mv.from_sq()).unwrap();
        let captured_piece = board.piece_at(mv.to_sq());
        let cap_bonus = if let Some(captured) = captured_piece {
            match captured.piece_type() {
                PieceType::Pawn => 0.7,
                PieceType::Knight => 2.0,
                PieceType::Bishop => 2.0,
                PieceType::Rook => 3.0,
                PieceType::Queen => 4.5,
                _ => 0.0,
            }
        } else {
            0.0
        };
        let pawn_protected_penalty = if pawn_protected.has(mv.to_sq()) {
            match moving_piece.piece_type() {
                PieceType::Pawn => 0.6,
                PieceType::Knight => 1.9,
                PieceType::Bishop => 1.9,
                PieceType::Rook => 2.8,
                PieceType::Queen => 4.2,
                _ => 0.0,
            }
        } else {
            0.0
        };

        let promo_bonus = if mv.kind() == MoveKind::Promotion {
            match mv.promo_piece() {
                PieceType::Knight => 0.2,
                PieceType::Queen => 2.0,
                _ => 0.0,
            }
        } else {
            0.0
        };

        cap_bonus + promo_bonus - pawn_protected_penalty
    }

    fn expand_node(&mut self, node_idx: u32) {
        let mut moves = MoveList::new();
        movegen(self.position.board(), &mut moves);

        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;
        for mv in moves.iter() {
            let policy = self.get_policy(*mv);
            max_policy = max_policy.max(policy);
            policies.push(policy);
        }

        softmax(&mut policies, max_policy);

        let first_child_idx = self.nodes.len() as u32;
        let node = &mut self.nodes[node_idx as usize];
        node.first_child_idx = first_child_idx;
        node.child_count = moves.len() as u8;

        for (i, mv) in moves.iter().enumerate() {
            self.nodes.push(Node::new(*mv, policies[i]));
        }
    }

    fn eval_wdl(&self) -> f32 {
        let board = self.position.board();
        let eval = eval::eval(board);

        sigmoid(eval as f32, Self::EVAL_SCALE)
    }

    fn simulate(&self) -> (f32, GameResult) {
        let mut moves = MoveList::new();
        movegen(self.position.board(), &mut moves);

        let result = if moves.len() == 0 {
            if self.position.board().checkers().any() {
                GameResult::Mated
            } else {
                GameResult::Drawn
            }
        } else if self.position.is_drawn() {
            GameResult::Drawn
        } else {
            GameResult::NonTerminal
        };

        match result {
            GameResult::Drawn => (0.5, result),
            GameResult::Mated => (0.0, result),
            GameResult::NonTerminal => (self.eval_wdl(), result),
        }
    }

    fn backprop(&mut self, mut result: f32) {
        for node_idx in self.selection.iter().rev() {
            let node = &mut self.nodes[*node_idx as usize];

            node.visits += 1;
            node.wins += result;

            result = 1.0 - result;
        }
    }

    fn perform_one_iter(&mut self) {
        self.position = self.root_position.clone();
        self.select_leaf();

        let (score, game_result) = self.simulate();

        let leaf_idx = *self.selection.last().unwrap();
        let leaf = &mut self.nodes[leaf_idx as usize];
        leaf.result = game_result;

        self.backprop(score);
    }

    fn get_best_move(&self) -> Move {
        let root_node = &self.nodes[0];
        let mut best_q = -1.0;
        let mut best_move = Move::NULL;
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            if child_node.visits == 0 {
                continue;
            }
            if 1.0 - child_node.q() > best_q {
                best_q = 1.0 - child_node.q();
                best_move = child_node.parent_move;
            }
        }
        best_move
    }

    pub fn display_tree(&self) {
        let root_node = &self.nodes[0];
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            println!(
                "{} => {} visits {} cp",
                child_node.parent_move,
                child_node.visits,
                sigmoid_inv(1.0 - child_node.q(), Self::EVAL_SCALE)
            );
        }
    }

    // depth 2 perft to find the node
    fn find_node(&self, position: &Position) -> Option<u32> {
        if self.nodes.len() == 0 {
            return None;
        }
        let root_node = &self.nodes[0];
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            let mut new_pos = self.root_position.clone();
            new_pos.make_move(child_node.parent_move);
            for child2_idx in child_node.child_indices() {
                let child2_node = &self.nodes[child2_idx as usize];
                let mut new_pos2 = new_pos.clone();
                new_pos2.make_move(child2_node.parent_move);
                if new_pos2 == *position {
                    println!("{} {}", child_node.parent_move, child2_node.parent_move);
                    return Some(child2_idx);
                }
            }
        }
        None
    }

    fn build_tree(&mut self, old_nodes: &Vec<Node>, node_idx: u32) {
        let old_node = &old_nodes[node_idx as usize];
        self.nodes.push(old_node.clone());

        self.build_tree_impl(old_nodes, node_idx, 0);
    }

    fn build_tree_impl(&mut self, old_nodes: &Vec<Node>, old_node_idx: u32, new_node_idx: u32) {
        let old_node = &old_nodes[old_node_idx as usize];
        let first_child_idx = self.nodes.len() as u32;
        if old_node.child_count == 0 {
            return;
        }

        {
            let new_node: &mut Node = &mut self.nodes[new_node_idx as usize];
            new_node.child_count = old_node.child_count;
            new_node.first_child_idx = first_child_idx as u32;
        }

        for old_child_idx in old_node.child_indices() {
            let old_child = &old_nodes[old_child_idx as usize];
            self.nodes.push(old_child.clone());
        }

        for (iter, old_child_idx) in old_node.child_indices().enumerate() {
            let new_node = &self.nodes[new_node_idx as usize];
            self.build_tree_impl(old_nodes, old_child_idx, new_node.first_child_idx + iter as u32);
        }
    }

    pub fn run(
        &mut self,
        limits: SearchLimits,
        report: bool,
        position: &Position,
    ) -> SearchResults {
        let node_idx = self.find_node(position);

        self.root_position = position.clone();
        self.position = self.root_position.clone();
        self.iters = 0;
        
        if let Some(old_node_idx) = node_idx {
            let mut old_nodes = Vec::with_capacity(self.nodes.capacity());
            swap(&mut old_nodes, &mut self.nodes);
            self.build_tree(&old_nodes, old_node_idx);
        } else {
            self.nodes.clear();
            self.nodes.push(Node::new(Move::NULL, 0f32));
            self.expand_node(0);
            self.nodes[0].visits += 1;
            self.nodes[0].wins += self.eval_wdl();
        }

        let mut total_depth = 0;
        let mut prev_depth = 0;

        let mut nodes = 0u64;

        let start_time = Instant::now();

        while limits.max_nodes < 0 || self.iters <= limits.max_nodes as u32 {
            self.perform_one_iter();

            total_depth += (self.selection.len() - 1) as u32;
            self.iters += 1;

            nodes += self.selection.len() as u64;

            let curr_depth = total_depth / self.iters;
            if curr_depth > prev_depth {
                if limits.max_depth > 0 && curr_depth >= limits.max_depth as u32 {
                    break;
                }
                prev_depth = curr_depth;
                if report {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    println!(
                        "info depth {} nodes {} time {} nps {} score cp {} pv {}",
                        curr_depth,
                        nodes,
                        (elapsed * 1000.0) as u64,
                        (nodes as f64 / elapsed as f64) as u64,
                        sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE).round(),
                        self.get_best_move()
                    );
                }
            }

            // don't check every iter
            if self.iters % 512 == 0 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let elapsed_ms = (elapsed * 1000.0) as i32;
                if limits.max_time >= 0 && elapsed_ms >= limits.max_time {
                    break;
                }

                if limits.use_clock && elapsed_ms >= limits.time / 20 + limits.inc / 2 {
                    break;
                }
            }
        }

        if report {
            let curr_depth = total_depth / self.iters;
            let elapsed = start_time.elapsed().as_secs_f64();
            println!(
                "info depth {} nodes {} time {} nps {} score cp {} pv {}",
                curr_depth,
                nodes,
                (elapsed * 1000.0) as u64,
                (nodes as f64 / elapsed as f64) as u64,
                sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE).round(),
                self.get_best_move()
            );
        }

        SearchResults {
            best_move: self.get_best_move(),
            nodes: nodes,
        }
    }

    pub fn new_game(&mut self) {
        self.nodes.clear();
    }
}
