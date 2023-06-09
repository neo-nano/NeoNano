use crate::SearchDirection;
use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        for graphme in self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if graphme == "\t" {
                result.push_str("  ");
            } else {
                result.push_str(graphme);
            }
        }
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
            len: splitted_length,
        }
    }

    pub fn find(&self, query: &str, after: usize, direction: SearchDirection) -> Option<usize> {
        if after > self.len {
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
}
