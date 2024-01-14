pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

#[derive(Default, Clone)]
pub struct HighlightingOptions {
    hl_query: Option<&'static str>,
    inj_query: Option<&'static str>,
}

impl HighlightingOptions {
    pub fn get_hl_query(&self) -> Option<&str> {
        self.hl_query
    }
    pub fn get_inj_query(&self) -> Option<&str> {
        self.inj_query
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl FileType {
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn highlighting_options(&self) -> HighlightingOptions {
        self.hl_opts.clone()
    }

    pub fn from(file_name: &str) -> Option<Self> {
        if file_name.ends_with(".rs") {
            return Some(Self {
                name: String::from("Rust"),
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_rust::HIGHLIGHT_QUERY),
                    inj_query: Some(tree_sitter_rust::INJECTIONS_QUERY),
                },
            });
        }
        None
    }
}
