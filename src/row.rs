use crate::highlighting;
use crate::HighlightingOptions;
use crate::SearchDirection;
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    highlighting: Vec<highlighting::Type>,
    len: usize,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            highlighting: Vec::new(),
            len: slice.graphemes(true).count(),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlighting::Type::None;
        for (index, graphme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = graphme.chars().next() {
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlighting::Type::None);
                if highlighting_type != current_highlighting {
                    current_highlighting = highlighting_type;
                    let start_highlight =
                        format!("{}", termion::color::Fg(highlighting_type.to_color()));
                    result.push_str(start_highlight.as_str());
                }
                if c == '\t' {
                    result.push_str("  ");
                } else {
                    result.push(c);
                }
            }
        }

        let end_highlight = format!("{}", termion::color::Fg(color::Reset));
        result.push_str(end_highlight.as_str());
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
            highlighting: Vec::new(),
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

    fn highlight_match(&mut self, word: Option<&str>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }
            let mut index = 0;
            while let Some(search_match) = self.find(word, index, SearchDirection::Forward) {
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                {
                    for i in index.saturating_add(search_match)..next_index {
                        self.highlighting[i] = highlighting::Type::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.characters() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let char_len = if *next_char == '\\' { 3 } else { 2 };
                let closing_index = index.saturating_add(char_len);
                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..char_len.saturating_add(1) {
                            self.highlighting.push(highlighting::Type::Character);
                            *index += 1;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(highlighting::Type::Comment);
                        *index += 1;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            loop {
                self.highlighting.push(highlighting::Type::String);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char == '"' {
                        break;
                    }
                }
            }
            self.highlighting.push(highlighting::Type::String);
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                let prev_char = chars[*index - 1];
                if !prev_char.is_ascii_punctuation() && !prev_char.is_ascii_whitespace() {
                    return false;
                }
            }

            loop {
                self.highlighting.push(highlighting::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    }
                } else {
                    break;
                }
            }
            return true;
        }
        false
    }

    pub fn highlight(&mut self, opts: HighlightingOptions, word: Option<&str>) {
        self.highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut index = 0;
        while let Some(c) = chars.get(index) {
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                continue;
            }
            self.highlighting.push(highlighting::Type::None);
            index += 1;
        }
        self.highlight_match(word);
    }
}

#[cfg(test)]
mod tests {
    use crate::{highlighting, FileType, Row};

    fn highlight(content: &str, expected: &[highlighting::Type]) {
        let mut row = Row::from(content);
        let filetype = FileType::from("test.rs");
        row.highlight(filetype.highlighting_options(), None);
        for i in 0..expected.len() {
            println!("'{:?}' == '{:?}'", row.highlighting[i], expected[i]);
            assert_eq!(row.highlighting[i], expected[i]);
        }
    }

    #[test]
    fn str_highlight() {
        let hl_content = "\"123\"";
        let hl_expected = [
            highlighting::Type::String,
            highlighting::Type::String,
            highlighting::Type::String,
            highlighting::Type::String,
            highlighting::Type::String,
        ];

        highlight(hl_content, &hl_expected);
    }
    #[test]
    fn int_highlight() {
        let hl_content = "123.0";
        let hl_expected = [
            highlighting::Type::Number,
            highlighting::Type::Number,
            highlighting::Type::Number,
            highlighting::Type::Number,
            highlighting::Type::Number,
        ];

        highlight(hl_content, &hl_expected);
    }
    #[test]
    fn comment_highlight() {
        let hl_content = "// comment";
        let hl_expected = [
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
            highlighting::Type::Comment,
        ];

        highlight(hl_content, &hl_expected);
    }
    #[test]
    fn char_highlight() {
        let hl_content = "'@'";
        let hl_expected = [
            highlighting::Type::Character,
            highlighting::Type::Character,
            highlighting::Type::Character,
        ];

        highlight(hl_content, &hl_expected);
    }
    #[test]
    fn char_backslash_highlight() {
        let hl_content = "'\\a'";
        let hl_expected = [
            highlighting::Type::Character,
            highlighting::Type::Character,
            highlighting::Type::Character,
            highlighting::Type::Character,
        ];

        highlight(hl_content, &hl_expected);
    }
}
