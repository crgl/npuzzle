use std::io::prelude::*;
use std::fs::File;
use std::env;
use std::collections::HashSet;
use std::collections::BinaryHeap;

struct Quest {
	goal: Vec<Vec<usize>>,
	open: BinaryHeap<Node<'a>>,
	closed: HashSet<Node<'a>>,
	heur: Heuristic,
	greedy: bool,
}

impl<'a> Quest<'a> {
	fn new(board: Vec<Vec<usize>>, heur: Heuristic, greedy: bool, goal: Vec<Vec<usize>>) -> Quest {
		let mut open = BinaryHeap::new();
		open.push(board);
		Quest {
			goal,
			open,
			closed: HashSet::new(),
			heur,
			greedy,
		}
	}
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
enum Heuristic
{
	Hamming,
	Manhattan,
	OutOfLine,
	Nilsson,
}

#[derive(Clone, Hash, Eq)]
struct Node<'a> {
	f: usize,
	g: usize,
	zero: (usize, usize),
	goal: &'a Vec<Vec<usize>>,
	board: Vec<Vec<usize>>,
}

impl<'a> Node<'a> {
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
		let mut out = Node { f: 0, g: 0, zero, goal, board };
		match heur {
			Heuristic::Hamming => out.hamming(greedy),
			Heuristic::Manhattan => out.manhattan(greedy),
			Heuristic::OutOfLine => out.out_of_line(greedy),
			Heuristic::Nilsson => out.nilsson(greedy),
		}
		out
	}

	fn hamming(& mut self, greedy: bool) {
		let mut h = 0;
		if greedy {
			self.f = h;
		}
		else {
			self.f = self.g + h;
		}
	}

	fn manhattan(& mut self, greedy: bool) {
		let mut h = 0;
		if greedy {
			self.f = h;
		}
		else {
			self.f = self.g + h;
		}
	}

	fn out_of_line(& mut self, greedy: bool) {
		let mut h = 0;
		if greedy {
			self.f = h;
		}
		else {
			self.f = self.g + h;
		}
	}

	fn nilsson(& mut self, greedy: bool) {
		let mut h = 0;
		if greedy {
			self.f = h;
		}
		else {
			self.f = self.g + h;
		}
	}
}

impl<'a> Ord for Node<'a> {
	fn cmp(&self, other: &Node) -> std::cmp::Ordering {
		self.f.cmp(&other.f)
	}
}

impl<'a> PartialOrd for Node<'a> {
	fn partial_cmp(&self, other: &Node) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

// ???? Maybe just make it normal
impl<'a> PartialEq for Node<'a> {
	fn eq(&self, other: &Node) -> bool {
		self.f == other.f
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
		out.push(tmp);
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

fn main() {
	let args: Vec<String> = env::args().collect();
	if args.len() < 2 {
		panic!("usage: cargo run puzzle.txt");
	}
	let mut f = File::open(&args[1]).expect("could not open file");
	let mut contents = String::new();
	f.read_to_string(&mut contents).expect("could not read file");
	let puzzle = parse_input(contents).expect("invalid puzzle");
	println!("{:?}", puzzle[0]);
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
