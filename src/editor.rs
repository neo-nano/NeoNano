use std::env;
use std::time::{Duration, Instant};

use anyhow::{Error, Result};
use termion::color;
use termion::event::Key;
use unicode_segmentation::UnicodeSegmentation;

use crate::Document;
use crate::Row;
use crate::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const QUIT_TIMES: u8 = 3;

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, PartialEq, Copy)]
pub enum SearchDirection {
    Forward,
    Backward,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }
            if self.should_quit {
                break;
            }
            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut initial_status =
            String::from("HELP: Ctrl-S = Save | Ctrl-F = Search | Ctrl-Q = Quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to Initialize Terminal"),
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
            document,
        }
    }

    fn draw_welcome_message(&self) -> Vec<String> {
        let mut welcome_message = format!("Hecto Editor -- version {VERSION}");
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding);
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        welcome_message
            .graphemes(true)
            .map(String::from)
            .collect::<Vec<String>>()
    }

    pub fn draw_row(&self, row: &Row) -> Vec<String> {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        row.render(start, end)
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for terminal_row in 0..height {
            let mut row_array: Vec<String>;
            Terminal::clear_current_line();
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                row_array = self.draw_row(row);
            } else if self.document.is_empty() && terminal_row == height / 3 {
                row_array = self.draw_welcome_message();
            } else {
                row_array = vec![String::from("~"), String::from("\r")];
            }
            for floating_idx in 0..self.document.floating_len() {
                if let Some(floating) = self.document.floating(floating_idx) {
                    row_array = floating.render(&row_array, terminal_row as usize);
                }
            }

            println!("{}{}\r", color::Fg(color::Reset), row_array.concat());
        }
    }

    fn refresh_screen(&self) -> Result<()> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Good bye \r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn process_keypress(&mut self) -> Result<()> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
                self.document.clear_floating();
            }
            Key::Ctrl('q') => {
                if !self.document.is_dirty() {
                    self.should_quit = true;
                    return Ok(());
                }
                if self.quit_times == 1 {
                    self.should_quit = true;
                } else {
                    self.quit_times -= 1;
                    let unsaved_msg: String = format!(
                        "WARNING! Unsaved changes will be discarded! Press Ctrl-Q {} times to quit.",
                        self.quit_times
                    );
                    self.status_message = StatusMessage::from(unsaved_msg);
                }
            }
            Key::Ctrl('s') => {
                self.save();
                self.document.clear_floating();
            }
            Key::Ctrl('f') => self.search(),
            Key::F(1) => self.hover(),
            Key::Delete => {
                self.document.clear_floating();
                self.document.delete(&self.cursor_position);
                self.document.clear_floating();
            }
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
                self.document.clear_floating();
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => {
                self.move_cursor(pressed_key);
            }
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.offset;

        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let height = self.document.len();
        let terminal_height = self.terminal.size().height as usize;
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            Key::Left => {
                if x >= 1 {
                    x -= 1
                } else if y >= 1 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len()
                    } else {
                        x = 0
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }
        self.cursor_position = Position { x, y }
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };
        let mut file_name = "[No File]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{} | {}/{}",
            self.document.file_type(),
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );
        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width.saturating_sub(len)));
        }

        status = format!("{status}{line_indicator}");
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{status}\r");
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{text}");
        }
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File Saved successfully".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn prompt<C>(&mut self, prompt: &str, mut callback: C) -> Result<Option<String>>
    where
        C: FnMut(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len().saturating_sub(1));
                    }
                }
                Key::Char('\n') | Key::Ctrl('c') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }

                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            };

            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn search(&mut self) {
        let prev_position = self.cursor_position.clone();
        let mut direction = SearchDirection::Forward;
        let query = self
            .prompt(&"Search: ", |editor, key, query| {
                let mut moved = false;
                match key {
                    Key::Down | Key::Right => {
                        direction = SearchDirection::Forward;
                        editor.move_cursor(Key::Right);
                        moved = true;
                    }
                    Key::Up | Key::Left => {
                        direction = SearchDirection::Backward;
                    }
                    _ => {
                        direction = SearchDirection::Forward;
                    }
                }
                if let Some(pos) = editor
                    .document
                    .find(&query, &editor.cursor_position, direction)
                {
                    editor.cursor_position = pos;
                    editor.scroll();
                } else if moved {
                    editor.move_cursor(Key::Left);
                }
                // editor.document.highlight(Some(query.as_str()));
            })
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = prev_position;
            self.scroll();
        }
        // self.document.highlight(None);
    }

    fn hover(&mut self) {
        self.document
            .hover(self.cursor_position.x as u32, self.cursor_position.y as u32);
    }
}

fn die(e: Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
