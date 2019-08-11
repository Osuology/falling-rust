extern crate rand;

use ggez::event::KeyMods;
use ggez::event::KeyCode;
use ggez::*;

use std::{path, env};

use rand::Rng;

const yellow_color: (f32, f32, f32) = (1.0, 1.0, 0.0);
const blue_color: (f32, f32, f32) = (0.0, 0.0, 1.0);
const red_color: (f32, f32, f32) = (1.0, 0.0, 0.0);
const green_color: (f32, f32, f32) = (0.0, 1.0, 0.0);

#[derive(Clone,Copy,PartialEq)]
struct V2 {
    x: f32,
    y: f32,
}

impl std::ops::Sub for V2 {
    type Output = V2;

    fn sub(self, other: V2) -> V2 {
        V2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Add for V2 {
    type Output = V2;

    fn add(self, other: V2) -> V2 {
        V2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone,Copy,PartialEq)]
enum ParentType {
    L1,
    L2,
    T,
    Line,
    Block,
}

#[derive(Copy,Clone,PartialEq)]
struct Cell {
    pos: V2,
    offset: V2,
    hitbox: ggez::graphics::Rect,
    parent: ParentType,
    color: ggez::graphics::Color,
}

impl Cell {
    fn new(position: (f32, f32), parent: ParentType, off: V2, set_color: (f32, f32, f32)) -> Self {
        Cell { pos: V2 {x: position.0 + off.x, y: position.1 + off.y},
        hitbox: ggez::graphics::Rect::new(position.0 + off.x - 16.0, position.1 + off.y - 16.0, 32.0, 32.0),
        parent: parent,
        offset: off,
        color: ggez::graphics::Color::new(set_color.0, set_color.1, set_color.2, 1.0)}
    }

    fn update(&mut self, ctx: &mut Context, pos: V2) -> GameResult {
        self.pos = pos + self.offset;

        self.hitbox = ggez::graphics::Rect::new(self.pos.x - 16.0, self.pos.y - 16.0, 32.0, 32.0);

        Ok(())
    }

    fn update_hb(&mut self, ctx: &mut Context) -> GameResult {
        self.hitbox = ggez::graphics::Rect::new(self.pos.x - 16.0, self.pos.y - 16.0, 32.0, 32.0);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let rec = ggez::graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.hitbox,
            [0.7, 0.7, 0.7, 1.0].into()
            )?;

        graphics::draw(ctx, &rec, (ggez::mint::Point2 {x: 0.0, y: 0.0},))?;

        let rec = ggez::graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::graphics::Rect::new(self.hitbox.x + 2.0, self.hitbox.y + 2.0, self.hitbox.w - 4.0, self.hitbox.h - 4.0),
            self.color.into()
            )?;

        graphics::draw(ctx, &rec, (ggez::mint::Point2 {x: 0.0, y: 0.0},))?;

        Ok(())
    }
}

#[derive(Copy,Clone,PartialEq)]
enum PieceMove {
    Right,
    Left,
    Up,
    Down,
}

impl PieceMove {
    fn from_keycode(key: KeyCode) -> Option<PieceMove> {
        match key {
            KeyCode::W => Some(PieceMove::Up),
            KeyCode::D => Some(PieceMove::Right),
            KeyCode::S => Some(PieceMove::Down),
            KeyCode::A => Some(PieceMove::Left),
            _ => None,
        }
    }
}

struct Piece {
    pos: V2,
    cells: Vec<Cell>,
    piece: ParentType,
}

impl Piece {
    fn new(piece_type: ParentType, color: (f32, f32, f32)) -> Self {
        match piece_type {
            ParentType::Block => Piece { pos: V2 {x: 32.0, y: 32.0}, cells: vec![Cell::new((32.0, 32.0), ParentType::Block, V2 {x: -16.0, y: -16.0 }, color),Cell::new((32.0, 32.0), ParentType::Block, V2 {x: 16.0, y: -16.0 }, color),Cell::new((32.0, 32.0), ParentType::Block, V2 {x: -16.0, y: 16.0 }, color),Cell::new((32.0, 32.0), ParentType::Block, V2 {x: 16.0, y: 16.0 },color)], piece: piece_type },
            ParentType::Line => Piece { pos: V2 {x: 80.0, y: 16.0}, cells: vec![Cell::new((0.0, 0.0), ParentType::Line, V2 {x: -64.0, y: 0.0 }, color),Cell::new((32.0, 0.0), ParentType::Line, V2 {x: -32.0, y: 0.0 },color),Cell::new((64.0, 0.0), ParentType::Line, V2 {x: 0.0, y: 0.0 }, color),Cell::new((96.0, 0.0), ParentType::Line, V2 {x: 32.0, y: 0.0 }, color)], piece: piece_type },
            _ => Piece { pos: V2 {x: 0.0, y: 0.0}, cells: vec![], piece: piece_type },
        }
    }

