use std::{
    io::{self, BufWriter, IntoInnerError},
    string::FromUtf8Error,
};

use comrak::{
    nodes::{AstNode, NodeHtmlBlock, NodeValue},
    Arena,
    ExtensionOptions,
    ExtensionOptionsBuilder,
    Options,
    ParseOptions,
    ParseOptionsBuilder,
    PluginsBuilder,
    RenderOptions,
    RenderOptionsBuilder,
    RenderPluginsBuilder,
};

use inkjet::theme::{self, Theme};

use latex2mathml::{latex_to_mathml, DisplayStyle, LatexError};

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

    pub fn to_html(&self) -> Result<String, ToHtmlError> {
        // NOTE: We replace LaTeX equations with MathML here rather than on construction because the AST may be modified
        //       from the outside even after construction.
        replace_mathml(self.ast)?;

        let theme = Theme::from_helix(theme::vendored::GRUVBOX).unwrap(); // PANICS: This should be a valid theme.

        let inkjet = Inkjet::new(theme);

        let render_plugins = RenderPluginsBuilder::default()
            .codefence_syntax_highlighter(Some(&inkjet))
            .build()
            .unwrap(); // PANICS: None of the fields in this builder are required.
        let plugins = PluginsBuilder::default()
            .render(render_plugins)
            .build()
            .unwrap(); // PANICS: Same as above.

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

fn extension_options() -> ExtensionOptions {
    ExtensionOptionsBuilder::default()
        .autolink(true)
        .description_lists(true)
        .footnotes(true)
        .front_matter_delimiter(Some(String::from("+++")))
        .greentext(true)
        .header_ids(Some(String::new()))
        .math_dollars(true)
        .multiline_block_quotes(true)
        .shortcodes(true)
        .spoiler(true)
        .strikethrough(true)
        .superscript(true)
        .table(true)
        .tagfilter(true) // NOTE: Needs to be enabled to escape potentially dangerous elements. See `render_options` below.
        .tasklist(true)
        .underline(true)
        .build()
        .unwrap() // PANICS: None of the fields in this builder are required.
}

fn parse_options() -> ParseOptions<'static> {
    ParseOptionsBuilder::default().smart(true).build().unwrap() // PANICS: Same as above.
}

fn render_options() -> RenderOptions {
    RenderOptionsBuilder::default()
        // NOTE: This option needs to be enabled or else `comrak` rejects `mathml` elements. However, the `tagfilter`
        //       extension is also enabled above, so potentially dangerous elements are escaped anyways.
        .unsafe_(true)
        .build()
        .unwrap() // PANICS: Same as above.
}
