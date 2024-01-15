use std::fs;
use std::io::{Error, Write};

use crate::editor::SearchDirection;
use crate::floating_item::FloatingItem;
use crate::highlighting::Highlight;
use crate::Row;
use crate::{FileType, Position};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
    file_type: FileType,
    floatings: Vec<FloatingItem>,
}

impl Document {
    pub fn open(file_name: &str) -> Result<Self, Error> {
        let contents = fs::read_to_string(file_name)?;
        let file_type = FileType::from(file_name).unwrap_or_default();
        let mut rows: Vec<Row> = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        let mut res = Self {
            rows,
            file_name: Some(file_name.to_owned()),
            dirty: false,
            file_type,
            floatings: vec![FloatingItem::create(Position { x: 10, y: 4 }, 7, 2)],
        };
        res.highlight();
        Ok(res)
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn floating_len(&self) -> usize {
        self.floatings.len()
    }

    pub fn floating(&self, index: usize) -> Option<&FloatingItem> {
        self.floatings.get(index)
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);
        self.rows.insert(at.y + 1, new_row);
        self.highlight();
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }

        self.dirty = true;

        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
        self.highlight();
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;

        if at.x == self.rows.get_mut(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
        self.highlight();
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(file_name) = &self.file_name {
            let mut file = fs::File::create(file_name)?;
            self.file_type = FileType::from(file_name).unwrap_or(FileType::default());
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.highlight();
            self.dirty = false;
        }
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = at.clone();
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };

        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(&query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    pub fn file_type(&self) -> String {
        self.file_type.name()
    }

    pub fn highlight(&mut self) {
        let chars: Vec<Vec<u8>> = self
            .rows
            .iter()
            .map(|r| {
                let mut res = r.as_bytes().to_vec();
                res.push(b'\r');
                res.push(b'\n');
                res
            })
            .collect();
        let chars = chars.into_iter().flatten().collect::<Vec<u8>>();
        let chars: &[u8] = chars.as_slice();

        let hl_opt = self.file_type.highlighting_options();
        if !hl_opt.get_hl_query().is_some() || !hl_opt.get_inj_query().is_some() {
            return;
        }
        let highlighter = Highlight::new(
            hl_opt.get_lang().unwrap(),
            hl_opt.get_hl_query().unwrap(),
            hl_opt.get_inj_query().unwrap(),
        );

        if let Ok(mut highlighter) = highlighter {
            if let Ok(highlight_vec) = highlighter.highlight(chars) {
                let mut highlight_idx: usize = 0;
                for row in &mut self.rows {
                    let row_len = row.as_bytes().len();
                    if let Some(new_hl) =
                        highlight_vec.get(highlight_idx..highlight_idx.saturating_add(row_len))
                    {
                        row.set_highlight(new_hl.to_vec());
                    }
                    highlight_idx += row.as_bytes().len() + 2;
                }
            }
        }
    }
}
