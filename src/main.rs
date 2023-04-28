mod document;
mod editor;
mod row;
mod terminal;
pub use document::Document;
pub use editor::Position;
pub use terminal::Terminal;
pub use row::Row;
use editor::Editor;

fn main() {
    Editor::default().run();
}