    fn move_to(&mut self, position: V2) -> GameResult {
        for c in self.cells.iter_mut() {
            c.pos = position + c.offset;
        }

        self.pos = position;

        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        for c in &mut self.cells {
            c.update(ctx, self.pos).expect("Failed to update cell from piece");
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        for c in &mut self.cells {
            c.draw(ctx).expect("Failed to draw cell from piece");
        }

        Ok(())
    }

    fn move_piece(&mut self, p_move: PieceMove) -> GameResult {
        match p_move {
            PieceMove::Down => {
                self.pos.y += 32.0;

                for c in &mut self.cells {
                    c.pos.y += 32.0;
                }
            },
            PieceMove::Up => (),
            PieceMove::Left => {
                self.pos.x -= 32.0;

                for c in &mut self.cells {
                    c.pos.x -= 32.0;
                }
            },
            PieceMove::Right => {
                self.pos.x += 32.0;

                for c in &mut self.cells {
                    c.pos.x += 32.0;
                }
            },
        }

        Ok(())
    }
}

struct State {
    blocks: Vec<Cell>,
    falling_piece: Piece,
    last_update: std::time::Instant,
}

impl State {
    fn can_move(&mut self, p_move: PieceMove) -> bool {
        if p_move == PieceMove::Right {
            let mut canmove = false;
            for c in &mut self.falling_piece.cells {
                match self.blocks.iter().find(|&x| x.pos.x == c.pos.x + 32.0 && x.pos.y == c.pos.y) {
                    Some(_o) => {
                        canmove = false;
                        break;},
                    None => canmove = true,
                }
            }

            canmove
        } else if p_move == PieceMove::Left {
            let mut canmove = false;
            for c in &mut self.falling_piece.cells {
                match self.blocks.iter().find(|&x| x.pos.x == c.pos.x - 32.0 && x.pos.y == c.pos.y) {
                    Some(_o) => {
                        canmove = false;
                        break;},
                    None => canmove = true,
                }
            }

            canmove
        } else if p_move == PieceMove::Down {
            if self.falling_piece.cells.iter().find(|&x| x.pos.y >= 568.0) != None {
                return false;
            }

            let mut canmove = false;
            for c in &mut self.falling_piece.cells {
                match self.blocks.iter().find(|&x| x.pos.y == c.pos.y + 32.0 && x.pos.x == c.pos.x) {
                    Some(_o) => {
                        canmove = false;
                        break;},
                    None => canmove = true,
                }
            }

            canmove
        } else {
            false
        }
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if std::time::Instant::now() - self.last_update >= std::time::Duration::from_millis(1000) {
            if self.can_move(PieceMove::Down) {
                self.falling_piece.move_piece(PieceMove::Down).expect("Failed to use gravity");
            } else if !self.can_move(PieceMove::Down) {
                let copy = &mut self.falling_piece.cells;
                self.blocks.append(copy);
                self.falling_piece = Piece::new(ParentType::Line, gen_color());
            }

            self.last_update = std::time::Instant::now();
        }

        match self.falling_piece.update(ctx) {
            Ok(_n) => (),
            Err(e) => panic!("Error: {:?}", e)
        }

        for c in &mut self.blocks {
            c.update_hb(ctx).expect("Failed to update cells");
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ggez::graphics::clear(ctx, ggez::graphics::BLACK);

        self.falling_piece.draw(ctx).expect("Failed to draw piece");

        for cell in &mut self.blocks {
            cell.draw(ctx).expect("Failed to draw cell");
        }

        ggez::graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if let Some(dir) = PieceMove::from_keycode(keycode) {
            if dir != PieceMove::Up && self.can_move(dir) {
                self.falling_piece.move_piece(dir).expect("Failed to move piece");
            }
        }
    }
}

fn gen_color() -> (f32, f32, f32) {
    let rng = rand::thread_rng().gen_range(0, 4);

    match rng {
        0 => yellow_color,
        1 => blue_color,
        2 => red_color,
        3 => green_color,
        _ => (0.0, 0.0, 0.0),
    }
}

fn main() -> GameResult {

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR")
    {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("Tetris", "Osuology")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Tetris"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(928.0, 608.0))
        .build()
        .unwrap();

    let mut state = State { blocks: vec![], falling_piece: Piece::new(ParentType::Block, gen_color()), last_update: std::time::Instant::now()};

    event::run(ctx, event_loop, &mut state)
}
