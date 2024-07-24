use std::io;

use cosmic_text::{
    Attrs, Buffer, Color, Edit, Editor, Family, FontSystem, Metrics, Shaping, Weight,
};
use markdown::{tokenize, Block, ListItem, Span};
use syntect::{
    easy::HighlightLines,
    highlighting::{Style, Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

use super::metrics;

pub struct Parser<'a, 'b> {
    spans: Vec<(&'a str, Attrs<'b>)>,
}

impl<'a, 'b> Parser<'a, 'b>
where
    'b: 'a,
{
    pub fn new() -> Self {
        Self { spans: Vec::new() }
    }

    pub fn get_spans(self) -> Vec<(&'a str, Attrs<'b>)> {
        self.spans
    }

    pub fn run<'block>(&mut self, block: &'block Vec<Block>)
    where
        'block: 'b,
    {
        for block in block {
            self.parse_block(block, Attrs::new());
            self.spans.push(("\n", Attrs::new()));
        }
    }

    fn parse_block<'block>(&mut self, block: &'block Block, attrs: Attrs<'block>)
    where
        'block: 'b,
    {
        match block {
            Block::Header(span, level) => {
                for item in span {
                    match level {
                        1 => {
                            let attrs = attrs.metrics(metrics(24.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        2 => {
                            let attrs = attrs.metrics(metrics(22.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        3 => {
                            let attrs = attrs.metrics(metrics(20.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        4 => {
                            let attrs = attrs.metrics(metrics(18.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        5 => {
                            let attrs = attrs.metrics(metrics(16.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        6 => {
                            let attrs = attrs.metrics(metrics(14.0));
                            attrs.weight(Weight::BOLD);
                            self.parse_span(item, attrs);
                        }
                        _ => self.parse_span(item, Attrs::new()),
                    }
                }
                self.spans.push(("\n", Attrs::new()));
            }
            Block::Paragraph(span) => {
                for item in span {
                    self.parse_span(item, attrs);
                }
                self.spans.push(("\n", Attrs::new()));
            }
            Block::Blockquote(blockquote) => {
                for item in blockquote {
                    let attrs = attrs.family(Family::Monospace);
                    self.parse_block(item, attrs);
                }
                self.spans.push(("\n", Attrs::new()));
            }
            Block::CodeBlock(lang, code) => {
                let extension = if let Some(lang) = lang {
                    language_to_extension(lang)
                } else {
                    "txt"
                };

                let attrs = attrs.family(Family::Monospace);
                let code_block = highlight_code(code, extension, attrs);

                self.spans.extend(code_block);
                self.spans.push(("\n", attrs));
            }
            Block::OrderedList(listitem, _type) => {
                for (num, item) in listitem.iter().enumerate() {
                    let attrs = attrs.family(Family::Serif);
                    self.spans
                        .push((Box::leak(Box::new(format!("{}.  ", num + 1))), attrs));
                    self.parse_listitem(item);
                    self.spans.push(("\n", attrs));
                }
                self.spans.push(("\n", Attrs::new()));
            }
            Block::UnorderedList(listitem) => {
                for item in listitem {
                    let attrs = attrs.family(Family::Serif);
                    self.spans.push((" - ", attrs));
                    self.parse_listitem(item);
                    self.spans.push(("\n", attrs));
                }
                self.spans.push(("\n", Attrs::new()));
            }
            Block::Raw(raw_text) => {
                self.spans.push((raw_text, Attrs::new()));
                self.spans.push(("\n", Attrs::new()));
            }
            Block::Hr => self.spans.push(("\n", Attrs::new())),
        }
    }

    fn parse_span<'c>(&mut self, span: &'c Span, attrs: Attrs<'c>)
    where
        'c: 'b,
    {
        match span {
            Span::Break => self.spans.push(("\n", attrs)),
            Span::Text(text) => self.spans.push((text, attrs)),
            Span::Code(code) => {
                let attrs = attrs.family(Family::Monospace);
                self.spans.push((code, attrs));
            }
            Span::Link(name, url, _title) => {
                let color = Attrs::new().color(link_color());
                let spans: &[(&str, Attrs)] = &[
                    (Box::leak(format!("{}: ", name).into_boxed_str()), attrs),
                    (url, color),
                ];
                self.spans.extend_from_slice(spans);
            }
            Span::Image(alt, url, title) => {
                let color = Attrs::new().color(link_color());
                let spans: &[(&str, Attrs)] = if let Some(title) = title {
                    &[
                        (Box::leak(format!("{}: ", title).into_boxed_str()), attrs),
                        (url, color),
                    ]
                } else {
                    &[
                        (Box::leak(format!("{}: ", alt).into_boxed_str()), attrs),
                        (url, color),
                    ]
                };
                self.spans.extend_from_slice(spans);
            }
            Span::Emphasis(emphasis) => {
                for item in emphasis {
                    let attrs = attrs.family(Family::Cursive);
                    self.parse_span(item, attrs);
                }
            }
            Span::Strong(strong) => {
                for item in strong {
                    let attrs = attrs.weight(Weight::BOLD);
                    self.parse_span(item, attrs);
                }
            }
        }
    }

    fn parse_listitem<'d>(&mut self, item: &'d ListItem)
    where
        'd: 'b,
    {
        match item {
            ListItem::Simple(simple) => {
                for item in simple {
                    self.parse_span(item, Attrs::new());
                }
            }
            ListItem::Paragraph(block) => {
                for item in block {
                    self.parse_block(item, Attrs::new());
                }
            }
        }
    }
}

fn highlight_code<'a>(
    code: &'a str,
    extension: &str,
    attrs: Attrs<'a>,
) -> Vec<(&'a str, Attrs<'a>)> {
    let mut result: Vec<(&'a str, Attrs)> = Vec::new();

    let ps = SyntaxSet::load_defaults_newlines();
    let syntax_theme = syntax_theme();

    let syntax = ps.find_syntax_by_extension(extension).unwrap();
    let mut h = HighlightLines::new(syntax, &syntax_theme);
    for line in LinesWithEndings::from(code) {
        let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();

        for (style, text) in ranges {
            let fg = style.foreground;
            let color = Color::rgb(fg.r, fg.g, fg.b);

            let attrs = attrs.color(color);
            result.push((text, attrs));
        }
    }

    result
}

fn link_color() -> Color {
    if !crate::theme::is_dark() {
        return Color::rgb(5, 200, 95);
    }

    Color::rgb(125, 206, 243)
}

fn syntax_theme() -> Theme {
    let ts = cosmic_syntax();

    if !crate::theme::is_dark() {
        return ts.themes.get("COSMIC Light").unwrap().clone();
    }

    ts.themes.get("COSMIC Dark").unwrap().clone()
}

fn cosmic_syntax() -> ThemeSet {
    let lazy_theme_set = two_face::theme::LazyThemeSet::from(two_face::theme::extra());
    let mut theme_set = syntect::highlighting::ThemeSet::from(&lazy_theme_set);
    // Hardcoded COSMIC themes
    for (theme_name, theme_data) in &[
        ("COSMIC Dark", cosmic_syntax_theme::COSMIC_DARK_TM_THEME),
        ("COSMIC Light", cosmic_syntax_theme::COSMIC_LIGHT_TM_THEME),
    ] {
        let mut cursor = io::Cursor::new(theme_data);
        match syntect::highlighting::ThemeSet::load_from_reader(&mut cursor) {
            Ok(mut theme) => {
                // Use libcosmic theme for background and gutter
                theme.settings.background = Some(syntect::highlighting::Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                });
                theme.settings.gutter = Some(syntect::highlighting::Color {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                });
                theme_set.themes.insert(theme_name.to_string(), theme);
            }
            Err(err) => {
                eprintln!("failed to load {:?} syntax theme: {}", theme_name, err);
            }
        }
    }

    theme_set
}

fn language_to_extension(lang: &str) -> &'static str {
    match lang {
        "python" => "py",
        "javascript" => "js",
        "java" => "java",
        "c" => "c",
        "cpp" => "cpp",
        "c++" => "cpp",
        "csharp" => "cs",
        "c#" => "cs",
        "php" => "php",
        "ruby" => "rb",
        "swift" => "swift",
        "kotlin" => "kt",
        "go" => "go",
        "r" => "r",
        "perl" => "pl",
        "shell" => "sh",
        "bash" => "sh",
        "objective-c" => "m",
        "objective-c++" => "mm",
        "typescript" => "ts",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        "matlab" => "m",
        "scala" => "scala",
        "rust" => "rs",
        "dart" => "dart",
        "elixir" => "ex",
        "haskell" => "hs",
        "lua" => "lua",
        "assembly" => "asm",
        "fortran" => "f90",
        "pascal" => "pas",
        "cobol" => "cob",
        "erlang" => "erl",
        "fsharp" => "fs",
        "f#" => "fs",
        "julia" => "jl",
        "groovy" => "groovy",
        "ada" => "adb",
        "markdown" => "md",
        _ => "txt",
    }
}
