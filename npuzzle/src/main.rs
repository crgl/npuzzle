use std::io::prelude::*;
use std::fs::File;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

use rand::{thread_rng, Rng};

use clap::{Arg, App};

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL, Texture };
use piston_window::TextureSettings;
use std::path::Path;

pub struct Game {
	gl: GlGraphics,
	zero: (usize, usize),
	board: Vec<Vec<usize>>,
	goal: Vec<Vec<usize>>,
	imgref: Vec<(graphics::Image, Texture)>,
	complete: bool,
	missing: (graphics::Image, Texture),
}

impl Game {
	fn new(gl: GlGraphics, board: Vec<Vec<usize>>, goal: Vec<Vec<usize>>, width: u32) -> Game {
		use graphics::*;

		let n = goal.len();
		let width = width as usize;
		let def_t = TextureSettings::new();
		let mut imgref : Vec<(Image, Texture)> = Vec::with_capacity(n * n);
		for _ in 0..(n * n) {
			let piece = Image::new().rect(rectangle::square(0.0, 0.0, (width / n) as f64 - 1.));
			let texture = Texture::from_path(Path::new("assets/argyle.png"), &def_t).unwrap();
			imgref.push((piece, texture));
		}
		let piece = Image::new().rect(rectangle::square(0.0, 0.0, (width / n) as f64 - 1.));
		let texture = Texture::from_path(Path::new("assets/argyle.png"), &def_t).unwrap();
		let mut missing = (piece, texture);
		let mut zero = (0, 0);
		if n < 8 {
			for i in 0..n {
				for j in 0..n {
					let mut p = "assets/split-2018-11-14/".to_owned();
					p.push_str(&(i.to_string()));
					p.push_str(&(j.to_string()));
					p.push_str(".png");
					if goal[i][j] != 0 {
						imgref[goal[i][j]].1 = Texture::from_path(Path::new(&p), &def_t).unwrap();
					}
					else {
						missing.1 = Texture::from_path(Path::new(&p), &def_t).unwrap();
						imgref[goal[i][j]].1 = Texture::from_path(Path::new("assets/Solid_black.png"), &def_t).unwrap();
					}
					if board[i][j] == 0 {
						zero = (i, j);
					}
				}
			}
		}
		let complete = board == goal;
		Game {
			gl,
			zero,
			board,
			goal,
			imgref,
			complete,
			missing,
		}
	}

	fn render(&mut self, args: &RenderArgs) {
		use graphics::*;

		const BLACK: [f32; 4] = [0.0; 4];

		let n = self.board.len() as u32;
		let def_d = DrawState::default();

		self.gl.draw(args.viewport(), |c, gl| {
			clear(BLACK, gl);
		});
		for i in 0..n {
			for j in 0..n {
				let (x, y) = ((j * args.width / n) as f64,
							  (i * args.height / n) as f64);
				let val = self.board[i as usize][j as usize];
				let ref piece = self.imgref[val].0;
				let ref texture = self.imgref[val].1;
				self.gl.draw(args.viewport(), |c, gl| {
					let transform = c.transform.trans(x, y);
					piece.draw(texture, &def_d, transform, gl)
				});
				if self.complete && val == 0 {
					let ref piece = self.missing.0;
					let ref texture = self.missing.1;
					self.gl.draw(args.viewport(), |c, gl| {
						let transform = c.transform.trans(x, y);
						piece.draw(texture, &def_d, transform, gl)
					});
				}
			}
		}
	}

