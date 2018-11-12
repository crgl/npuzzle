use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum Heuristic
{
	Hamming,
	Manhattan,
	OutOfLine,
	Nilsson,
}

#[derive(Copy, Clone)]
enum Direction
{
	Up,
	Down,
	Left,
	Right,
}

struct Quest {
	goal: Vec<Vec<usize>>,
	open: BinaryHeap<Node>,
	closed: HashSet<Vec<Vec<usize>>>,
	heur: Heuristic,
	greedy: bool,
	max_space: usize,
}

impl Quest {
	fn new(board: Vec<Vec<usize>>, heur: Heuristic, greedy: bool, goal: Vec<Vec<usize>>) -> Quest {
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

	fn insoluble(&self) -> bool {
		let board = self.open.peek().unwrap().board_ref();
		let len = board.len();
		let n = len * len;
		let mut inv_board = 0;
		let mut inv_goal = 0;
		let mut zero_row_board = 0;
		let mut zero_row_goal = 0;
		let mut weights_board : Vec<usize> = std::iter::repeat(0).take(n).collect();
		let mut weights_goal = weights_board.clone();
		for i in 0..len {
			for j in 0..len {
				inv_board += weights_board[board[i][j]];
				inv_goal += weights_goal[self.goal[i][j]];
				if board[i][j] == 0 {
					zero_row_board = i;
				}
				if self.goal[i][j] == 0 {
					zero_row_goal = i;
				}
				for k in 1..board[i][j] {
					weights_board[k] += 1;
				}
				for k in 1..self.goal[i][j] {
					weights_goal[k] += 1;
				}
			}
		}
		let check = (inv_board + inv_goal + ((zero_row_board + zero_row_goal) % 2) * (len % 2)) % 2;
		if check == 0 {
			true
		}
		else {
			false
		}
	}

	fn step(& mut self) -> Option<Node> {
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

	fn print_goal(&self) {
		for row in self.goal.iter() {
			println!("{:?}", row);
		}
	}

	fn peek(&self) -> Option<&Node> {
		self.open.peek()
	}

	fn continues(&self) -> bool {
		!self.open.is_empty()
	}

	fn space(&self) -> usize {
		self.max_space
	}

	fn time(&self) -> usize {
		self.closed.len()
	}
}

#[derive(Clone, Hash, Eq)]
struct Node {
	f: i64,
	g: i64,
	pub path: Vec<(usize, usize)>,
	board: Vec<Vec<usize>>,
}

impl Node {
	fn new(board: Vec<Vec<usize>>, heur: Heuristic, greedy: bool, goal: &Vec<Vec<usize>>) -> Node {
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
		let mut out = Node { f: 0, g: 0, path, board };
		match heur {
			Heuristic::Hamming => out.hamming(goal, greedy),
			Heuristic::Manhattan => out.manhattan(goal, greedy),
			Heuristic::OutOfLine => out.out_of_line(goal, greedy),
			Heuristic::Nilsson => out.nilsson(goal, greedy),
		}
		out
	}

	fn shift(&self, dir: Direction, heur: Heuristic, greedy: bool, goal: &Vec<Vec<usize>>) -> Self {
		let mut out = self.clone();
		out.swap(dir);
		match heur {
			Heuristic::Hamming => out.hamming(goal, greedy),
			Heuristic::Manhattan => out.manhattan(goal, greedy),
			Heuristic::OutOfLine => out.out_of_line(goal, greedy),
			Heuristic::Nilsson => out.nilsson(goal, greedy),
		}
		out
	}

	fn swap(& mut self, dir: Direction) {
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

	fn inc(& mut self) {
		self.g += 1;
	}

	fn dist(&self) -> i64 {
		-1 * (self.f + self.g)
	}

	fn steps(&self) -> Vec<(usize, usize)> {
		self.path.clone()
	}

	fn board_ref(&self) -> &Vec<Vec<usize>> {
		&self.board
	}

	fn into_board(self) -> Vec<Vec<usize>> {
		self.board
	}

	fn print_board(&self) {
		for row in self.board.iter() {
			println!("{:?}", row);
		}
	}

	fn hamming(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		let mut h : i64 = 0;
		let n = goal.len();
		for i in 0..n {
			for j in 0..n {
				if self.board[i][j] != goal[i][j] {
					h += 1;
				}
			}
		}
		if greedy {
			self.f = -1 * h;
		}
		else {
			self.f = -1 * (self.g + h);
		}
	}

	fn manhattan(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		let mut h : i64 = 0;
		let len = goal.len();
		let n = len * len;
		let mut bdp : Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
		let mut glp = bdp.clone();
		for i in 0..len {
			for j in 0..len {
				bdp[self.board[i][j]] = (i, j);
				glp[goal[i][j]] = (i, j);
			}
		}
		for i in 0..n {
			h += (std::cmp::max(bdp[i].0, glp[i].0) - std::cmp::min(bdp[i].0, glp[i].0)) as i64;
			h += (std::cmp::max(bdp[i].1, glp[i].1) - std::cmp::min(bdp[i].1, glp[i].1)) as i64;
		}
		if greedy {
			self.f = -1 * h;
		}
		else {
			self.f = -1 * (self.g + h);
		}
	}

	fn out_of_line(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		let mut h = 0;
		let len = goal.len();
		let n = len * len;
		let mut bdp : Vec<(usize, usize)> = std::iter::repeat((0, 0)).take(n).collect();
		let mut glp = bdp.clone();
		for i in 0..len {
			for j in 0..len {
				bdp[self.board[i][j]] = (i, j);
				glp[goal[i][j]] = (i, j);
			}
		}
		for i in 0..n {
			h += match (bdp[i].0 == glp[i].0, bdp[i].1 == glp[i].1) {
				(true, true) => 0,
				(true, false) | (false, true) => 1,
				(false, false) => 2,
			};
		}
		if greedy {
			self.f = -1 * h;
		}
		else {
			self.f = -1 * (self.g + h);
		}
	}

	fn nilsson(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		let mut h = 0;
		if greedy {
			self.f = -1 * h;
		}
		else {
			self.f = -1 * (self.g + h);
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

fn parse_input(contents : String) -> Result<Vec<Vec<usize>>, &'static str> {
	let mut out = Vec::new();
	let lines = contents.lines();
	for line in lines {
		let words = line.split_whitespace();
		let mut tmp = Vec::new();
		for word in words {
			if word == "#" {
				break ;
			}
			tmp.push(word.parse::<usize>().unwrap_or(0));
		}
		if !tmp.is_empty() {
			out.push(tmp);
		}
	}
	if out.is_empty() {
		return Err("no input");
	}
	let len = out[0].len();
	let mut height = 0;
	let mut s: HashSet<usize> = HashSet::new();
	for row in out.iter() {
		if row.len() != len {
			return Err("not a square");
		}
		for e in row.iter() {
			if s.contains(e) {
				return Err("duplicate value");
			}
			s.insert(*e);
		}
		height += 1;
	}
	if height != len {
		return Err("not a square");
	}
	for i in 0..(len * len) {
		if !s.contains(&i) {
			return Err("invalid value");
		}
	}
	Ok(out)
}

fn refine(puzzle: Vec<Vec<usize>>) -> Quest {
	let n = puzzle.len();
	let mut goal = Vec::new();
	for r in 0..n {
		let mut row = Vec::new();
		for c in 0..n {
			row.push(0);
		}
		goal.push(row);
	}
	let mut base = 0;
	for i in 0..(n / 2) {
		for j in i..(n - i - 1) {
			let common = base + j - i + 1;
			let diff = n - 2 * i - 1;
			goal[i][j] = common;
			goal[j][n - i - 1] = common + diff;
			goal[n - i - 1][n - j - 1] = common + 2 * diff;
			goal[n - j - 1][i] = common + 3 * diff;
		}
		base += (n - 2 * i - 1) * 4;
	}
	goal[n / 2][(n - 1) / 2] = 0;
	Quest::new(puzzle, Heuristic::Manhattan, false, goal)
}

fn solverize(mut quest: Quest) {
	while quest.continues() {
		let mut out = match quest.step() {
			Some(output) => output,
			None => continue,
		};
		println!("space: {}", quest.space());
		println!("time: {}", quest.time());
		println!("steps: {}", out.steps().len() - 1);
		println!("dist: {}", out.dist());
		return ;
	}
	println!("Unstackable cups!");
}

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		panic!("usage: cargo run puzzle.txt");
	}
	let mut f = File::open(&args[1]).expect("could not open file");
	let mut contents = String::new();
	f.read_to_string(&mut contents).expect("could not read file");
	let puzzle = parse_input(contents).expect("invalid puzzle");
	let quest = refine(puzzle);
	if quest.insoluble() {
		println!("Unstackable cups!");
	}
	else {
		solverize(quest);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reject_empty() {
		let mut f = File::open("puzzles/parsing/empty.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		assert_eq!("no input", parse_input(contents).expect_err("returned"));
	}

	#[test]
	fn reject_oblong() {
		let mut f = File::open("puzzles/parsing/oblong.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		assert_eq!("not a square", parse_input(contents).expect_err("returned"));
	}

	#[test]
	fn reject_rect() {
		let mut f = File::open("puzzles/parsing/rect.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		assert_eq!("not a square", parse_input(contents).expect_err("returned"));
	}

	#[test]
	fn reject_invalid() {
		let mut f = File::open("puzzles/parsing/wrong_nums.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		assert_eq!("invalid value", parse_input(contents).expect_err("returned"));
	}

	#[test]
	fn reject_dup() {
		let mut f = File::open("puzzles/parsing/dups.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		assert_eq!("duplicate value", parse_input(contents).expect_err("returned"));
	}

	#[test]
	fn accept_valid() {
		let mut f = File::open("puzzles/parsing/clean.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		let puzzle = parse_input(contents).expect("Error");
		assert!(puzzle.len() == 3);
		for i in 0..3 {
			assert!(puzzle[i].len() == 3);
		}
	}
}
