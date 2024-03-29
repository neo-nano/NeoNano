use std::env::current_dir;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use lsp_types::HoverContents;
use unicode_segmentation::UnicodeSegmentation;

use crate::editor::SearchDirection;
use crate::floating_item::FloatingItem;
use crate::highlighting::Highlight;
use crate::lsp::LspConnector;
use crate::Row;
use crate::{FileType, Position};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
    file_type: FileType,
    floatings: Vec<FloatingItem>,
    lsp: Option<LspConnector>,
    highlighter: Option<Highlight>,
}

impl Document {
    pub fn open(file_name: &str) -> Result<Self> {
        let contents = fs::read_to_string(file_name)?;
        let file_type = FileType::from(file_name).unwrap_or_default();
        let hl_opt = file_type.highlighting_options();
        let highlighter = match Highlight::new(
            hl_opt.get_lang().unwrap(),
            hl_opt.get_hl_query().unwrap(),
            hl_opt.get_inj_query().unwrap(),
        ) {
            Ok(highlighter) => Some(highlighter),
            Err(_) => None,
        };
        let lsp = match LspConnector::new(
            file_type.lsp_name().unwrap_or_default(),
            file_type.lsp_args().unwrap_or_default(),
            file_type.name(),
            current_dir()
                .unwrap_or(PathBuf::new())
                .join(
                    PathBuf::from(file_name)
                        .canonicalize()
                        .unwrap_or(PathBuf::new()),
                )
                .into_os_string()
                .into_string()
                .unwrap_or(String::from("Unknown File")),
        ) {
            Ok(lsp) => Some(lsp),
            Err(_) => None,
        };
        let mut rows: Vec<Row> = Vec::new();
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        let mut res = Self {
            rows,
            file_name: Some(file_name.to_owned()),
            dirty: false,
            file_type,
            floatings: vec![],
            lsp,
            highlighter,
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

    pub fn save(&mut self) -> Result<()> {
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
        if let Some(highlighter) = self.highlighter.as_mut() {
            if let Ok(highlight_vec) = highlighter.highlight(chars) {
                let mut highlight_idx: usize = 0;
                for row in &mut self.rows {
                    let row_len = row.as_bytes().len();
                    if let Some(new_hl) =
                        highlight_vec.get(highlight_idx..highlight_idx.saturating_add(row_len))
                    {
                        row.set_highlight(new_hl.to_vec());
                    }
                    highlight_idx += row.as_bytes().len().saturating_add(2);
                }
            }
        }
    }

    pub fn clear_floating(&mut self) {
        self.floatings.clear();
    }

    pub fn hover(&mut self, x: u32, y: u32) {
        if let Some(lsp) = self.lsp.as_mut() {
            if !lsp.is_initialized() {
                let a = self.rows.iter().map(|r| r.as_str()).collect::<Vec<&str>>();
                lsp.init(a.join("\r\n"));
            }

            if let Some(hover) = lsp.hover(y, x) {
                match hover.contents {
                    HoverContents::Scalar(_) => (),
                    HoverContents::Markup(content) => {
                        let txt = content.value;
                        self.floatings.clear();
                        let width = txt
                            .lines()
                            .map(|x| x.graphemes(true).count())
                            .max()
                            .unwrap_or(0);
                        self.floatings.append(&mut vec![FloatingItem::new(
                            Position {
                                x: x as usize,
                                y: y.saturating_add(1) as usize,
                            },
                            width,
                            txt.lines().filter(|x| !x.is_empty()).count(),
                            txt.lines()
                                .map(ToString::to_string)
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<String>>(),
                        )]);
                    }
                    HoverContents::Array(_) => (),
                    // TODO
                }
            }
        }
    }
}
