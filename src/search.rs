use std::{num::NonZeroI16, time::Instant};

use crate::{
    chess::{
        movegen::{movegen, MoveList},
        Move,
    },
    eval,
    position::Position,
    tree::{GameResult, MateScore, Node, NodeIndex, Score, Tree},
};

fn sigmoid(x: f32, scale: f32) -> f32 {
    1.0 / (1.0 + (-x / scale).exp())
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
    iters: u32,
    tree: Tree,
    root_position: Position,
    position: Position,
    nodes: u32,
}

impl MCTS {
    const ROOT_CPUCT: f32 = 0.75 * 1.66393528;
    const CPUCT: f32 = 0.75 * 1.06066017;
    pub const EVAL_SCALE: f32 = 187.5;

    pub fn new() -> Self {
        Self {
            tree: Tree::new(24),
            iters: 0,
            root_position: Position::new(),
            position: Position::new(),
            nodes: 0,
        }
    }

    pub fn set_hash(&mut self, hash: u64) {
        self.tree = Tree::new(hash);
    }

    pub fn new_game(&mut self) {
        self.tree.clear();
    }

    fn eval_wdl(&self) -> f32 {
        let board = self.position.board();
        let eval = eval::eval(board);

        sigmoid(eval as f32, Self::EVAL_SCALE)
    }