	fn moveroo(&mut self, args: &Button) {
		use crate::Direction::*;
		if self.complete {
			return ;
		}
		let dir = match &args {
			Button::Keyboard(Key::Right) => Left,
			Button::Keyboard(Key::Left) => Right,
			Button::Keyboard(Key::Down) => Up,
			Button::Keyboard(Key::Up) => Down,
			_ => return,
		};
		match (dir, self.zero.0 > 0, self.zero.0 < self.goal.len() - 1, self.zero.1 > 0, self.zero.1 < self.goal.len() - 1) {
			(Right, _, _, _, true) => {
				self.board[self.zero.0][self.zero.1] = self.board[self.zero.0][self.zero.1 + 1];
				self.board[self.zero.0][self.zero.1 + 1] = 0;
				self.zero.1 += 1;
			},
			(Left, _, _, true, _) => {
				self.board[self.zero.0][self.zero.1] = self.board[self.zero.0][self.zero.1 - 1];
				self.board[self.zero.0][self.zero.1 - 1] = 0;
				self.zero.1 -= 1;
			},
			(Down, _, true, _, _) => {
				self.board[self.zero.0][self.zero.1] = self.board[self.zero.0 + 1][self.zero.1];
				self.board[self.zero.0 + 1][self.zero.1] = 0;
				self.zero.0 += 1;
			},
			(Up, true, _, _, _) => {
				self.board[self.zero.0][self.zero.1] = self.board[self.zero.0 - 1][self.zero.1];
				self.board[self.zero.0 - 1][self.zero.1] = 0;
				self.zero.0 -= 1;
			},
			_ => {},
		}
		self.complete = self.board == self.goal;
	}
}

pub struct Viz {
	gl: GlGraphics, // OpenGL drawing backend.
	step: usize, // Current location in sequence 
	steps: Vec<(usize, usize)>, // sequence
	board: Vec<Vec<usize>>, // starting board
	imgref: Vec<(graphics::Image, Texture)>, // images
}

impl Viz {
	fn new(gl: GlGraphics, steps: Vec<(usize, usize)>, board: Vec<Vec<usize>>, goal: &Vec<Vec<usize>>, width: u32) -> Viz {
		use graphics::*;

		let n = goal.len();
		let width = width as usize;
		let def_t = TextureSettings::new();
		let mut imgref : Vec<(Image, Texture)> = Vec::with_capacity(n * n);
		for _ in 0..(n * n) {
			let piece = Image::new().rect(rectangle::square(0.0, 0.0, (width / n) as f64 - 1.));
			let texture = Texture::from_path(Path::new("assets/argyle.png"), &def_t).unwrap();
			imgref.push((piece, texture));
		}
		if n < 8 {
			for i in 0..n {
				for j in 0..n {
					let mut p = "assets/split-2018-11-14/".to_owned();
					p.push_str(&(i.to_string()));
					p.push_str(&(j.to_string()));
					p.push_str(".png");
					if goal[i][j] != 0 {
						imgref[goal[i][j]].1 = Texture::from_path(Path::new(&p), &def_t).unwrap();
					}
					else {
						imgref[goal[i][j]].1 = Texture::from_path(Path::new("assets/Solid_black.png"), &def_t).unwrap();
					}
				}
			}
		}
		Viz {
			gl,
			step: 0,
			steps,
			board,
			imgref,
		}
	}

	fn render(&mut self, args: &RenderArgs) {
		use graphics::*;

		const BLACK: [f32; 4] = [0.0; 4];
		let mut color: [f32; 4] = [0.0; 4];

		let n = self.board.len() as u32;
		let def_d = DrawState::default();

		self.gl.draw(args.viewport(), |c, gl| {
			clear(BLACK, gl);
		});
		for i in 0..n {
			for j in 0..n {
				let (x, y) = ((j * args.width / n) as f64,
							  (i * args.height / n) as f64);
				let val = self.board[i as usize][j as usize];
				let ref piece = self.imgref[val].0;
				let ref texture = self.imgref[val].1;
				self.gl.draw(args.viewport(), |c, gl| {
					let transform = c.transform.trans(x, y);
					piece.draw(texture, &def_d, transform, gl)
				});
			}
		}
	}

