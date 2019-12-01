use std::cmp::Ordering;

use crate::Direction;
pub use crate::Heuristic;

#[derive(Clone, Hash, Eq)]
pub struct Node {
    f: i64,
    g: i64,
    h: i64,
    pub path: Vec<(usize, usize)>,
    board: Vec<Vec<usize>>,
}

impl Node {
    pub fn new(
        board: Vec<Vec<usize>>,
        heur: Heuristic,
        greedy: bool,
        goal: &Vec<Vec<usize>>,
    ) -> Node {
        let mut i = 0;
        let zero = 'outer: loop {
            for j in 0..(board.len()) {
                if board[i][j] == 0 {
                    break 'outer (i, j);
                }
            }
            i += 1;
            if i == board.len() {
                panic!();
            }
        };
        let mut path = Vec::new();
        path.push(zero);
        let mut out = Node {
            f: 0,
            g: 0,
            h: 0,
            path,
            board,
        };
        match heur {
            Heuristic::Hamming => out.hamming(goal, greedy),
            Heuristic::Manhattan => out.manhattan(goal, greedy),
            Heuristic::OutOfLine => out.out_of_line(goal, greedy),
            Heuristic::Nilsson => out.nilsson(goal, greedy),
            Heuristic::Custom => out.custom(goal, greedy),
        }
        out
    }

    pub(crate) fn shift(
        &self,
        dir: Direction,
        heur: Heuristic,
        greedy: bool,
        goal: &Vec<Vec<usize>>,
    ) -> Self {
        let mut out = self.clone();
        out.swap(dir);
        match heur {
            Heuristic::Hamming => out.hamming(goal, greedy),
            Heuristic::Manhattan => out.manhattan(goal, greedy),
            Heuristic::OutOfLine => out.out_of_line(goal, greedy),
            Heuristic::Nilsson => out.nilsson(goal, greedy),
            Heuristic::Custom => out.custom(goal, greedy),
        }
        out
    }

    fn swap(&mut self, dir: Direction) {
        let &curr = self.path.last().unwrap();
        let next = match dir {
            Direction::Up => (curr.0 - 1, curr.1),
            Direction::Down => (curr.0 + 1, curr.1),
            Direction::Left => (curr.0, curr.1 - 1),
            Direction::Right => (curr.0, curr.1 + 1),
        };
        self.board[curr.0][curr.1] = self.board[next.0][next.1];
        self.board[next.0][next.1] = 0;
        self.path.push(next);
        self.inc();
    }

    fn inc(&mut self) {
        self.g += 1;
    }

    pub fn dist(&self) -> i64 {
        self.h
    }

    pub fn steps(&self) -> Vec<(usize, usize)> {
        self.path.clone()
    }

    pub fn board_ref(&self) -> &Vec<Vec<usize>> {
        &self.board
    }

    pub fn into_board(self) -> Vec<Vec<usize>> {
        self.board
    }

    fn _print_board(&self) {
        for row in self.board.iter() {
            println!("{:?}", row);
        }
    }

    fn hamming(&mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
        self.h = 0;
        let n = goal.len();
        for i in 0..n {
            for j in 0..n {
                if self.board[i][j] != goal[i][j] {
                    self.h += 1;
                }
            }
        }
        if greedy {
            self.f = -1 * self.h;
        } else {
            self.f = -1 * (self.g + self.h);
        }
    }

    fn manhattan(&mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
        self.h = 0;
        let len = goal.len();
        let n = len * len;
        let mut bdp: Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
        let mut glp = bdp.clone();
        for i in 0..len {
            for j in 0..len {
                bdp[self.board[i][j]] = (i, j);
                glp[goal[i][j]] = (i, j);
            }
        }
        for i in 0..n {
            self.h +=
                (std::cmp::max(bdp[i].0, glp[i].0) - std::cmp::min(bdp[i].0, glp[i].0)) as i64;
            self.h +=
                (std::cmp::max(bdp[i].1, glp[i].1) - std::cmp::min(bdp[i].1, glp[i].1)) as i64;
        }
        if greedy {
            self.f = -1 * self.h;
        } else {
            self.f = -1 * (self.g + self.h);
        }
    }

    fn out_of_line(&mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
        self.h = 0;
        let len = goal.len();
        let n = len * len;
        let mut bdp: Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
        let mut glp = bdp.clone();
        for i in 0..len {
            for j in 0..len {
                bdp[self.board[i][j]] = (i, j);
                glp[goal[i][j]] = (i, j);
            }
        }
        for i in 0..n {
            self.h += match (bdp[i].0 == glp[i].0, bdp[i].1 == glp[i].1) {
                (true, true) => 0,
                (true, false) | (false, true) => 1,
                (false, false) => 2,
            };
        }
        if greedy {
            self.f = -1 * self.h;
        } else {
            self.f = -1 * (self.g + self.h);
        }
    }

    fn nilsson(&mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
        self.h = 0;
        let len = goal.len();
        let n = len * len;
        let mut bdp: Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
        let mut glp = bdp.clone();
        for i in 0..len {
            for j in 0..len {
                bdp[self.board[i][j]] = (i, j);
                glp[goal[i][j]] = (i, j);
            }
        }
        for i in 0..n {
            self.h +=
                (std::cmp::max(bdp[i].0, glp[i].0) - std::cmp::min(bdp[i].0, glp[i].0)) as i64;
            self.h +=
                (std::cmp::max(bdp[i].1, glp[i].1) - std::cmp::min(bdp[i].1, glp[i].1)) as i64;
        }
        for i in 0..(len / 2) {
            for j in i..(len - i - 1) {
                if (self.board[i][j] != 0 && self.board[i][j] != n - 1)
                    && self.board[i][j] + 1 != self.board[i][j + 1]
                {
                    self.h += 6;
                }
                if (self.board[j][len - i - 1] != 0 && self.board[j][len - i - 1] != n - 1)
                    && self.board[j][len - i - 1] + 1 != self.board[j + 1][len - i - 1]
                {
                    self.h += 6;
                }
                if (self.board[len - i - 1][len - j - 1] != 0
                    && self.board[len - i - 1][len - j - 1] != n - 1)
                    && self.board[len - i - 1][len - j - 1] + 1
                        != self.board[len - i - 1][len - j - 2]
                {
                    self.h += 6;
                }
                if (self.board[len - j - 1][i] != 0 && self.board[len - j - 1][i] != n - 1)
                    && self.board[len - j - 1][i] + 1 != self.board[len - j - 2][i]
                {
                    if i == (len - 1) / 2 {
                        self.h += 6;
                    }
                }
            }
        }
        if self.board[len / 2][(len - 1) / 2] != 0 {
            self.h += 3;
        }
        if greedy {
            self.f = -1 * self.h;
        } else {
            self.f = -1 * (self.g + self.h);
        }
    }

    fn custom(&mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
        self.h = 0;
        let len = goal.len();
        let n = len * len;
        let mut bdp: Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
        let mut glp = bdp.clone();
        for i in 0..len {
            for j in 0..len {
                bdp[self.board[i][j]] = (i, j);
                glp[goal[i][j]] = (i, j);
            }
        }
        for i in 0..n {
            let tmp = (bdp[i].0 as i64 - glp[i].0 as i64).abs()
                + (bdp[i].1 as i64 - glp[i].1 as i64).abs();
            let to_add = 10 * tmp;
            self.h += to_add;
        }
        if greedy {
            self.f = -1 * self.h;
        } else {
            self.f = -1 * (self.g + self.h);
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Node) -> Ordering {
        match self.f.cmp(&other.f) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => self.g.cmp(&other.g),
            Ordering::Greater => Ordering::Greater,
        }
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Node) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ???? Maybe just make it normal
impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.f == other.f && self.g == other.g
    }
}