    fn simulate(&self, ply: i32) -> (f32, GameResult) {
        let mut moves = MoveList::new();
        movegen(self.position.board(), &mut moves);

        let result = if moves.len() == 0 {
            if self.position.board().checkers().any() {
                GameResult::Mated
            } else {
                GameResult::Drawn
            }
        } else if self.position.is_drawn(ply) {
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
                    node.set_mate_dist(Some(replace));
                    Some(move_mate_dist)
                }
                MateScore::Win(dist) => {
                    if move_mate_dist < dist as i32 {
                        node.set_mate_dist(Some(replace));
                        Some(move_mate_dist)
                    } else {
                        None
                    }
                }
            }
        } else {
            node.set_mate_dist(Some(replace));
            Some(move_mate_dist)
        }
    }

    fn try_prove_mate_loss(tree: &mut Tree, node_idx: NodeIndex) -> Option<i32> {
        // a node is only proven to be a loss if every child is a win for the opponent
        let node = &tree[node_idx];
        let mut max_dist = 0;
        for child_idx in node.child_indices() {
            let child_node = &tree[child_idx];
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
        let node = &mut tree[node_idx];
        if max_dist > 0 {
            let move_dist = -max_dist - 1;
            let replace = NonZeroI16::new(move_dist as i16).unwrap();
            if let Some(mate_score) = node.mate_score() {
                match mate_score {
                    MateScore::Loss(mate_dist) => {
                        if -move_dist < mate_dist as i32 {
                            node.set_mate_dist(Some(replace));
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
                node.set_mate_dist(Some(replace));
                Some(move_dist as i32)
            }
        } else {
            unreachable!()
        }
    }

    fn perform_one_impl(&mut self, node_idx: NodeIndex, ply: u32) -> Option<(f32, Option<i32>)> {
        let root = node_idx == self.tree.root_node();
        if self.tree[node_idx].is_terminal() || self.tree[node_idx].visits() == 0 {
            let (score, game_result) = self.simulate(ply as i32);

            let node = &mut self.tree[node_idx];
            node.set_game_result(game_result);
            node.add_score(score);

            self.nodes += ply + 1;

            return Some((
                score,
                if game_result == GameResult::Mated {
                    Some(0)
                } else {
                    None
                },
            ));
        } else {
            // node can't be terminal here, must be unexpanded
            if self.tree[node_idx].child_count() == 0 {
                self.tree.expand_node(node_idx, self.position.board())?;
            }
            self.tree.fetch_children(node_idx);

            let node = &self.tree[node_idx];

            let mut best_uct = -1f32;
            let mut best_child_idx = self.tree.root_node();
            for child_idx in node.child_indices() {
                let child = &self.tree[child_idx];
                let q = if child.visits() == 0 {
                    if root {
                        1000.0
                    } else {
                        node.q()
                    }
                } else {
                    // 1 - child q because child q is from opposite perspective of current node
                    1.0 - child.q()
                };
                let policy = child.policy();
                let expl = (node.visits() as f32).sqrt() / (1 + child.visits()) as f32;
                let mut cpuct = if root { Self::ROOT_CPUCT } else { Self::CPUCT };
                cpuct *= 1.0 + (1.0 + node.visits() as f32 / 1048576.0).ln();
                let uct = q + cpuct * policy * expl;

                if uct > best_uct {
                    best_child_idx = child_idx;
                    best_uct = uct;
                }
            }

            self.position
                .make_move(self.tree[best_child_idx].parent_move());
            let (child_score, mut child_mate_dist) =
                self.perform_one_impl(best_child_idx, ply + 1)?;

            if let Some(mate_dist) = child_mate_dist {
                if mate_dist <= 0 {
                    child_mate_dist = Self::try_prove_mate_win(&mut self.tree[node_idx], mate_dist);
                } else {
                    child_mate_dist = Self::try_prove_mate_loss(&mut self.tree, node_idx);
                }
            }

            let score = 1.0 - child_score;

            let node = &mut self.tree[node_idx];

            node.add_score(score);

            Some((score, child_mate_dist))
        }
    }

    fn perform_one_iter(&mut self) -> Result<(), ()> {
        self.position = self.root_position.clone();
        if self.perform_one_impl(self.tree.root_node(), 0).is_none() {
            return Err(());
        }
        self.iters += 1;
        Ok(())
    }

    fn pv_score(node: &Node) -> f32 {
        match node.score().flip() {
            Score::Win(dist) => 1000.0 - dist as f32,
            Score::Draw => 0.5,
            Score::Loss(dist) => -1000.0 + dist as f32,
            Score::Normal(score) => score,
        }
    }

    fn get_best_move(&self) -> Move {
        let root_node = &self.tree[self.tree.root_node()];
        let mut best_score = -1000.0;
        let mut best_move = Move::NULL;
        for child_idx in root_node.child_indices() {
            let child_node = &self.tree[child_idx];
            if child_node.visits() == 0 {
                continue;
            }
            let score = Self::pv_score(child_node);
            if score > best_score {
                best_score = score;
                best_move = child_node.parent_move();
            }
        }
        best_move
    }

    fn display_tree_impl(&self, node_idx: NodeIndex, depth: i32, ply: i32) {
        if depth <= 0 {
            return;
        }
        let indentation = || "    ".repeat(ply as usize);
        let node = &self.tree[node_idx];
        let mut children: Vec<NodeIndex> = node.child_indices().collect();
        children.sort_by(|a, b| self.tree[*b].visits().cmp(&self.tree[*a].visits()));
        for child_idx in children {
            let child_node = &self.tree[child_idx];
            println!(
                "{}{} => {} visits {}",
                indentation(),
                child_node.parent_move(),
                child_node.visits(),
                child_node.score()
            );
            self.display_tree_impl(child_idx, depth - 1, ply + 1);
        }
    }

    pub fn display_tree(&self, depth: i32) {
        self.display_tree_impl(self.tree.root_node(), depth, 0);
    }

    fn get_visit_dist(&self) -> Vec<(Move, f32)> {
        let mut result = Vec::new();
        let total = self.tree[self.tree.root_node()].visits();
        for child_idx in self.tree[self.tree.root_node()].child_indices() {
            let child_node = &self.tree[child_idx];
            result.push((
                child_node.parent_move(),
                child_node.visits() as f32 / total as f32,
            ));
        }
        result
    }

    // depth 2 perft to find the node
    fn find_node(&self, position: &Position) -> NodeIndex {
        if self.tree.size() == 0 {
            return NodeIndex::NULL;
        }
        let root_node = &self.tree[self.tree.root_node()];
        for child_idx in root_node.child_indices() {
            let child_node = &self.tree[child_idx];
            let mut new_pos = self.root_position.clone();
            new_pos.make_move(child_node.parent_move());
            for child2_idx in child_node.child_indices() {
                let child2_node = &self.tree[child2_idx];
                let mut new_pos2 = new_pos.clone();
                new_pos2.make_move(child2_node.parent_move());
                if new_pos2 == *position {
                    return child2_idx;
                }
            }
        }
        NodeIndex::NULL
    }

    fn depth(&self) -> u32 {
        (self.nodes - self.iters) / self.iters.max(1)
    }

    pub fn run(
        &mut self,
        limits: SearchLimits,
        report: bool,
        position: &Position,
    ) -> SearchResults {
        let new_root_idx = self.find_node(position);

        self.root_position = position.clone();
        self.position = self.root_position.clone();
        self.iters = 0;
        self.nodes = 0;

        if new_root_idx != NodeIndex::NULL && self.tree[new_root_idx].child_count() > 0 {
            self.tree.set_as_root(new_root_idx);
            self.tree
                .relabel_policies(self.tree.root_node(), &self.root_position.board());
        } else {
            self.tree.clear();
            self.tree.add_root_node();
            self.tree
                .expand_node(self.tree.root_node(), self.root_position.board())
                .expect("Cannot expand root node in tree");
            let eval = self.eval_wdl();
            let root = self.tree.root_node();
            self.tree[root].add_score(eval);
        }

        let mut prev_depth = 0;

        let start_time = Instant::now();

        while limits.max_nodes < 0 || self.iters <= limits.max_nodes as u32 {
            let result = self.perform_one_iter();
            if result.is_err() {
                self.tree.flip();
                continue;
            }

            let curr_depth = self.depth();
            if curr_depth > prev_depth {
                if limits.max_depth > 0 && curr_depth >= limits.max_depth as u32 {
                    break;
                }

                prev_depth = curr_depth;
                if report {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    println!(
                        "info depth {} nodes {} time {} nps {} score {} pv {}",
                        curr_depth,
                        self.nodes,
                        (elapsed * 1000.0) as u64,
                        (self.nodes as f64 / elapsed as f64) as u64,
                        self.tree[self.tree.root_node()].score().uci_str(),
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
            let curr_depth = self.depth();
            let elapsed = start_time.elapsed().as_secs_f64();
            println!(
                "info depth {} nodes {} time {} nps {} score {} pv {}",
                curr_depth,
                self.nodes,
                (elapsed * 1000.0) as u64,
                (self.nodes as f64 / elapsed as f64) as u64,
                self.tree[self.tree.root_node()].score().uci_str(),
                self.get_best_move()
            );
        }

        SearchResults {
            best_move: self.get_best_move(),
            nodes: self.nodes as u64,
            score: self.tree[self.tree.root_node()].score(),
            visit_dist: self.get_visit_dist(),
        }
    }
}
