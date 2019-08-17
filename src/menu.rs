use crate::WINDOW_SIZE;
use ggez::*;
use super::V2;

pub struct TextOption {
    pos: V2,
    selected: bool,
    text: String,
    font: graphics::Font,
}

impl TextOption {
    pub fn new(pos_s: V2, text_s: &str, font_s: graphics::Font) -> Self {
        TextOption { pos: pos_s, selected: false, text: String::from(text_s), font: font_s }
    }

    pub fn get_text(&mut self) -> String {
        self.text.clone()
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn unselect(&mut self) {
        self.selected = false;
    }

    pub fn is_selected(&mut self) -> bool {
        self.selected.clone()
    }

    pub fn update(&mut self) -> GameResult {


        Ok(())
    }

    pub fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut color = graphics::Color::new(0.4, 0.4, 0.5, 1.0);

        if self.selected {
            color = graphics::Color::new(1.0, 1.0, 1.0, 1.0);
        }

        let mut d_text = graphics::Text::new(self.text.clone());
        d_text.set_bounds(graphics::mint::Point2 {x: WINDOW_SIZE.0/3.0, y:1000.0}, graphics::Align::Center);
        d_text.set_font(self.font, graphics::Scale::uniform(48.0));

        graphics::draw(ctx, &d_text, graphics::DrawParam::new().dest(graphics::mint::Point2 { x: self.pos.x, y: self.pos.y}).color(color)).expect("Failed to draw text");

        Ok(())
    }
}
