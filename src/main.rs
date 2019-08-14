extern crate rand;

use ggez::event::KeyMods;
use ggez::event::KeyCode;
use ggez::*;

use std::{path, env};

use rand::Rng;

//Constants

const WINDOW_SIZE: (f32, f32) = (CELL_SIZE*29.0, CELL_SIZE*19.0);

const YELLOW: (f32, f32, f32) = (1.0, 1.0, 0.0);
const BLUE: (f32, f32, f32) = (0.0, 0.0, 1.0);
const RED: (f32, f32, f32) = (1.0, 0.0, 0.0);
const GREEN: (f32, f32, f32) = (0.0, 1.0, 0.0);

const EDGE_X1: f32 = 0.0;
const EDGE_X2: f32 = CELL_SIZE*10.0;

const CELL_SIZE: f32 = 32.0;

//Methods/Structs

#[derive(Clone,Copy,PartialEq,Debug)]
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
    Z1,
    Z2,
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
        hitbox: ggez::graphics::Rect::new(position.0 + off.x - (CELL_SIZE/2.0), position.1 + off.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE),
        parent: parent,
        offset: off,
        color: ggez::graphics::Color::new(set_color.0, set_color.1, set_color.2, 1.0)}
    }

    fn update(&mut self, pos: V2) -> GameResult {
        self.pos = pos + self.offset;

        self.hitbox = ggez::graphics::Rect::new(self.pos.x - (CELL_SIZE/2.0), self.pos.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE);

        Ok(())
    }

    fn update_hb(&mut self) -> GameResult {
        self.hitbox = ggez::graphics::Rect::new(self.pos.x - (CELL_SIZE/2.0), self.pos.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE);

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
            //2.0 is the visual margin
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
            KeyCode::Up => Some(PieceMove::Up),
            KeyCode::Right => Some(PieceMove::Right),
            KeyCode::Down => Some(PieceMove::Down),
            KeyCode::Left => Some(PieceMove::Left),
            _ => None,
        }
    }
}

#[derive(Clone,PartialEq)]
struct Piece {
    pos: V2,
    cells: Vec<Cell>,
    piece: ParentType,
}

