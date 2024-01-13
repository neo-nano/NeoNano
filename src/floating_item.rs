use termion::color;
use unicode_segmentation::UnicodeSegmentation;

use crate::terminal::Size;
use crate::Position;

#[derive(Default)]
pub struct FloatingItem {
    pos: Position,
    width: usize,
    height: usize,
    msg: Vec<String>,
    bg_color: (u8, u8, u8),
}

impl FloatingItem {
    pub fn create(pos: Position, width: usize, height: usize) -> Self {
        Self {
            pos,
            width,
            height,
            msg: vec![String::from("nop"), String::from("nah")],
            bg_color: (0, 0, 0),
        }
    }

    pub fn get_pos(&self) -> &Position {
        &self.pos
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_str(&self, index: usize) -> Option<&String> {
        self.msg.get(index)
    }

    pub fn get_bg(&self) -> (u8, u8, u8) {
        self.bg_color
    }

    // TODO Direction
    pub fn render(&self, plain_row: &Vec<String>, drawing_y: usize) -> Vec<String> {
        if drawing_y < self.pos.y || self.pos.y.saturating_add(self.height) <= drawing_y {
            return plain_row.clone();
        }

        let x = self.pos.x;
        let y = self.pos.y;
        let (r, g, b) = self.bg_color;
        let mut result: Vec<String> = plain_row.clone();
        let floating_str = match self.msg.get(drawing_y.saturating_sub(y)) {
            Some(s) => String::from(s),
            None => String::new(),
        };

        let mut floating_vec = vec![];
        let floating_str_len = floating_str.graphemes(true).count();
        if self.width <= floating_str_len {
            for (i, v) in floating_str.graphemes(true).enumerate() {
                if i == 0 {
                    floating_vec.push(format!("{}{}", color::Bg(color::Rgb(r, g, b)), v));
                } else if floating_str_len.saturating_sub(1) == i {
                    floating_vec.push(format!("{}{}", v, color::Bg(color::Reset)));
                } else {
                    floating_vec.push(String::from(v));
                }
            }
        } else {
            for (i, v) in floating_str.graphemes(true).enumerate() {
                if i == 0 {
                    floating_vec.push(format!("{}{}", color::Bg(color::Rgb(r, g, b)), v));
                } else {
                    floating_vec.push(String::from(v));
                }
            }
            let padding_size = self.width.saturating_sub(floating_str_len);
            for i in 0..padding_size {
                if i == padding_size.saturating_sub(1) {
                    floating_vec.push(format!(" {}", color::Bg(color::Reset)));
                } else {
                    floating_vec.push(String::from(" "));
                }
            }
        }

        if x >= plain_row.len() {
            let padding_size = x.saturating_sub(plain_row.len().saturating_sub(1));
            for _ in 0..padding_size {
                result.push(String::from(" "));
            }
            for s in floating_vec {
                result.push(s);
            }
        } else {
            result.truncate(0);
            for s in plain_row.iter().take(x) {
                result.push(s.clone());
            }
            for s in &floating_vec {
                result.push(s.clone());
            }
            for s in plain_row.iter().skip(x.saturating_add(self.width)) {
                result.push(s.clone());
            }
        }
        result
    }
}
