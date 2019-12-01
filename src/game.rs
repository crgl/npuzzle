use opengl_graphics::{GlGraphics, Texture};
use piston::input::*;
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
    pub fn new(gl: GlGraphics, board: Vec<Vec<usize>>, goal: Vec<Vec<usize>>, width: u32) -> Game {
        use graphics::*;

        let n = goal.len();
        let width = width as usize;
        let def_t = TextureSettings::new();
        let mut imgref: Vec<(Image, Texture)> = Vec::with_capacity(n * n);
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
                    } else {
                        missing.1 = Texture::from_path(Path::new(&p), &def_t).unwrap();
                        imgref[goal[i][j]].1 =
                            Texture::from_path(Path::new("assets/Solid_black.png"), &def_t)
                                .unwrap();
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
                let (x, y) = ((j * args.width / n) as f64, (i * args.height / n) as f64);
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

    pub fn moveroo(&mut self, args: &Button) {
        use crate::Direction::*;
        if self.complete {
            return;
        }
        let dir = match &args {
            Button::Keyboard(Key::Right) => Left,
            Button::Keyboard(Key::Left) => Right,
            Button::Keyboard(Key::Down) => Up,
            Button::Keyboard(Key::Up) => Down,
            _ => return,
        };
        match (
            dir,
            self.zero.0 > 0,
            self.zero.0 < self.goal.len() - 1,
            self.zero.1 > 0,
            self.zero.1 < self.goal.len() - 1,
        ) {
            (Right, _, _, _, true) => {
                self.board[self.zero.0][self.zero.1] = self.board[self.zero.0][self.zero.1 + 1];
                self.board[self.zero.0][self.zero.1 + 1] = 0;
                self.zero.1 += 1;
            }
            (Left, _, _, true, _) => {
                self.board[self.zero.0][self.zero.1] = self.board[self.zero.0][self.zero.1 - 1];
                self.board[self.zero.0][self.zero.1 - 1] = 0;
                self.zero.1 -= 1;
            }
            (Down, _, true, _, _) => {
                self.board[self.zero.0][self.zero.1] = self.board[self.zero.0 + 1][self.zero.1];
                self.board[self.zero.0 + 1][self.zero.1] = 0;
                self.zero.0 += 1;
            }
            (Up, true, _, _, _) => {
                self.board[self.zero.0][self.zero.1] = self.board[self.zero.0 - 1][self.zero.1];
                self.board[self.zero.0 - 1][self.zero.1] = 0;
                self.zero.0 -= 1;
            }
            _ => {}
        }
        self.complete = self.board == self.goal;
    }
}
