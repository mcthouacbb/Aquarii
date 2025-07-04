use std::{mem::swap, num::NonZeroI16, ops::Range, time::Instant};

use arrayvec::ArrayVec;

use crate::{
    chess::{
        movegen::{movegen, MoveList},
        Move,
    },
    eval,
    policy::get_policy,
    position::Position,
};

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum GameResult {
    NonTerminal,
    Mated,
    Drawn,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MateScore {
    Loss(u16),
    Win(u16),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Score {
    Mate(MateScore),
    Normal(f32),
}

#[derive(Clone)]
struct Node {
    first_child_idx: u32,
    child_count: u8,
    parent_move: Move,
    result: GameResult,
    mate_dist: Option<NonZeroI16>,
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
            mate_dist: None,
            policy: policy,
            wins: 0.0,
            visits: 0,
        }
    }

    fn q(&self) -> f32 {
        self.wins / self.visits as f32
    }

    fn mate_score(&self) -> Option<MateScore> {
        if self.result == GameResult::Mated {
            Some(MateScore::Loss(0))
        } else if let Some(mate_dist) = self.mate_dist {
            let mate_dist = mate_dist.get() as i32;
            if mate_dist > 0 {
                Some(MateScore::Win(mate_dist as u16))
            } else {
                Some(MateScore::Loss(-mate_dist as u16))
            }
        } else {
            None
        }
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

#[derive(Clone)]
pub struct SearchResults {
    pub best_move: Move,
    pub nodes: u64,
    pub score: Score,
    pub visit_dist: Vec<(Move, f32)>,
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
    const ROOT_CPUCT: f32 = 1.10929019;
    const CPUCT: f32 = 0.70710678;
    const EVAL_SCALE: f32 = 400.0;

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
            let root = node_idx == 0;
            if node.is_terminal() || node.child_count == 0 {
                break;
            } else {
                let mut best_uct = -1f32;
                let mut best_child_idx = 0u32;
                for child_idx in node.child_indices() {
                    let child = &self.nodes[child_idx as usize];
                    let q = if child.visits == 0 {
                        if root {
                            1000.0
                        } else {
                            node.q()
                        }
                    } else {
                        // 1 - child q because child q is from opposite perspective of current node
                        1.0 - child.q()
                    };
                    let policy = child.policy;
                    let expl = (node.visits as f32).sqrt() / (1 + child.visits) as f32;
                    let cpuct = if root { Self::ROOT_CPUCT } else { Self::CPUCT };
                    let uct = q + cpuct * policy * expl;

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

    fn expand_node(&mut self, node_idx: u32) {
        let mut moves = MoveList::new();
        movegen(self.position.board(), &mut moves);

        let tmp = if node_idx == 0 { 3.0 } else { 1.0 };

        let mut policies = ArrayVec::<f32, 256>::new();
        let mut max_policy = 0f32;
        for mv in moves.iter() {
            let policy = get_policy(self.position.board(), *mv) / tmp;
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

    fn try_prove_mate_win(node: &mut Node, backprop_mate_dist: i32) -> Option<i32> {
        let move_mate_dist = -backprop_mate_dist + 1;
        let replace = NonZeroI16::new(move_mate_dist as i16).unwrap();
        if let Some(mate_score) = node.mate_score() {
            match mate_score {
                MateScore::Loss(_) => {
                    node.mate_dist = Some(replace);
                    Some(move_mate_dist)
                }
                MateScore::Win(dist) => {
                    if move_mate_dist < dist as i32 {
                        node.mate_dist = Some(replace);
                        Some(move_mate_dist)
                    } else {
                        None
                    }
                }
            }
        } else {
            node.mate_dist = Some(replace);
            Some(move_mate_dist)
        }
    }

    fn try_prove_mate_loss(nodes: &mut Vec<Node>, node_idx: u32) -> Option<i32> {
        // a node is only proven to be a loss if every child is a win for the opponent
        let node = &nodes[node_idx as usize];
        let mut max_dist = 0;
        for child_idx in node.child_indices() {
            let child_node = &nodes[child_idx as usize];
            if let Some(mate_score) = child_node.mate_score() {
                if let MateScore::Win(child_dist) = mate_score {
                    max_dist = max_dist.max(child_dist as i32);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        let node = &mut nodes[node_idx as usize];
        if max_dist > 0 {
            let move_dist = -max_dist - 1;
            let replace = NonZeroI16::new(move_dist as i16).unwrap();
            if let Some(mate_score) = node.mate_score() {
                match mate_score {
                    MateScore::Loss(mate_dist) => {
                        if -move_dist < mate_dist as i32 {
                            node.mate_dist = Some(replace);
                            Some(move_dist)
                        } else {
                            None
                        }
                    }
                    MateScore::Win(_) => {
                        unreachable!()
                    }
                }
            } else {
                node.mate_dist = Some(replace);
                Some(move_dist as i32)
            }
        } else {
            unreachable!()
        }
    }

    fn backprop(&mut self, mut result: f32) {
        let mut child_mate_dist: Option<i32> = None;
        for node_idx in self.selection.iter().rev() {
            if let Some(mate_dist) = child_mate_dist {
                if mate_dist <= 0 {
                    child_mate_dist =
                        Self::try_prove_mate_win(&mut self.nodes[*node_idx as usize], mate_dist);
                } else {
                    child_mate_dist = Self::try_prove_mate_loss(&mut self.nodes, *node_idx as u32);
                }
            }

            let node = &mut self.nodes[*node_idx as usize];

            node.visits += 1;
            node.wins += result;

            if node.result == GameResult::Mated {
                child_mate_dist = Some(0);
            }

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

    fn root_move_score(child_node: &Node) -> f32 {
        match child_node.result {
            GameResult::NonTerminal => {
                if let Some(mate_score) = child_node.mate_score() {
                    match mate_score {
                        MateScore::Loss(dist) => 1000.0 - dist as f32,
                        MateScore::Win(dist) => -(dist as f32),
                    }
                } else {
                    1.0 - child_node.q()
                }
            }
            GameResult::Mated => 1000.0,
            GameResult::Drawn => 0.5,
        }
    }

    fn get_best_move(&self) -> Move {
        let root_node = &self.nodes[0];
        let mut best_score = -1000.0;
        let mut best_move = Move::NULL;
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            if child_node.visits == 0 {
                continue;
            }
            let score = Self::root_move_score(child_node);
            if score > best_score {
                best_score = score;
                best_move = child_node.parent_move;
            }
        }
        best_move
    }

    pub fn display_tree(&self) {
        let root_node = &self.nodes[0];
        for child_idx in root_node.child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            let score_str = if let Some(mate_score) = child_node.mate_score() {
                // child win = parent loss and vice versa
                match mate_score {
                    MateScore::Loss(dist) => format!("win {} plies", dist),
                    MateScore::Win(dist) => format!("loss {} plies", dist),
                }
            } else {
                format!("{} cp", sigmoid_inv(1.0 - child_node.q(), Self::EVAL_SCALE))
            };
            println!(
                "{} => {} visits {}",
                child_node.parent_move, child_node.visits, score_str
            );
        }
    }

    fn get_visit_dist(&self) -> Vec<(Move, f32)> {
        let mut result = Vec::new();
        let total = self.nodes[0].visits;
        for child_idx in self.nodes[0].child_indices() {
            let child_node = &self.nodes[child_idx as usize];
            result.push((
                child_node.parent_move,
                child_node.visits as f32 / total as f32,
            ));
        }
        result
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
            self.build_tree_impl(
                old_nodes,
                old_child_idx,
                new_node.first_child_idx + iter as u32,
            );
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
            self.nodes.push(Node::new(Move::NULL, 0.0));
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
                    let score_str = if let Some(mate_score) = self.nodes[0].mate_score() {
                        match mate_score {
                            MateScore::Loss(dist) => format!("mate {}", -(dist as i32) / 2),
                            MateScore::Win(dist) => format!("mate {}", (dist as i32 + 1) / 2),
                        }
                    } else {
                        format!(
                            "cp {}",
                            sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE).round()
                        )
                    };
                    println!(
                        "info depth {} nodes {} time {} nps {} score {} pv {}",
                        curr_depth,
                        nodes,
                        (elapsed * 1000.0) as u64,
                        (nodes as f64 / elapsed as f64) as u64,
                        score_str,
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
            let score_str = if let Some(mate_score) = self.nodes[0].mate_score() {
                match mate_score {
                    MateScore::Loss(dist) => format!("mate {}", -(dist as i32) / 2),
                    MateScore::Win(dist) => format!("mate {}", (dist as i32 + 1) / 2),
                }
            } else {
                format!(
                    "cp {}",
                    sigmoid_inv(self.nodes[0].q(), Self::EVAL_SCALE).round()
                )
            };
            println!(
                "info depth {} nodes {} time {} nps {} score {} pv {}",
                curr_depth,
                nodes,
                (elapsed * 1000.0) as u64,
                (nodes as f64 / elapsed as f64) as u64,
                score_str,
                self.get_best_move()
            );
        }

        SearchResults {
            best_move: self.get_best_move(),
            nodes: nodes,
            score: if let Some(mate_score) = self.nodes[0].mate_score() {
                Score::Mate(mate_score)
            } else {
                Score::Normal(self.nodes[0].q())
            },
            visit_dist: self.get_visit_dist(),
        }
    }

    pub fn new_game(&mut self) {
        self.nodes.clear();
    }
}