	fn update(&mut self, args: &UpdateArgs) {
		if self.step < self.steps.len() - 1 {
			self.step += 1;
			let (y1, x1) = self.steps[self.step - 1];
			let (y2, x2) = self.steps[self.step];
			self.board[y1][x1] = self.board[y2][x2];
			self.board[y2][x2] = 0;
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
		let check = (inv_board + inv_goal + ((zero_row_board + zero_row_goal) % 2) * ((len + 1) % 2)) % 2;
		if check == 0 {
			false
		}
		else {
			true
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

	fn get_goal(&self) -> Vec<Vec<usize>> {
		self.goal.clone()
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
	h: i64,
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
		let mut out = Node { f: 0, g: 0, h: 0, path, board };
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
		self.h
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
		}
		else {
			self.f = -1 * (self.g + self.h);
		}
	}

	fn manhattan(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		self.h = 0;
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
			self.h += (std::cmp::max(bdp[i].0, glp[i].0) - std::cmp::min(bdp[i].0, glp[i].0)) as i64;
			self.h += (std::cmp::max(bdp[i].1, glp[i].1) - std::cmp::min(bdp[i].1, glp[i].1)) as i64;
		}
		if greedy {
			self.f = -1 * self.h;
		}
		else {
			self.f = -1 * (self.g + self.h);
		}
	}

	fn out_of_line(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		self.h = 0;
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
			self.h += match (bdp[i].0 == glp[i].0, bdp[i].1 == glp[i].1) {
				(true, true) => 0,
				(true, false) | (false, true) => 1,
				(false, false) => 2,
			};
		}
		if greedy {
			self.f = -1 * self.h;
		}
		else {
			self.f = -1 * (self.g + self.h);
		}
	}

