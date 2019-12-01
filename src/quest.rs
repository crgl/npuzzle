use std::collections::BinaryHeap;
use std::collections::HashSet;

use crate::node::Node;
use crate::Direction;
use crate::Heuristic;

use crate::insoluble as insolucrate;

pub struct Quest {
    goal: Vec<Vec<usize>>,
    open: BinaryHeap<Node>,
    closed: HashSet<Vec<Vec<usize>>>,
    heur: Heuristic,
    greedy: bool,
    max_space: usize,
}

impl Quest {
    pub fn new(
        board: Vec<Vec<usize>>,
        heur: Heuristic,
        greedy: bool,
        goal: Vec<Vec<usize>>,
    ) -> Quest {
        let mut open = BinaryHeap::new();
        open.push(Node::new(board, heur, greedy, &goal));
        Quest {
            goal,
            open,
            closed: HashSet::new(),
            heur,
            greedy,
            max_space: 1,
        }
    }

    pub fn insoluble(&self) -> bool {
        let board = self.open.peek().unwrap().board_ref();
        insolucrate(board, Some(self.get_goal()))
    }

    pub fn step(&mut self) -> Option<Node> {
        if self.open.is_empty() {
            return None;
        }
        let to_search = self.open.pop().unwrap();
        if self.closed.contains(to_search.board_ref()) {
            return None;
        }
        if to_search.dist() == 0 {
            return Some(to_search);
        }
        let &(y, x) = to_search.path.last().unwrap();
        if x > 0 {
            let to_push = to_search.shift(Direction::Left, self.heur, self.greedy, &self.goal);
            if !self.closed.contains(to_push.board_ref()) {
                self.open.push(to_push);
            }
        }
        if x < self.goal.len() - 1 {
            let to_push = to_search.shift(Direction::Right, self.heur, self.greedy, &self.goal);
            if !self.closed.contains(to_push.board_ref()) {
                self.open.push(to_push);
            }
        }
        if y > 0 {
            let to_push = to_search.shift(Direction::Up, self.heur, self.greedy, &self.goal);
            if !self.closed.contains(to_push.board_ref()) {
                self.open.push(to_push);
            }
        }
        if y < self.goal.len() - 1 {
            let to_push = to_search.shift(Direction::Down, self.heur, self.greedy, &self.goal);
            if !self.closed.contains(to_push.board_ref()) {
                self.open.push(to_push);
            }
        }
        self.closed.insert(to_search.into_board());
        if self.open.len() > self.max_space {
            self.max_space = self.open.len();
        }
        None
    }

    pub fn get_goal(&self) -> Vec<Vec<usize>> {
        self.goal.clone()
    }

    pub fn continues(&self) -> bool {
        !self.open.is_empty()
    }

    pub fn space(&self) -> usize {
        self.max_space
    }

    pub fn time(&self) -> usize {
        self.closed.len()
    }
}
