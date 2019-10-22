use std::io::prelude::*;
use std::fs::File;
use std::collections::HashSet;

use rand::{thread_rng, Rng};

use clap::{Arg, App};

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

mod node;
mod quest;
mod game;
mod viz;

use crate::quest::Quest;
use crate::game::Game;
use crate::viz::Viz;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Heuristic
{
	Hamming,
	Manhattan,
	OutOfLine,
	Nilsson,
	Custom,
}

#[derive(Copy, Clone)]
pub(crate) enum Direction
{
	Up,
	Down,
	Left,
	Right,
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

fn construct_basic_goal(n: usize) -> Vec<Vec<usize>> {
	let mut goal = Vec::new();
	for _ in 0..n {
		let mut row = Vec::new();
		for _ in 0..n {
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
	goal
}

fn refine(puzzle: Vec<Vec<usize>>, heur: Heuristic, greedy: bool) -> Quest {
	let n = puzzle.len();
	let goal = construct_basic_goal(n);
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

fn insoluble(board: &Vec<Vec<usize>>, goal: Option<Vec<Vec<usize>>>) -> bool {
	let len = board.len();
	let n = len * len;
	let mut inv_board = 0;
	let mut inv_goal = 0;
	let mut zero_row_board = 0;
	let mut zero_row_goal = 0;
	let mut weights_board : Vec<usize> = std::iter::repeat(0).take(n).collect();
	let mut weights_goal = weights_board.clone();
	let goal = if let Some(goal) = goal {
		goal
	} else {
		construct_basic_goal(len)
	};
	for i in 0..len {
		for j in 0..len {
			inv_board += weights_board[board[i][j]];
			inv_goal += weights_goal[goal[i][j]];
			if board[i][j] == 0 {
				zero_row_board = i;
			}
			if goal[i][j] == 0 {
				zero_row_goal = i;
			}
			for k in 1..board[i][j] {
				weights_board[k] += 1;
			}
			for k in 1..goal[i][j] {
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

fn puzzle_gen(len: usize) -> Vec<Vec<usize>> {
	let mut arr = Vec::with_capacity(len * len);
	for x in 0..(len*len) {
		arr.push(x);
	}
	thread_rng().shuffle(&mut arr);
	let mut out = Vec::with_capacity(len);
	for _ in 0..len {
		let mut row = Vec::with_capacity(len);
		for _ in 0..len {
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
	if insoluble(&out, None) {
		puzzle_gen(len)
	} else {
		out
	}
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
							   .possible_values(&["manhattan", "hamming", "ool", "nilsson", "custom"]))
						  .arg(Arg::with_name("auto")
						  	   .short("a")
							   .long("auto")
							   .help("Auto-generates board")
							   .takes_value(true)
							   .possible_values(&["2", "3", "4", "5", "6", "7"]))
						  .arg(Arg::with_name("mine")
							   .short("m")
							   .long("mine")
							   .conflicts_with_all(&["quiet", "heuristic", "greedy"])
							   .help("Lets you take the wheel"))
						  .arg(Arg::with_name("quiet")
							   .short("q")
							   .long("quiet")
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
		"custom" => Heuristic::Custom,
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
		let window: Window = WindowSettings::new(
				"NPuzzle",
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