	fn nilsson(& mut self, goal: &Vec<Vec<usize>>, greedy: bool) {
		self.h = 0;
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
			self.h += (std::cmp::max(bdp[i].0, glp[i].0) - std::cmp::min(bdp[i].0, glp[i].0)) as i64;
			self.h += (std::cmp::max(bdp[i].1, glp[i].1) - std::cmp::min(bdp[i].1, glp[i].1)) as i64;
		}
		for i in 0..(len / 2) {
			for j in i..(len - i - 1) {
				if (self.board[i][j] != 0 && self.board[i][j] != n - 1) && self.board[i][j] + 1 != self.board[i][j + 1] {
					self.h += 6;
				}
				if (self.board[j][len - i - 1] != 0 && self.board[j][len - i - 1] != n - 1) && self.board[j][len - i - 1] + 1 != self.board[j + 1][len - i - 1] {
					self.h += 6;
				}
				if (self.board[len - i - 1][len - j - 1] != 0 && self.board[len - i - 1][len - j - 1] != n - 1) && self.board[len - i - 1][len - j - 1] + 1 != self.board[len - i - 1][len - j - 2] {
					self.h += 6;
				}
				if (self.board[len - j - 1][i] != 0 && self.board[len - j - 1][i] != n - 1) && self.board[len - j - 1][i] + 1 != self.board[len - j - 2][i] {
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
		}
		else {
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

fn parse_input(contents : String) -> Result<Vec<Vec<usize>>, &'static str> {
	let mut out = Vec::new();
	let lines = contents.lines();
	for line in lines {
		let words = line.split_whitespace();
		let mut tmp = Vec::new();
		for word in words {
			if word.chars().next().unwrap() == '#' {
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

fn refine(puzzle: Vec<Vec<usize>>, heur: Heuristic, greedy: bool) -> Quest {
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
	Quest::new(puzzle, heur, greedy, goal)
}

fn solverize(mut quest: Quest) -> Vec<(usize, usize)> {
	while quest.continues() {
		let out = match quest.step() {
			Some(output) => output,
			None => continue,
		};
		println!("space: {}", quest.space());
		println!("time: {}", quest.time());
		println!("steps: {}", out.steps().len() - 1);
		println!("dist: {}", out.dist());
		return out.steps().clone();
	}
	println!("Unstackable cups!");
	return Vec::new();
}

fn puzzle_gen(len: usize) -> Vec<Vec<usize>> {
	let mut arr = Vec::with_capacity(len * len);
	for x in 0..(len*len) {
		arr.push(x);
	}
	thread_rng().shuffle(&mut arr);
	let mut out = Vec::with_capacity(len);
	for n in 0..len {
		let mut row = Vec::with_capacity(len);
		for i in 0..len {
			row.push(arr.pop().unwrap());
		}
		out.push(row);
	}
	let mut tmp = File::create("puzzles/tmp.txt").unwrap();
	for row in out.iter() {
		for e in row.iter() {
			write!(tmp, "{} ", e).expect("Could not write to tmp file");
		}
		tmp.write(b"\n").expect("Could not write to tmp file");
	}
	out
}

fn let_me_try(mut game: Game, mut window: Window) {
	let mut events = Events::new(EventSettings::new());
	let ufps = 8;
	events.set_max_fps(ufps);
	events.set_ups(ufps);
	while let Some(e) = events.next(&mut window) {
		if let Some(r) = e.render_args() {
			game.render(&r);
		}
		if let Some(p) = e.press_args() {
			game.moveroo(&p);
		}
	}
}

fn do_it(mut viz: Viz, mut window: Window) {
	let mut events = Events::new(EventSettings::new());
	let ufps = 8;
	events.set_max_fps(ufps);
	events.set_ups(ufps);
	while let Some(e) = events.next(&mut window) {
		if let Some(r) = e.render_args() {
			viz.render(&r);
		}
		if let Some(u) = e.update_args() {
			viz.update(&u);
		}
	}
}

fn main() {
	let matches = App::new("npuzzle")
						  .version("1.0")
						  .author("Tomas D. <chagle27@gmail.com>")
						  .about("Solves the npuzzle")
						  .arg(Arg::with_name("greedy")
							   .short("g")
							   .long("greedy")
							   .help("Sets search to greedy"))
						  .arg(Arg::with_name("INPUT")
							   .help("Sets the input file to use")
							   .required(true)
							   .conflicts_with("auto")
							   .index(1))
						  .arg(Arg::with_name("heuristic")
							   .short("h")
							   .long("heuristic")
							   .help("Sets the heuristic")
							   .takes_value(true)
							   .possible_values(&["manhattan", "hamming", "ool", "nilsson"]))
						  .arg(Arg::with_name("auto")
						  	   .short("a")
							   .long("auto")
							   .help("Auto-generates board")
							   .takes_value(true))
						  .arg(Arg::with_name("mine")
							   .short("m")
							   .long("mine")
							   .help("Lets you take the wheel"))
						  .arg(Arg::with_name("quiet")
							   .short("q")
							   .long("quiet")
							   .conflicts_with("mine")
							   .help("Suppresses visualizer"))
						  .get_matches();
	let puzzle = if matches.is_present("auto") {
		puzzle_gen(matches.value_of("auto").unwrap().parse::<usize>().unwrap_or(3))
	}
	else {
		let mut f = File::open(matches.value_of("INPUT").unwrap()).expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		parse_input(contents).expect("invalid puzzle")
	};
	let greedy = matches.is_present("greedy");
	let heur = match matches.value_of("heuristic").unwrap_or("manhattan") {
		"manhattan" => Heuristic::Manhattan,
		"hamming" => Heuristic::Hamming,
		"ool" => Heuristic::OutOfLine,
		"nilsson" => Heuristic::Nilsson,
		_ => Heuristic::Manhattan,
	};
	let quest = refine(puzzle.clone(), heur, greedy);
	for row in puzzle.iter() {
		println!("{:?}", row);
	}
	if quest.insoluble() {
		println!("Unstackable cups!");
	}
	else {
		let width = 500;
		let opengl = OpenGL::V3_2;
		let mut window: Window = WindowSettings::new(
				"white-square",
				[width, width]
			)
			.opengl(opengl)
			.exit_on_esc(true)
			.build()
			.unwrap();
		if matches.is_present("mine") {
			let game = Game::new(GlGraphics::new(opengl), puzzle, quest.get_goal(), width);
			let_me_try(game, window);
		}
		else {
			let goal = quest.get_goal();
			let steps = solverize(quest);
			if matches.is_present("quiet") {
				for i in 0..(steps.len()) {
					if i != 0 {
						match (steps[i].0 as i64 - steps[i - 1].0 as i64, steps[i].1 as i64 - steps[i - 1].1 as i64) {
							(-1, 0) => println!("Slide down!"),
							(1, 0) => println!("Slide up!"),
							(0, -1) => println!("Slide right!"),
							(0, 1) => println!("Slide left!"),
							_ => println!("A hop, skip, or jump"),
						}
					}
				}
			}
			else {
				let viz = Viz::new(GlGraphics::new(opengl), steps, puzzle, &goal, width);
				do_it(viz, window);
			}
		}
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

	#[test]
	fn accept_comments() {
		let mut f = File::open("puzzles/parsing/comments.txt").expect("could not open file");
		let mut contents = String::new();
		f.read_to_string(&mut contents).expect("could not read file");
		let puzzle = parse_input(contents).expect("Error");
		assert!(puzzle.len() == 3);
		for i in 0..3 {
			assert!(puzzle[i].len() == 3);
		}
	}
}
