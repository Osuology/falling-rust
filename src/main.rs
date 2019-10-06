#![windows_subsystem = "windows"]

extern crate rand;

use ggez::graphics::Drawable;
use ggez::event::KeyMods;
use ggez::event::KeyCode;
use ggez::audio::SoundSource;
use ggez::*;

use std::{path, env};

use rand::Rng;

//Modules

pub mod menu;

use crate::menu::TextOption;

//Constants

const WINDOW_SIZE: (f32, f32) = (CELL_SIZE*29.0, CELL_SIZE*19.0);

/*const YELLOW: (f32, f32, f32) = (157.0/255.0, 97.0/255.0, 175.0/255.0);
const BLUE: (f32, f32, f32) = (66.0/255.0, 84.0/255.0, 193.0/255.0);
const RED: (f32, f32, f32) = (218.0/255.0, 220.0/255.0, 38.0/255.0);
const GREEN: (f32, f32, f32) = (37.0/255.0, 199.0/255.0, 75.0/255.0);*/

const YELLOW: (f32, f32, f32) = (161.0/255.0, 122.0/255.0, 13.0/255.0);
const BLUE: (f32, f32, f32) = (157.0/255.0, 7.0/255.0, 81.0/255.0);
const RED: (f32, f32, f32) = (171.0/255.0, 29.0/255.0, 118.0/255.0);
const GREEN: (f32, f32, f32) = (161.0/255.0, 73.0/255.0, 14.0/255.0);

const YELLOW_B: (f32, f32, f32) = (195.0/255.0, 175.0/255.0, 38.0/255.0);
const BLUE_B: (f32, f32, f32) = (202.0/255.0, 58.0/255.0, 126.0/255.0);
const RED_B: (f32, f32, f32) = (192.0/255.0, 46.0/255.0, 135.0/255.0);
const GREEN_B: (f32, f32, f32) = (188.0/255.0, 123.0/255.0, 41.0/255.0);

const EDGE_X1: f32 = CELL_SIZE*9.0;
const EDGE_X2: f32 = EDGE_X1 + CELL_SIZE*10.0;
const EDGE_X3: f32 = EDGE_X1 + CELL_SIZE*5.0;

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

#[derive(Clone,Copy,PartialEq,Debug)]
enum ParentType {
    L1,
    L2,
    T,
    Z1,
    Z2,
    Line,
    Block,
}

#[derive(Copy,Clone,PartialEq,Debug)]
struct Cell {
    pos: V2,
    offset: V2,
    hitbox: ggez::graphics::Rect,
    parent: ParentType,
    color: ggez::graphics::Color,
    back_color: ggez::graphics::Color,
    shrink: bool,
    done: bool,
}

impl Cell {
    fn new(position: (f32, f32), parent: ParentType, off: V2, set_color: (f32, f32, f32), set_back_color: (f32, f32, f32)) -> Self {
        Cell { pos: V2 {x: position.0 + off.x, y: position.1 + off.y},
        hitbox: ggez::graphics::Rect::new(position.0 + off.x - (CELL_SIZE/2.0), position.1 + off.y - (CELL_SIZE/2.0), CELL_SIZE, CELL_SIZE),
        parent: parent,
        offset: off,
        color: ggez::graphics::Color::new(set_color.0, set_color.1, set_color.2, 1.0),
        back_color: ggez::graphics::Color::new(set_back_color.0, set_back_color.1, set_back_color.2, 1.0),
        shrink: false,
        done: false}
    }

