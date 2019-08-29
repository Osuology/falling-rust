extern crate rand;

use ggez::graphics::Drawable;
use ggez::event::KeyMods;
use ggez::event::KeyCode;
use ggez::*;

use std::{path, env};

use rand::Rng;

//Modules

pub mod menu;

use crate::menu::TextOption;

//Constants

const WINDOW_SIZE: (f32, f32) = (CELL_SIZE*29.0, CELL_SIZE*19.0);

const YELLOW: (f32, f32, f32) = (1.0, 1.0, 0.0);
const BLUE: (f32, f32, f32) = (0.0, 0.0, 1.0);
const RED: (f32, f32, f32) = (1.0, 0.0, 0.0);
const GREEN: (f32, f32, f32) = (0.0, 1.0, 0.0);

const EDGE_X1: f32 = CELL_SIZE*9.0;
const EDGE_X2: f32 = EDGE_X1 + CELL_SIZE*10.0;

const CELL_SIZE: f32 = 32.0;

//Methods/Structs

#[derive(Clone,Copy,PartialEq,Debug)]
pub struct V2 {
    pub x: f32,
    pub y: f32,
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
    shrink: bool,
    done: bool,
}

impl Cell {
    fn new(position: (f32, f32), parent: ParentType, off: V2, set_color: (f32, f32, f32)) -> Self {
        Cell { pos: V2 {x: position.0 + off.x, y: position.1 + off.y},
        hitbox: ggez::graphics::Rect::new(position.0 + off.x - (CELL_SIZE/2.0), position.1 + off.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE),
        parent: parent,
        offset: off,
        color: ggez::graphics::Color::new(set_color.0, set_color.1, set_color.2, 1.0),
        shrink: false,
        done: false}
    }

    fn update(&mut self, pos: V2) -> GameResult {
        self.pos = pos + self.offset;

        self.hitbox = ggez::graphics::Rect::new(self.pos.x - (CELL_SIZE/2.0), self.pos.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE);



        Ok(())
    }

    fn update_hb(&mut self) -> GameResult {
        self.hitbox = ggez::graphics::Rect::new(self.pos.x - (CELL_SIZE/2.0), self.pos.y - (CELL_SIZE/2.0), self.hitbox.w, self.hitbox.h);

        if self.shrink && self.hitbox.w > 0.0 {
            self.hitbox.w -= 2.0;
            self.hitbox.h -= 2.0;
            self.pos.x += 1.0;
            self.pos.y += 1.0;
        } else if self.hitbox.w <= 0.0 {
            self.done = true;
        }

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
            ParentType::Block => Piece { pos: V2 {x: EDGE_X1 + CELL_SIZE, y: CELL_SIZE}, cells: vec![Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: (CELL_SIZE/2.0) }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: (CELL_SIZE/2.0) },color)], piece: piece_type },
            ParentType::Line => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE*2.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Line, V2 {x: -(CELL_SIZE*2.0), y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Line, V2 {x: -CELL_SIZE, y: 0.0 },color),Cell::new((EDGE_X1 + CELL_SIZE*2.0, 0.0), ParentType::Line, V2 {x: 0.0, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE*3.0, 0.0), ParentType::Line, V2 {x: CELL_SIZE, y: 0.0 }, color)], piece: piece_type },
            ParentType::L1 => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE*1.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::L1, V2 {x: -CELL_SIZE, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::L1, V2 {x: -0.0, y: 0.0 },color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color)], piece: piece_type },
            ParentType::L2 => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::L2, V2 {x: CELL_SIZE, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::L2, V2 {x: -0.0, y: 0.0 },color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color)], piece: piece_type },
            ParentType::T => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE*1.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::T, V2 {x: 0.0, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::T, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::T, V2 {x: CELL_SIZE, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::T, V2 {x: -CELL_SIZE, y: CELL_SIZE }, color)], piece: piece_type },
            ParentType::Z1 => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE*1.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Z1, V2 {x: 0.0, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Z1, V2 {x: CELL_SIZE, y: 0.0 },color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Z1, V2 {x: CELL_SIZE, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::Z1, V2 {x: CELL_SIZE*2.0, y: CELL_SIZE }, color)], piece: piece_type },
            ParentType::Z2 => Piece { pos: V2 {x: EDGE_X1 + (CELL_SIZE*1.5), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Z2, V2 {x: 0.0, y: CELL_SIZE }, color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Z2, V2 {x: CELL_SIZE, y: CELL_SIZE },color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Z2, V2 {x: CELL_SIZE, y: 0.0 }, color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::Z2, V2 {x: CELL_SIZE*2.0, y: 0.0 }, color)], piece: piece_type },
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

                    c.offset = V2 { x: -og_offset.y, y: og_offset.x };
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

