use piston::input::*;
use opengl_graphics::{ GlGraphics, Texture };
use std::path::Path;
use piston_window::TextureSettings;

pub struct Viz {
	gl: GlGraphics, // OpenGL drawing backend.
	step: usize, // Current location in sequence 
	steps: Vec<(usize, usize)>, // sequence
	board: Vec<Vec<usize>>, // starting board
	imgref: Vec<(graphics::Image, Texture)>, // images
}

impl Viz {
	pub fn new(gl: GlGraphics, steps: Vec<(usize, usize)>, board: Vec<Vec<usize>>, goal: &Vec<Vec<usize>>, width: u32) -> Viz {
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

	pub fn render(&mut self, args: &RenderArgs) {
		use graphics::*;

		const BLACK: [f32; 4] = [0.0; 4];

		let n = self.board.len() as u32;
		let def_d = DrawState::default();

		self.gl.draw(args.viewport(), |_c, gl| {
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

	pub fn update(&mut self, _args: &UpdateArgs) {
		if self.step < self.steps.len() - 1 {
			self.step += 1;
			let (y1, x1) = self.steps[self.step - 1];
			let (y2, x2) = self.steps[self.step];
			self.board[y1][x1] = self.board[y2][x2];
			self.board[y2][x2] = 0;
		}
	}
}