    fn move_rel(&mut self, position: (f32, f32)) {
        self.pos.x += position.0;
        self.pos.y += position.1;
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
            self.back_color.into()
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
    moved_down: bool,
}

impl Piece {
    fn new(piece_type: ParentType, color: (f32, f32, f32), back_color: (f32, f32, f32)) -> Self {
        match piece_type {
            ParentType::Block => Piece { pos: V2 {x: EDGE_X3, y: CELL_SIZE}, cells: vec![Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: -(CELL_SIZE/2.0) }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: -(CELL_SIZE/2.0), y: (CELL_SIZE/2.0) }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Block, V2 {x: (CELL_SIZE/2.0), y: (CELL_SIZE/2.0) },color, back_color)], piece: piece_type, moved_down: false },
            ParentType::Line => Piece { pos: V2 {x: EDGE_X3 + (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Line, V2 {x: -(CELL_SIZE*2.0), y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Line, V2 {x: -CELL_SIZE, y: 0.0 },color, back_color),Cell::new((EDGE_X1 + CELL_SIZE*2.0, 0.0), ParentType::Line, V2 {x: 0.0, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE*3.0, 0.0), ParentType::Line, V2 {x: CELL_SIZE, y: 0.0 }, color, back_color)], piece: piece_type, moved_down: false },
            ParentType::L1 => Piece { pos: V2 {x: EDGE_X3 - (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::L1, V2 {x: -CELL_SIZE, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::L1, V2 {x: -0.0, y: 0.0 },color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::L1, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color, back_color)], piece: piece_type, moved_down: false },
            ParentType::L2 => Piece { pos: V2 {x: EDGE_X3 - (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::L2, V2 {x: CELL_SIZE, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::L2, V2 {x: -0.0, y: 0.0 },color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::L2, V2 {x: 0.0, y: CELL_SIZE*2.0 }, color, back_color)], piece: piece_type, moved_down: false },
            ParentType::T => Piece { pos: V2 {x: EDGE_X3 - (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::T, V2 {x: 0.0, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::T, V2 {x: 0.0, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::T, V2 {x: CELL_SIZE, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::T, V2 {x: -CELL_SIZE, y: CELL_SIZE }, color, back_color)], piece: piece_type, moved_down: false },
            ParentType::Z1 => Piece { pos: V2 {x: EDGE_X3 - (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Z1, V2 {x: 0.0, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Z1, V2 {x: CELL_SIZE, y: 0.0 },color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Z1, V2 {x: CELL_SIZE, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::Z1, V2 {x: CELL_SIZE*2.0, y: CELL_SIZE }, color, back_color)], piece: piece_type, moved_down: false },
            ParentType::Z2 => Piece { pos: V2 {x: EDGE_X3 - (CELL_SIZE/2.0), y: (CELL_SIZE/2.0)}, cells: vec![Cell::new((EDGE_X1, 0.0), ParentType::Z2, V2 {x: 0.0, y: CELL_SIZE }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, 0.0), ParentType::Z2, V2 {x: CELL_SIZE, y: CELL_SIZE },color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE), ParentType::Z2, V2 {x: CELL_SIZE, y: 0.0 }, color, back_color),Cell::new((EDGE_X1 + CELL_SIZE, CELL_SIZE*2.0), ParentType::Z2, V2 {x: CELL_SIZE*2.0, y: 0.0 }, color, back_color)], piece: piece_type, moved_down: false },
        }
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

                self.moved_down = true;
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

#[derive(PartialEq,Copy,Clone,Debug)]
enum GameState {
    MainMenu,
    Game,
    PausedGame,
    Controls,
    GameOver,
}

struct State {
    blocks: Vec<Cell>,
    falling_piece: Piece,
    last_update: std::time::Instant,
    fps_update: std::time::Instant,
    tilemap: graphics::Image,
    game_state: GameState,
    previous_game_state: GameState,
    options: Vec<TextOption>,
    selected_option: Option<usize>,
    bg: graphics::Image,
    logo: graphics::Image,
    logo_timer: f32,
    logo_direction: bool,
    pause_options: Vec<TextOption>,
    pause_select: Option<usize>,
    rate: u32,
    move_sound: ggez::audio::Source,
    rotate_sound: ggez::audio::Source,
    land_sound: ggez::audio::Source,
    line_sound: ggez::audio::Source,
    music: ggez::audio::Source,
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

        Ok(())
    }

    fn game(&mut self) -> GameResult {
        if std::time::Instant::now() - self.last_update >= std::time::Duration::from_millis(self.rate.into()) {
            self.rate -= 3;
            if (self.rate < 250) {
                self.rate = 250;
            }

            if self.can_move(PieceMove::Down) {
                self.falling_piece.move_piece(PieceMove::Down).expect("Failed to use gravity");
            } else if !self.can_move(PieceMove::Down) {
                if self.falling_piece.moved_down == false {
                    self.game_state = GameState::GameOver;
                } else {
                    let copy = &mut self.falling_piece.cells;
                    self.blocks.append(copy);
                    self.falling_piece = gen_piece();

                    if self.land_sound.stopped() {
                        self.land_sound.set_volume(0.06);
                        self.land_sound.play();
                    }
                }
            }

            self.last_update = std::time::Instant::now();
        }

        match self.falling_piece.update() {
            Ok(_n) => (),
            Err(e) => panic!("Error: {:?}", e)
        }

        for c in (0..self.blocks.iter().count()).rev() {
            let mut blocks = self.blocks.clone();
            blocks[c].update_hb().expect("Failed to update cells");

            let index: usize = match blocks.iter().position(|x| *x == blocks[c]) {
                Some(num) => num,
                None => 0,
            };

            let cell = blocks[c];
            if blocks[c].done {
                if (self.line_sound.stopped()){
                    self.line_sound.set_volume(0.06);
                    self.line_sound.play();
                }
                
                for a in blocks.iter_mut().filter(|x| x.pos.x == cell.pos.x - 16.0 && x.pos.y < cell.pos.y).collect::<Vec<&mut Cell>>() {
                    a.move_rel((0.0, CELL_SIZE));
                }

                blocks.remove(index);
            }
            self.blocks = blocks;
        }

        for c in 0..self.blocks.iter().count() {
            if self.blocks[c].pos.x - (CELL_SIZE/2.0) == EDGE_X1 {
                let cell = self.blocks[c];
                let sum: usize = self.blocks.clone().iter().filter(|&x| x.pos.y == cell.pos.y).count();
                if sum > 9 {
                    for j in &mut self.blocks.iter_mut().filter(|x| x.pos.y == cell.pos.y).collect::<Vec<&mut Cell>>() {
                        j.shrink = true;
                    }
                }
            }
        }

        Ok(())
    }

    fn paused(&mut self) -> GameResult {


        Ok(())
    }

    fn game_over(&mut self) -> GameResult {

        Ok(())
    }

    fn draw_main_menu(&mut self, ctx: &mut Context) -> GameResult {
        let offset_y = (self.logo_timer - 1800.0) / 120.0;

        self.bg.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 0.0, y: 0.0}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw background");
        self.logo.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 64.0 , y: 64.0 + offset_y}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw logo");

        let mut ggez = graphics::Text::new("This game is made in ggez, a lightweight game framework licensed under MIT. https://ggez.rs");
        ggez.set_bounds(graphics::mint::Point2 {x: 450.0, y:1000.0}, graphics::Align::Center);
        ggez.set_font(graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to get font"), graphics::Scale::uniform(48.0));

        graphics::draw(ctx, &ggez,graphics::DrawParam::new().dest(graphics::mint::Point2 { x: 478.0, y: 350.0}).color(graphics::WHITE)).expect("Failed to draw text");

        for o in &mut self.options {
            o.draw(ctx).expect("Failed to draw text option");
        }

        self.last_update = std::time::Instant::now();
        Ok(())
    }

    fn draw_game(&mut self, ctx: &mut Context) -> GameResult {
        self.bg.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 0.0, y: 0.0}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw background");

        self.tilemap.draw(ctx, (graphics::mint::Point2 {x: EDGE_X1, y: 0.0},).into())?;

        self.falling_piece.draw(ctx).expect("Failed to draw piece");

        for cell in &mut self.blocks {
            cell.draw(ctx).expect("Failed to draw cell");
        }

        Ok(())
    }

    fn draw_controls(&mut self, ctx: &mut Context) -> GameResult {
        self.bg.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 0.0, y: 0.0}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw background");

        let mut controls = graphics::Text::new("Press Z to rotate falling piece.\nPress Left/Right/Down to move falling piece.\nPress Escape to go back.");
        controls.set_bounds(graphics::mint::Point2 {x: 940.0, y:1000.0}, graphics::Align::Center);
        controls.set_font(graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to get font"), graphics::Scale::uniform(48.0));

        graphics::draw(ctx, &controls,graphics::DrawParam::new().dest(graphics::mint::Point2 { x: 0.0, y: 213.0}).color(graphics::WHITE)).expect("Failed to draw text");

        Ok(())
    }

    fn draw_pause(&mut self, ctx: &mut Context) -> GameResult {
        for o in &mut self.pause_options {
            o.draw(ctx).expect("Failed to draw pause options");
        }

        Ok(())
    }

    fn draw_game_over(&mut self, ctx: &mut Context) -> GameResult {
        self.bg.draw(ctx, graphics::DrawParam::new().dest(mint::Point2 {x: 0.0, y: 0.0}).scale(graphics::mint::Point2 {x: 1.0, y: 1.0})).expect("Failed to draw background");

        let mut gameover = graphics::Text::new("Game Over!\nPress R to retry\nPress Escape to go to main menu");
        gameover.set_bounds(graphics::mint::Point2 {x: 940.0, y:1000.0}, graphics::Align::Center);
        gameover.set_font(graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to get font"), graphics::Scale::uniform(88.0));

        graphics::draw(ctx, &gameover,graphics::DrawParam::new().dest(graphics::mint::Point2 { x: 0.0, y: 163.0}).color(graphics::WHITE)).expect("Failed to draw text");

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
                    } else if self.options[num].get_text() == "Controls" {
                        self.game_state = GameState::Controls;
                    }
                }
            }
        }

        Ok(())
    }

