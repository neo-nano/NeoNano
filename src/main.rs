#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
clippy::missing_docs_in_private_items,
clippy::implicit_return,
clippy::shadow_reuse,
clippy::print_stdout,
clippy::wildcard_enum_match_arm,
clippy::else_if_without_else
)]

pub use document::Document;
use editor::Editor;
pub use editor::Position;
use editor::SearchDirection;
pub use filetype::FileType;
pub use filetype::HighlightingOptions;
pub use row::Row;
pub use terminal::Terminal;

mod document;
mod editor;
mod filetype;
mod highlighting;
mod row;
mod floating_item;
mod terminal;

fn main() {
    Editor::default().run();
}
