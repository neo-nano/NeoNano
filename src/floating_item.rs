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
            msg: vec![String::from("nop")],
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
        return self.bg_color;
    }
}