    fn game_keydown(&mut self, keycode: KeyCode) -> GameResult {
        if keycode == KeyCode::Z && self.can_rotate(PieceMove::Right) {
            self.falling_piece.rotate(PieceMove::Right);
                if (self.rotate_sound.stopped()){
                    self.rotate_sound.set_volume(0.06);
                    self.rotate_sound.play();
                }
        }

        if keycode == KeyCode::Escape {
            self.game_state = GameState::PausedGame;
        }

        if let Some(dir) = PieceMove::from_keycode(keycode) {
            if dir != PieceMove::Up && self.can_move(dir) {
                self.falling_piece.move_piece(dir).expect("Failed to move piece");
                if (self.move_sound.stopped()){
                    self.move_sound.set_volume(0.06);
                    self.move_sound.play();
                }
            }
        }

        Ok(())
    }

    fn control_keydown(&mut self, keycode: KeyCode) -> GameResult {
        if keycode == KeyCode::Escape {
            self.game_state = GameState::MainMenu;
        }

        Ok(())
    }

    fn paused_keydown(&mut self, keycode: KeyCode) -> GameResult {
        if keycode == KeyCode::Up {
            match self.pause_select {
                Some(s) => {
                    self.pause_options[s].unselect();

                    if self.pause_select.unwrap() == 0 {
                        self.pause_select = Some(self.pause_options.iter().count() - 1);
                    } else {
                        self.pause_select = Some(self.pause_select.unwrap() - 1);
                    }

                    },
                None => self.pause_select = Some(self.pause_options.iter().count() - 1),
            }

            self.pause_options[self.pause_select.unwrap()].select();
        } else if keycode == KeyCode::Down {
            match self.pause_select {
                Some(s) => {
                    self.pause_options[s].unselect();

                    if self.pause_select.unwrap() == self.pause_options.iter().count() - 1 {
                        self.pause_select = Some(0);
                    } else {
                        self.pause_select = Some(self.pause_select.unwrap() + 1);
                    }
                },
                None => self.pause_select = Some(0),
            }

            self.pause_options[self.pause_select.unwrap()].select();
        }

        if keycode == KeyCode::Return {

            match self.pause_select {
                None => (),
                Some(num) => {
                    if self.pause_options[num].get_text() == "Resume Game" {
                        self.game_state = GameState::Game;
                        self.previous_game_state = GameState::Game;
                    } else if self.pause_options[num].get_text() == "Restart Game" {
                        self.game_state = GameState::Game;
                    }
                    else if self.pause_options[num].get_text() == "Exit to Main Menu" {
                        self.game_state = GameState::MainMenu;
                    }
                }
            }
        } else if keycode == KeyCode::Escape {
            self.game_state = GameState::Game;
            self.previous_game_state = GameState::Game;
        }

        Ok(())
    }