enum GameState {
    MainMenu,
    Game,
    PausedGame,
}

struct State {
    blocks: Vec<Cell>,
    falling_piece: Piece,
    last_update: std::time::Instant,
    tilemap: graphics::Image,
    game_state: GameState,
    options: Vec<TextOption>,
    selected_option: Option<usize>,
    logo: graphics::Image,
    logo_timer: f32,
    logo_direction: bool,
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

                if c.hitbox.left() - CELL_SIZE < EDGE_X1 {
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

    fn main_menu(&mut self) -> GameResult {

        Ok(())
    }

    fn game(&mut self) -> GameResult {
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

        for c in 0..self.blocks.iter().count() {
            if self.blocks[c].pos.x - (CELL_SIZE/2.0) == EDGE_X1 {
                let cell = self.blocks[c];
                let sum: usize = self.blocks.clone().iter().filter(|&x| x.pos.y == cell.pos.y).count();
                println!("{}", format!("{}", sum));
                if sum > 9 {
                    println!("Line detected");
                    for j in &mut self.blocks.iter_mut().filter(|x| x.pos.y == cell.pos.y).collect::<Vec<&mut Cell>>() {
                        j.shrink = true;

                        if j.done == true {

                        }
                    }
                    //TODO: Implement deletion, and movement code
                }
            }
        }

        Ok(())
    }

    fn draw_main_menu(&mut self, ctx: &mut Context) -> GameResult {
        for o in &mut self.options {
            o.draw(ctx).expect("Failed to draw text option");
        }

        let delta = std::time::Instant::now() - self.last_update;

        if self.logo_direction {
            self.logo_timer += delta.as_millis() as f32;
        } else {
            self.logo_timer -= delta.as_millis() as f32;
        }

        if self.logo_timer > 3600.0 {
            self.logo_direction = false;
        } else if self.logo_timer < 0.0 {
            self.logo_direction = true;
        }

        let offset_y = (self.logo_timer - 1800.0) / 120.0;

        self.logo.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 64.0 , y: 64.0 + offset_y}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw logo");

        let mut ggez = graphics::Text::new("This game is made in ggez, a lightweight game framework licensed under MIT. https://ggez.rs");
        ggez.set_bounds(graphics::mint::Point2 {x: 450.0, y:1000.0}, graphics::Align::Center);
        ggez.set_font(graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to get font"), graphics::Scale::uniform(48.0));

        graphics::draw(ctx, &ggez,graphics::DrawParam::new().dest(graphics::mint::Point2 { x: 478.0, y: 350.0}).color(graphics::WHITE)).expect("Failed to draw text");

        self.last_update = std::time::Instant::now();
        Ok(())
    }

    fn draw_game(&mut self, ctx: &mut Context) -> GameResult {
        self.tilemap.draw(ctx, (graphics::mint::Point2 {x: EDGE_X1, y: 0.0},).into())?;

        self.falling_piece.draw(ctx).expect("Failed to draw piece");

        for cell in &mut self.blocks {
            cell.draw(ctx).expect("Failed to draw cell");
        }

        Ok(())
    }

    fn main_menu_keydown(&mut self, keycode: KeyCode, ctx: &mut Context) -> GameResult {
        if keycode == KeyCode::Up {
            match self.selected_option {
                Some(s) => {
                    self.options[s].unselect();

                    if self.selected_option.unwrap() == 0 {
                        self.selected_option = Some(self.options.iter().count() - 1);
                    } else {
                        self.selected_option = Some(self.selected_option.unwrap() - 1);
                    }

                    },
                None => self.selected_option = Some(self.options.iter().count() - 1),
            }

            self.options[self.selected_option.unwrap()].select();
        } else if keycode == KeyCode::Down {
            match self.selected_option {
                Some(s) => {
                    self.options[s].unselect();

                    if self.selected_option.unwrap() == self.options.iter().count() - 1 {
                        self.selected_option = Some(0);
                    } else {
                        self.selected_option = Some(self.selected_option.unwrap() + 1);
                    }
                },
                None => self.selected_option = Some(0),
            }

            self.options[self.selected_option.unwrap()].select();
        }

        if keycode == KeyCode::Return {
            match self.selected_option {
                None => (),
                Some(num) => {
                    if self.options[num].get_text() == "Play Game" {
                        self.game_state = GameState::Game;
                    } else if self.options[num].get_text() == "Exit Game" {
                        ggez::event::quit(ctx);
                    }
                }
            }
        }

        Ok(())
    }

    fn game_keydown(&mut self, keycode: KeyCode) -> GameResult {
        if keycode == KeyCode::Z && self.can_rotate(PieceMove::Right) {
            self.falling_piece.rotate(PieceMove::Right);
        }

        if keycode == KeyCode::Return {
            self.game_state = GameState::PausedGame;
        }

        if let Some(dir) = PieceMove::from_keycode(keycode) {
            if dir != PieceMove::Up && self.can_move(dir) {
                self.falling_piece.move_piece(dir).expect("Failed to move piece");
            }
        }

        Ok(())
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        match self.game_state {
            GameState::MainMenu => self.main_menu(),
            GameState::Game => self.game(),
            GameState::PausedGame => self.main_menu(),
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ggez::graphics::clear(ctx, ggez::graphics::BLACK);

        match self.game_state {
            GameState::MainMenu => self.draw_main_menu(ctx).expect("Failed to draw main menu"),
            GameState::Game => self.draw_game(ctx).expect("Failed to draw game"),
            GameState::PausedGame => {
                self.draw_game(ctx).expect("Failed to draw game");
                self.draw_main_menu(ctx).expect("Failed to draw main menu")
            }
        };

        ggez::graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match self.game_state {
            GameState::MainMenu => self.main_menu_keydown(keycode, ctx).expect("Failed to process key - main menu"),
            GameState::Game => self.game_keydown(keycode).expect("Failed to process key - game"),
            GameState::PausedGame => self.main_menu_keydown(keycode, ctx).expect("Failed to process key - main menu"),
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

    let rng = rand::thread_rng().gen_range(0, 7);

    let piecetype = match rng {
        0 => ParentType::Block,
        1 => ParentType::L1,
        2 => ParentType::L2,
        3 => ParentType::Line,
        4 => ParentType::T,
        5 => ParentType::Z1,
        6 => ParentType::Z2,
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

    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("Falling Rust", "Osuology")
        .add_resource_path(resource_dir)
        .window_setup(ggez::conf::WindowSetup::default().title("Falling Rust"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()
        .unwrap();

    let mut state = State { blocks: vec![], falling_piece: gen_piece(), last_update: std::time::Instant::now(),
        tilemap: ggez::graphics::Image::new(ctx, "/tilemap.png").expect("Failed to load bg image"), game_state: GameState::MainMenu,
        options: vec![TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0}, "Play Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0 + 64.0}, "Controls", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0 + 128.0}, "Exit Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font"))],
        selected_option: None,
        logo: graphics::Image::new(ctx, "/falling_rust_logo.png").expect("Failed to load logo"),
        logo_timer: 0.0,
        logo_direction: true};

    event::run(ctx, event_loop, &mut state)
}
