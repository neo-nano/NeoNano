use tree_sitter::Language;

pub struct FileType {
    name: String,
    lsp_name: Option<&'static str>,
    lsp_args: Option<Vec<&'static str>>,
    hl_opts: HighlightingOptions,
}

#[derive(Default, Clone)]
pub struct HighlightingOptions {
    hl_query: Option<&'static str>,
    inj_query: Option<&'static str>,
    lang: Option<Language>,
}

impl HighlightingOptions {
    pub fn get_hl_query(&self) -> Option<&str> {
        self.hl_query
    }
    pub fn get_inj_query(&self) -> Option<&str> {
        self.inj_query
    }
    pub fn get_lang(&self) -> Option<Language> {
        self.lang
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No filetype"),
            lsp_name: None,
            lsp_args: None,
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

    pub fn lsp_name(&self) -> Option<&str> {
        self.lsp_name
    }
    pub fn lsp_args(&self) -> Option<Vec<&str>> {
        self.lsp_args.clone()
    }

    pub fn from(file_name: &str) -> Option<Self> {
        if file_name.ends_with(".rs") {
            return Some(Self {
                name: String::from("Rust"),
                lsp_name: Some("rust-analyzer"),
                lsp_args: None,
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_rust::HIGHLIGHT_QUERY),
                    inj_query: Some(""),
                    lang: Some(tree_sitter_rust::language()),
                },
            });
        } else if file_name.ends_with(".go") {
            return Some(Self {
                name: String::from("Go"),
                lsp_name: Some("gopls"),
                lsp_args: None,
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_go::HIGHLIGHT_QUERY),
                    inj_query: Some(""),
                    lang: Some(tree_sitter_go::language()),
                },
            });
        } else if file_name.ends_with(".cpp") {
            return Some(Self {
                name: String::from("Cpp"),
                lsp_name: Some("clangd"),
                lsp_args: None,
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_cpp::HIGHLIGHT_QUERY),
                    inj_query: Some(""),
                    lang: Some(tree_sitter_cpp::language()),
                },
            });
        } else if file_name.ends_with(".c") {
            return Some(Self {
                name: String::from("C"),
                lsp_name: Some("clangd"),
                lsp_args: None,
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_c::HIGHLIGHT_QUERY),
                    inj_query: Some(""),
                    lang: Some(tree_sitter_c::language()),
                },
            });
        } else if file_name.ends_with(".py") {
            return Some(Self {
                name: String::from("Python"),
                lsp_name: Some("pyright"),
                lsp_args: Some(vec!["--stdio"]),
                hl_opts: HighlightingOptions {
                    hl_query: Some(tree_sitter_python::HIGHLIGHT_QUERY),
                    inj_query: Some(""),
                    lang: Some(tree_sitter_python::language()),
                },
            });
        }
        None
    }
}
