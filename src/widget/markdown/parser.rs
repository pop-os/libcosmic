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

pub fn markdown_to_string<'a, 'b>(content: &'a str, attrs: Attrs<'b>) -> Vec<(&'a str, Attrs<'b>)>
where
    'b: 'a,
{
    let mut result: Vec<(&'a str, Attrs)> = Vec::new();

    for block in tokenize(content) {
        result.extend(parse_block(block, attrs));
        result.push(("\n", attrs));
    }

    result
}

fn parse_block<'a, 'b>(block: Block, attrs: Attrs<'b>) -> Vec<(&'a str, Attrs<'b>)>
where
    'b: 'a,
{
    let mut result: Vec<(&'a str, Attrs)> = Vec::new();

    match block {
        Block::Header(span, level) => {
            for item in span {
                match level {
                    1 => {
                        let attrs = attrs.metrics(metrics(24.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    2 => {
                        let attrs = attrs.metrics(metrics(22.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    3 => {
                        let attrs = attrs.metrics(metrics(20.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    4 => {
                        let attrs = attrs.metrics(metrics(18.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    5 => {
                        let attrs = attrs.metrics(metrics(16.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    6 => {
                        let attrs = attrs.metrics(metrics(14.0));
                        attrs.weight(Weight::BOLD);
                        result.extend(parse_span(item, attrs));
                    }
                    _ => result.extend(parse_span(item, attrs)),
                }
            }
            result.push(("\n", attrs));
        }
        Block::Paragraph(span) => {
            for item in span {
                result.extend(parse_span(item, attrs));
            }
            result.push(("\n", attrs));
        }
        Block::Blockquote(blockquote) => {
            for item in blockquote {
                let attrs = attrs.family(Family::Monospace);
                result.extend(parse_block(item, attrs));
            }
            result.push(("\n", attrs));
        }
        Block::CodeBlock(lang, code) => {
            if let Some(lang) = lang {
                let code_block = highlight_code(
                    Box::leak(code.into_boxed_str()),
                    language_to_extension(lang),
                    attrs,
                );

                result.extend(code_block);
            } else {
                result.push((Box::leak(code.into_boxed_str()), attrs));
            };

            result.push(("\n", attrs));
        }
        Block::OrderedList(listitem, _type) => {
            for (num, item) in listitem.iter().enumerate() {
                let attrs = attrs.family(Family::Serif);
                result.push((Box::leak(format!("{}. ", num + 1).into_boxed_str()), attrs));
                result.extend(parse_listitem(item.to_owned(), attrs));
                result.push(("\n", attrs));
            }
            result.push(("\n", attrs));
        }
        Block::UnorderedList(listitem) => {
            for item in listitem {
                let attrs = attrs.family(Family::Serif);
                result.push((" - ", attrs));
                result.extend(parse_listitem(item.to_owned(), attrs));
                result.push(("\n", attrs));
            }
            result.push(("\n", attrs));
        }
        Block::Raw(raw_text) => {
            result.push((Box::leak(raw_text.into_boxed_str()), attrs));

            result.push(("\n", attrs));
        }
        Block::Hr => result.push(("\n", attrs)),
    }

    result
}

fn parse_span<'a>(span: Span, attrs: Attrs<'a>) -> Vec<(&'a str, Attrs)> {
    let mut result: Vec<(&'a str, Attrs)> = Vec::new();

    match span {
        Span::Break => result.push(("\n", attrs)),
        Span::Text(text) => result.push((Box::leak(text.into_boxed_str()), attrs)),
        Span::Code(code) => {
            attrs.family(Family::Monospace);
            result.push((Box::leak(code.into_boxed_str()), attrs));
        }
        Span::Link(name, url, _title) => {
            let spans: &[(&str, Attrs)] = &[
                (Box::leak(format!("{}: ", name).into_boxed_str()), attrs),
                (Box::leak(url.into_boxed_str()), attrs.color(link_color())),
            ];
            result.extend_from_slice(spans);
        }
        Span::Image(alt, url, title) => {
            let spans: &[(&str, Attrs)] = if let Some(title) = title {
                &[
                    (Box::leak(format!("{}: ", title).into_boxed_str()), attrs),
                    (Box::leak(url.into_boxed_str()), attrs.color(link_color())),
                ]
            } else {
                &[
                    (Box::leak(format!("{}: ", alt).into_boxed_str()), attrs),
                    (Box::leak(url.into_boxed_str()), attrs.color(link_color())),
                ]
            };
            result.extend_from_slice(spans);
        }
        Span::Emphasis(emphasis) => {
            for item in emphasis {
                let attrs = attrs.family(Family::Cursive);
                result.extend(parse_span(item, attrs));
            }
        }
        Span::Strong(strong) => {
            for item in strong {
                let attrs = attrs.weight(Weight::BOLD);
                result.extend(parse_span(item, attrs));
            }
        }
    }

    result
}

fn parse_listitem<'a>(item: ListItem, attrs: Attrs<'a>) -> Vec<(&'a str, Attrs)> {
    let mut result: Vec<(&'a str, Attrs)> = Vec::new();

    match item {
        ListItem::Simple(simple) => {
            for item in simple {
                result.extend(parse_span(item, attrs));
            }
        }
        ListItem::Paragraph(block) => {
            for item in block {
                result.extend(parse_block(item, attrs));
            }
        }
    }

    result
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

fn language_to_extension(lang: String) -> &'static str {
    match lang.as_str() {
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