impl Piece {
    fn new(piece_type: ParentType, color: (f32, f32, f32)) -> Self {
        match piece_type {
            ParentType::Block => Piece { pos: V2 {x: CELL_SIZE, y: CELL_SIZE}, cells: vec![Cell::new((CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color),Cell::new((CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color),Cell::new((CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: (CELL_SIZE/2.0) }, color),Cell::new((CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: (CELL_SIZE/2.0) },color)], piece: piece_type },
            ParentType::Line => Piece { pos: V2 {x: (CELL_SIZE*2.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((0.0, 0.0), ParentType::Line, V2 {x: -(CELL_SIZE*2.0), y: 0.0 }, color),Cell::new((CELL_SIZE, 0.0), ParentType::Line, V2 {x: -CELL_SIZE, y: 0.0 },color),Cell::new((CELL_SIZE*2.0, 0.0), ParentType::Line, V2 {x: 0.0, y: 0.0 }, color),Cell::new((CELL_SIZE*3.0, 0.0), ParentType::Line, V2 {x: CELL_SIZE, y: 0.0 }, color)], piece: piece_type },
            ParentType::L1 => Piece { pos: V2 {x: (CELL_SIZE*1.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((0.0, 0.0), ParentType::L1, V2 {x: -CELL_SIZE, y: 0.0 }, color),Cell::new((CELL_SIZE, 0.0), ParentType::L1, V2 {x: -0.0, y: 0.0 },color),Cell::new((CELL_SIZE, CELL_SIZE), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((CELL_SIZE, CELL_SIZE*2.0), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color)], piece: piece_type },
            ParentType::L2 => Piece { pos: V2 {x: (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((0.0, 0.0), ParentType::L2, V2 {x: CELL_SIZE, y: 0.0 }, color),Cell::new((CELL_SIZE, 0.0), ParentType::L2, V2 {x: -0.0, y: 0.0 },color),Cell::new((CELL_SIZE, CELL_SIZE), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((CELL_SIZE, CELL_SIZE*2.0), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color)], piece: piece_type },
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

    fn rotate(&mut self, dir: PieceMove) {
        if self.piece == ParentType::Block {
            return;
        }

        match dir {
            PieceMove::Right | PieceMove::Left => {
                for c in &mut self.cells {
                    let og_offset = c.offset;

                    if og_offset.x * og_offset.y > 0.0 {
                        c.offset.x *= -1.0;
                    } else if og_offset.x * og_offset.y < 0.0 {
                        c.offset.y *= -1.0;
                    } else if og_offset.x == 0.0 || og_offset.y == 0.0 {
                        c.offset.x = og_offset.y;
                        c.offset.y = og_offset.x;
                    }
                }
            },
            _ => (),
        }
    }

    fn update(&mut self) -> GameResult {
        for c in &mut self.cells {
            c.update(self.pos).expect("Failed to update cell from piece");
        }

        Ok(())
    }

    fn update_hb(&mut self) -> GameResult {
        for c in &mut self.cells {
            c.update_hb().expect("Failed to update cell hitboxes from piece");
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
                self.pos.y += CELL_SIZE;

                for c in &mut self.cells {
                    c.pos.y += CELL_SIZE;
                }
            },
            PieceMove::Up => (),
            PieceMove::Left => {
                self.pos.x -= CELL_SIZE;

                for c in &mut self.cells {
                    c.pos.x -= CELL_SIZE;
                }
            },
            PieceMove::Right => {
                self.pos.x += CELL_SIZE;

                for c in &mut self.cells {
                    c.pos.x += CELL_SIZE;
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
                match self.blocks.iter().find(|&x| x.pos.x == c.pos.x + CELL_SIZE && x.pos.y == c.pos.y) {
                    Some(_o) => {
                        canmove = false;
                        break;},
                    None => canmove = true,
                }

                if c.hitbox.right() + CELL_SIZE > EDGE_X2 {
                    canmove = false;
                    break;
                }
            }

            canmove
        } else if p_move == PieceMove::Left {
            let mut canmove = false;
            for c in &mut self.falling_piece.cells {
                match self.blocks.iter().find(|&x| x.pos.x == c.pos.x - CELL_SIZE && x.pos.y == c.pos.y) {
                    Some(_o) => {
                        canmove = false;
                        break;},
                    None => canmove = true,
                }

                if c.hitbox.left() - CELL_SIZE < 0.0 {
                    canmove = false;
                    break;
                }
            }

            canmove
        } else if p_move == PieceMove::Down {
            if self.falling_piece.cells.iter().find(|&x| x.pos.y + (CELL_SIZE/2.0) >= WINDOW_SIZE.1) != None {
                return false;
            }

            let mut canmove = false;
            for c in &mut self.falling_piece.cells {
                match self.blocks.iter().find(|&x| x.pos.y == c.pos.y + CELL_SIZE && x.pos.x == c.pos.x) {
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

    fn can_rotate(&mut self, rotation: PieceMove) -> bool {
        let mut rotate = self.falling_piece.clone();
        rotate.rotate(rotation);
        rotate.update().expect("Failed to update hitboxes");

        if rotate.cells.iter().find(|&x| x.pos.x < EDGE_X1 || x.pos.x + (CELL_SIZE/2.0) > EDGE_X2) != None {
            return false;
        }

        let mut can = false;

        for b in &mut rotate.cells {
            match self.blocks.iter().find(|&x| x.pos == b.pos) {
                Some(_c) => {can = false; break;},
                None => can = true,
            };
        }

        can
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
                self.falling_piece = gen_piece();
            }

            self.last_update = std::time::Instant::now();
        }

        match self.falling_piece.update() {
            Ok(_n) => (),
            Err(e) => panic!("Error: {:?}", e)
        }

        for c in &mut self.blocks {
            c.update_hb().expect("Failed to update cells");
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
        if keycode == KeyCode::Z && self.can_rotate(PieceMove::Right) {
            self.falling_piece.rotate(PieceMove::Right);
        }

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
        0 => YELLOW,
        1 => BLUE,
        2 => RED,
        3 => GREEN,
        _ => (0.0, 0.0, 0.0),
    }
}

fn gen_piece() -> Piece {
    let color = gen_color();

    let rng = rand::thread_rng().gen_range(0, 4);

    let piecetype = match rng {
        0 => ParentType::Block,
        1 => ParentType::L1,
        2 => ParentType::L2,
        3 => ParentType::Line,
        _ => ParentType::Block,
    };

    Piece::new(piecetype, color)
}

fn main() -> GameResult {
    let num = rand::thread_rng().gen_range(0, std::u32::MAX);

    if num == 0 {
        panic!("This is literally a 1 in 4294967296 chance.");
    }

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
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()
        .unwrap();

    let mut state = State { blocks: vec![], falling_piece: gen_piece(), last_update: std::time::Instant::now()};

    event::run(ctx, event_loop, &mut state)
}
