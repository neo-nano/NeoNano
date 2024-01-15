use termion::color;
use tree_sitter::Language;
use tree_sitter_highlight::{Error, HighlightConfiguration};
use tree_sitter_highlight::{HighlightEvent, Highlighter};

const HIGHLIGHTS: [(&str, Type); 52] = [
    ("attribute", Type::Attribute),
    ("boolean", Type::Boolean),
    ("carriage-return", Type::CarriageReturn),
    ("comment", Type::Comment),
    ("comment.documentation", Type::CommentDocumentation),
    ("constant", Type::Constant),
    ("constant.builtin", Type::ConstantBuiltin),
    ("constructor", Type::Constructor),
    ("constructor.builtin", Type::ConstructorBuiltin),
    ("embedded", Type::Embedded),
    ("error", Type::Error),
    ("escape", Type::Escape),
    ("function", Type::Function),
    ("function.builtin", Type::FunctionBuiltin),
    ("keyword", Type::Keyword),
    ("markup", Type::Markup),
    ("markup.bold", Type::MarkupBold),
    ("markup.heading", Type::MarkupHeading),
    ("markup.italic", Type::MarkupItalic),
    ("markup.link", Type::MarkupLink),
    ("markup.link.url", Type::MarkupLinkUrl),
    ("markup.list", Type::MarkupList),
    ("markup.list.checked", Type::MarkupListChecked),
    ("markup.list.numbered", Type::MarkupListNumbered),
    ("markup.list.unchecked", Type::MarkupListUnchecked),
    ("markup.list.unnumbered", Type::MarkupListUnnumbered),
    ("markup.quote", Type::MarkupQuote),
    ("markup.raw", Type::MarkupRaw),
    ("markup.raw.block", Type::MarkupRawBlock),
    ("markup.raw.inline", Type::MarkupRawInline),
    ("markup.strikethrough", Type::MarkupStrikethrough),
    ("module", Type::Module),
    ("number", Type::Number),
    ("operator", Type::Operator),
    ("property", Type::Property),
    ("property.builtin", Type::PropertyBuiltin),
    ("punctuation", Type::Punctuation),
    ("punctuation.bracket", Type::PunctuationBracket),
    ("punctuation.delimiter", Type::PunctuationDelimiter),
    ("punctuation.special", Type::PunctuationSpecial),
    ("string", Type::String),
    ("string.escape", Type::StringEscape),
    ("string.regexp", Type::StringRegexp),
    ("string.special", Type::StringSpecial),
    ("string.special.symbol", Type::StringSpecialSymbol),
    ("tag", Type::Tag),
    ("type", Type::Type),
    ("type.builtin", Type::TypeBuiltin),
    ("variable", Type::Variable),
    ("variable.builtin", Type::VariableBuiltin),
    ("variable.member", Type::VariableMember),
    ("variable.parameter", Type::VariableParameter),
];
pub struct Highlight {
    highlighter: Highlighter,
    config: HighlightConfiguration,
}

impl Highlight {
    pub fn new(lang: Language, hl_query: &str, inj_query: &str) -> Result<Self, String> {
        let highlighter = Highlighter::new();
        let config = HighlightConfiguration::new(lang, hl_query, inj_query, "");
        if let Ok(mut config) = config {
            config.configure(&HIGHLIGHTS.map(|x| x.0));
            return Ok(Self {
                highlighter,
                config,
            });
        }
        Err(String::from("Failed to initialize config"))
    }