    fn game_over_keydown(&mut self, keycode: KeyCode) -> GameResult {
        if keycode == KeyCode::Escape {
            self.game_state = GameState::MainMenu;
        } else if keycode == KeyCode::R {
            self.game_state = GameState::Game;
        }

        Ok(())
    }
}

impl ggez::event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if std::time::Instant::now() - self.fps_update >= std::time::Duration::from_millis(17) {
            match self.game_state {
                GameState::MainMenu => {
                    if self.game_state != self.previous_game_state {
                        self.logo_timer = 0.0;
                        if self.selected_option != None {
                            self.options[self.selected_option.unwrap()].unselect();
                        }
                        self.selected_option = None;

                        if self.previous_game_state == GameState::Game {
                            self.music.stop();
                        }
                    }

                    self.main_menu()
                },
                GameState::Game => {
                    if self.game_state != self.previous_game_state {
                        self.falling_piece = gen_piece();
                        self.blocks = Vec::new();
                        self.rate = 1000;
                        self.music.set_repeat(true);
                        self.music.set_volume(0.02);
                        self.music.play();
                    }

                    if self.music.paused() {
                        self.music.resume();
                    }

                    self.game()
                },
                GameState::PausedGame => {
                    if self.game_state != self.previous_game_state {
                        if self.pause_select != None {
                            self.pause_options[self.pause_select.unwrap()].unselect();
                        }
                        self.pause_select = None;

                        if self.previous_game_state == GameState::Game {
                            self.music.pause();
                        }
                    }

                    self.paused()
                },
                GameState::Controls => {
                    if self.previous_game_state == GameState::Game {
                        self.music.stop();
                    }

                    Ok(())},
                GameState::GameOver => {
                    self.music.stop();

                    self.game_over()
                    },
            }.expect("Failed to update");

