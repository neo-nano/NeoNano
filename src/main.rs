mod editor;
mod terminal;
use editor::Editor;

use terminal::Terminal;

fn main() {
    Editor::default().run();
}
