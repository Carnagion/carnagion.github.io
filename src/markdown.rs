use std::{
    io::{self, BufWriter, IntoInnerError},
    string::FromUtf8Error,
};

use comrak::{
    nodes::{AstNode, NodeHtmlBlock, NodeValue},
    Arena,
    ExtensionOptions,
    Options,
    ParseOptions,
    Plugins,
    RenderOptions,
    RenderPlugins,
};

use inkjet::theme::{self, Theme};

use latex2mathml::{latex_to_mathml, DisplayStyle, LatexError};

use serde::de::{DeserializeOwned, Error};

use thiserror::Error;

pub mod highlighter;
use highlighter::Inkjet;

#[derive(Debug, Clone)]
pub struct Markdown<'a> {
    pub ast: &'a AstNode<'a>,
}

impl<'a> Markdown<'a> {
    pub fn new(md: &str, arena: &'a Arena<AstNode<'a>>) -> Self {
        let options = Options {
            extension: extension_options(),
            parse: parse_options(),
            render: render_options(),
        };

        let ast = comrak::parse_document(arena, md, &options);

        Self { ast }
    }

    pub fn extract_front_matter<T>(&self) -> Result<T, toml::de::Error>
    where
        T: DeserializeOwned,
    {
        self.ast
            .descendants()
            .find_map(|node| {
                let value = &node.data.borrow().value;
                let front_matter = match value {
                    NodeValue::FrontMatter(front_matter) => Some(front_matter),
                    _ => None,
                }?;

                // Remove the front matter node
                node.detach();

                let front_matter =
                    front_matter.trim_matches(|c: char| c.is_whitespace() || c == '+');
                Some(toml::from_str(front_matter))
            })
            .unwrap_or_else(|| Err(toml::de::Error::custom("no front matter found")))
    }

    pub fn to_html(&self) -> Result<String, ToHtmlError> {
        // NOTE: We replace LaTeX equations with MathML here rather than on construction because the AST may be modified
        //       from the outside even after construction.
        replace_mathml(self.ast)?;

        let theme = Theme::from_helix(theme::vendored::GRUVBOX).unwrap(); // PANICS: This should be a valid theme.
        let inkjet = Inkjet::new(theme);

        let render_plugins = RenderPlugins {
            codefence_syntax_highlighter: Some(&inkjet),
            ..RenderPlugins::default()
        };

        let plugins = Plugins {
            render: render_plugins,
        };

        let options = Options {
            extension: extension_options(),
            parse: parse_options(),
            render: render_options(),
        };

        let mut html = BufWriter::new(Vec::new());
        comrak::format_html_with_plugins(self.ast, &options, &mut html, &plugins)?;

        let html = String::from_utf8(html.into_inner()?)?;
        Ok(html)
    }

    pub fn reading_time(&self) -> (usize, usize) {
        // NOTE: These statistics are taken from Firefox's source code, which uses them for its reader view.
        //       See https://searchfox.org/mozilla-central/source/mobile/android/android-components/components/feature/readerview/src/main/assets/extensions/readerview/readerview.js#221.
        let cpm = 987;
        let variance = 118;

        let chars = self
            .ast
            .descendants()
            .filter_map(|node| {
                let node = &node.data.borrow().value;
                let text = extract_text(node)?;
                // NOTE: I would normally use the `unicode-segmentation` crate for this, but I write primarily in English with very few
                //       emojis, so the vast majority of text I write will be ASCII.
                Some(text.len())
            })
            .sum::<usize>();

        let fast = chars.div_euclid(cpm + variance);
        let slow = chars.div_ceil(cpm - variance);

        (fast, slow)
    }
}

fn replace_mathml<'a>(ast: &'a AstNode<'a>) -> Result<(), LatexError> {
    // Replace Latex equations with MathML
    for node in ast.descendants() {
        let node = &mut node.data.borrow_mut().value;
        let NodeValue::Math(math) = node else {
            continue;
        };

        if math.display_math {
            let mathml = latex_to_mathml(&math.literal, DisplayStyle::Block)?;
            *node = NodeValue::HtmlBlock(NodeHtmlBlock {
                block_type: 1,
                literal: mathml,
            });
        } else {
            let mathml = latex_to_mathml(&math.literal, DisplayStyle::Inline)?;
            *node = NodeValue::HtmlInline(mathml);
        }
    }

    Ok(())
}

fn extract_text(node: &NodeValue) -> Option<&str> {
    match node {
        NodeValue::Code(code) => Some(&code.literal),
        NodeValue::CodeBlock(code) => Some(&code.literal),
        NodeValue::EscapedTag(tag) => Some(tag),
        NodeValue::HtmlBlock(html) => Some(&html.literal),
        NodeValue::HtmlInline(html) => Some(html),
        NodeValue::Link(link) => Some(&link.title),
        NodeValue::Math(math) => Some(&math.literal),
        NodeValue::Text(text) => Some(text),
        NodeValue::ShortCode(shortcode) => Some(&shortcode.emoji),
        _ => None,
    }
}

#[derive(Debug, Error)]
pub enum ToHtmlError {
    #[error("could not parse LaTeX expression: {0}")]
    Latex(#[from] LatexError),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("could not flush HTML buffer: {0}")]
    Flush(#[from] IntoInnerError<BufWriter<Vec<u8>>>),
    #[error("rendered HTML contained invalid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),
}

fn extension_options() -> ExtensionOptions<'static> {
    ExtensionOptions {
        autolink: true,
        description_lists: true,
        footnotes: true,
        front_matter_delimiter: Some(String::from("+++")),
        greentext: true,
        header_ids: Some(String::new()),
        math_dollars: true,
        multiline_block_quotes: true,
        shortcodes: true,
        spoiler: true,
        strikethrough: true,
        subscript: true,
        superscript: true,
        table: true,
        tagfilter: true, // NOTE: Needs to be enabled to escape potentially dangerous elements. See `render_options` below.
        tasklist: true,
        underline: true,
        ..ExtensionOptions::default()
    }
}

fn parse_options() -> ParseOptions<'static> {
    ParseOptions {
        smart: true,
        ..ParseOptions::default()
    }
}

fn render_options() -> RenderOptions {
    RenderOptions {
        figure_with_caption: true,
        // NOTE: This option needs to be enabled or else `comrak` rejects `mathml` elements. However, the `tagfilter`
        //       extension is also enabled above, so potentially dangerous elements are escaped anyways.
        unsafe_: true,
        ..RenderOptions::default()
    }
}