            self.previous_game_state = self.game_state;
            self.fps_update = std::time::Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        ggez::graphics::clear(ctx, ggez::graphics::BLACK);

        match self.game_state {
            GameState::MainMenu => self.draw_main_menu(ctx).expect("Failed to draw main menu"),
            GameState::Game => self.draw_game(ctx).expect("Failed to draw game"),
            GameState::PausedGame => {
                self.draw_game(ctx).expect("Failed to draw game");
                self.draw_pause(ctx).expect("Failed to draw pause menu");
            },
            GameState::Controls => self.draw_controls(ctx).expect("Failed to draw controls"),
            GameState::GameOver => self.draw_game_over(ctx).expect("Failed to draw game over"),
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
            GameState::PausedGame => self.paused_keydown(keycode).expect("Failed to process key - pause menu"),
            GameState::Controls => self.control_keydown(keycode).expect("Failed to process key - controls"),
            GameState::GameOver => self.game_over_keydown(keycode).expect("Failed to process key - game_over"),
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

fn gen_backcolor(matchcolor: (f32, f32, f32)) -> (f32, f32, f32) {
    if matchcolor == YELLOW {
        YELLOW_B
    } else if matchcolor == BLUE {
        BLUE_B
    } else if matchcolor == RED {
        RED_B
    } else if matchcolor == GREEN {
        GREEN_B
    } else {
        (0.0, 0.0, 0.0)
    }
}

fn gen_piece() -> Piece {
    let color = gen_color();
    let backcolor = gen_backcolor(color);

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

    Piece::new(piecetype, color, backcolor)
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
        .window_setup(ggez::conf::WindowSetup::default().title("Falling Rust").icon("/icon.ico"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(WINDOW_SIZE.0, WINDOW_SIZE.1))
        .build()
        .unwrap();

    graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);

    let mut state = State { blocks: vec![], falling_piece: gen_piece(), last_update: std::time::Instant::now(),
        fps_update: std::time::Instant::now(),
        tilemap: ggez::graphics::Image::new(ctx, "/tilemap.png").expect("Failed to load bg image"),
        previous_game_state: GameState::MainMenu,
        game_state: GameState::MainMenu,
        options: vec![TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0}, "Play Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0 + 64.0}, "Controls", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: 0.0, y: WINDOW_SIZE.1 / 2.0 + 128.0}, "Exit Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font"))],
        selected_option: None,
        bg: graphics::Image::new(ctx, "/bg_2.png").expect("Failed to load /bg_2.png"),
        logo: graphics::Image::new(ctx, "/falling_rust_logo.png").expect("Failed to load logo"),
        logo_timer: 0.0,
        logo_direction: true,
        pause_options: vec![TextOption::new(V2 {x: -8.0, y: WINDOW_SIZE.1 / 2.0}, "Resume Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: -8.0, y: WINDOW_SIZE.1 / 2.0 + 64.0}, "Restart Game", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font")),
        TextOption::new(V2 {x: -8.0, y: WINDOW_SIZE.1 / 2.0 + 128.0}, "Exit to Main Menu", graphics::Font::new(ctx, "/VLOBJ_bold.ttf").expect("Failed to load font"))],
        pause_select: None,
        rate: 1000,
        move_sound: ggez::audio::Source::new(ctx, "/move.wav").expect("Failed to load sound effect"),
        rotate_sound: ggez::audio::Source::new(ctx, "/rotate.wav").expect("Failed to load sound effect"),
        land_sound: ggez::audio::Source::new(ctx, "/land.wav").expect("Failed to load sound effect"),
        line_sound: ggez::audio::Source::new(ctx, "/line.wav").expect("Failed to load sound effect"),
        music: ggez::audio::Source::new(ctx, "/music.ogg").expect("Failed to load music")};

    ggez::input::mouse::set_cursor_hidden(ctx, true);

    event::run(ctx, event_loop, &mut state)
}