    pub fn highlight(&mut self, code: &[u8]) -> Result<Vec<Type>, Error> {
        let mut res: Vec<Type> = vec![];
        let mut current_hl: Type = Type::None;
        for event in self
            .highlighter
            .highlight(&self.config, code, None, |_| None)?
        {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    if current_hl == Type::CarriageReturn {
                        continue;
                    }
                    for _ in start..end {
                        res.push(current_hl.clone())
                    }
                }
                HighlightEvent::HighlightStart(s) => current_hl = HIGHLIGHTS[s.0].1.clone(),
                HighlightEvent::HighlightEnd => current_hl = Type::None,
            }
        }
        Ok(res)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Type {
    None,
    Attribute,
    Boolean,
    CarriageReturn,
    Comment,
    CommentDocumentation,
    Constant,
    ConstantBuiltin,
    Constructor,
    ConstructorBuiltin,
    Embedded,
    Error,
    Escape,
    Function,
    FunctionBuiltin,
    Keyword,
    Markup,
    MarkupBold,
    MarkupHeading,
    MarkupItalic,
    MarkupLink,
    MarkupLinkUrl,
    MarkupList,
    MarkupListChecked,
    MarkupListNumbered,
    MarkupListUnchecked,
    MarkupListUnnumbered,
    MarkupQuote,
    MarkupRaw,
    MarkupRawBlock,
    MarkupRawInline,
    MarkupStrikethrough,
    Module,
    Number,
    Operator,
    Property,
    PropertyBuiltin,
    Punctuation,
    PunctuationBracket,
    PunctuationDelimiter,
    PunctuationSpecial,
    String,
    StringEscape,
    StringRegexp,
    StringSpecial,
    StringSpecialSymbol,
    Tag,
    Type,
    TypeBuiltin,
    Variable,
    VariableBuiltin,
    VariableMember,
    VariableParameter,
}

impl Type {
    pub fn to_color(&self) -> impl color::Color {
        match self {
            Type::None => color::Rgb(220, 138, 120),
            Type::Keyword => color::Rgb(0, 255, 0),
            Type::Attribute => color::Rgb(221, 120, 120),
            Type::Boolean => color::Rgb(234, 118, 203),
            Type::CarriageReturn => color::Rgb(136, 57, 239),
            Type::Comment => color::Rgb(92, 95, 119),
            Type::CommentDocumentation => color::Rgb(92, 95, 119),
            Type::Constant => color::Rgb(210, 15, 57),
            Type::ConstantBuiltin => color::Rgb(210, 15, 57),
            Type::Constructor => color::Rgb(234, 118, 203),
            Type::ConstructorBuiltin => color::Rgb(234, 118, 203),
            Type::Embedded => color::Rgb(23, 146, 153),
            Type::Error => color::Rgb(114, 135, 253),
            Type::Escape => color::Rgb(32, 159, 181),
            Type::Function => color::Rgb(223, 142, 29),
            Type::FunctionBuiltin => color::Rgb(223, 142, 29),
            Type::Module => color::Rgb(4, 165, 229),
            Type::Number => color::Rgb(114, 135, 253),
            Type::Operator => color::Rgb(32, 159, 181),
            Type::Property => color::Rgb(114, 135, 253),
            Type::PropertyBuiltin => color::Rgb(30, 102, 245),
            Type::Punctuation => color::Rgb(4, 165, 229),
            Type::PunctuationBracket => color::Rgb(4, 165, 229),
            Type::PunctuationDelimiter => color::Rgb(4, 165, 229),
            Type::PunctuationSpecial => color::Rgb(4, 165, 229),
            Type::String => color::Rgb(64, 160, 43),
            Type::StringEscape => color::Rgb(223, 142, 29),
            Type::StringRegexp => color::Rgb(223, 142, 29),
            Type::StringSpecial => color::Rgb(30, 102, 245),
            Type::StringSpecialSymbol => color::Rgb(210, 15, 57),
            Type::Tag => color::Rgb(220, 138, 120),
            Type::Type => color::Rgb(220, 138, 120),
            Type::TypeBuiltin => color::Rgb(220, 138, 120),
            Type::Variable => color::Rgb(23, 146, 153),
            Type::VariableBuiltin => color::Rgb(23, 146, 153),
            Type::VariableMember => color::Rgb(23, 146, 153),
            Type::VariableParameter => color::Rgb(23, 146, 153),
            _ => color::Rgb(0, 0, 0),
        }
    }
}
