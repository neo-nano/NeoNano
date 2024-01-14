use std::cmp;

use termion::color;
use unicode_segmentation::UnicodeSegmentation;

use crate::highlighting;
use crate::highlighting::Type;
use crate::HighlightingOptions;
use crate::SearchDirection;

#[derive(Default)]
pub struct Row {
    string: String,
    highlight: Vec<Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlight: Vec::new(),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> Vec<String> {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result: Vec<String> = Vec::new();
        let mut current_highlighting = &Type::None;
        for (index, graphme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = graphme.chars().next() {
                let mut current_str = String::new();
                let highlighting_type = self.highlight.get(index).unwrap_or(&Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    if current_highlighting == &Type::None {
                        let start_highlight = format!("{}", color::Fg(color::Reset));
                        current_str.push_str(start_highlight.as_str());
                    } else {
                        let start_highlight =
                            format!("{}", color::Fg(highlighting_type.to_color()));
                        current_str.push_str(start_highlight.as_str());
                    }
                }

                if c == '\t' {
                    current_str.push_str("  ");
                } else {
                    current_str.push(c);
                }
                result.push(current_str);
            }
        }

        let end_highlight = format!("{}", color::Fg(color::Reset));
        result.push(end_highlight);
        result
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }

        let mut result: String = String::new();
        let mut length = 0;
        for (index, graphme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(graphme);
        }
        self.len = length;
        self.string = result;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }

        let mut result: String = String::new();
        let mut length = 0;
        for (index, graphme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(graphme);
            }
        }
        self.len = length;
        self.string = result;
    }

    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row: String = String::new();
        let mut length = 0;
        let mut splitted_row: String = String::new();
        let mut splitted_length = 0;
        for (index, graphme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(graphme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(graphme);
            }
        }
        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            highlight: Vec::new(),
            len: splitted_length,
        }
    }

    pub fn find(&self, query: &str, after: usize, direction: SearchDirection) -> Option<usize> {
        if after > self.len || query.is_empty() {
            return None;
        }
        let start = if direction == SearchDirection::Forward {
            after
        } else {
            0
        };

        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            after
        };

        let sub_string: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            sub_string.find(query)
        } else {
            sub_string.rfind(query)
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (graphme_index, (byte_index, _)) in
                sub_string[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(start + graphme_index);
                }
            }
        }
        None
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }

    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }

    pub fn set_highlight(&mut self, vector: Vec<Type>) {
        self.highlight = vector;
    }
}
