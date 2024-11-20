use std::{collections::HashMap, fmt, io};

use comrak::adapters::SyntaxHighlighterAdapter;

use inkjet::{
    formatter::{Formatter, ThemedHtml},
    theme::Theme,
    tree_sitter_highlight::HighlightEvent,
    Highlighter,
    Language,
};

#[derive(Debug, Clone)]
pub struct Inkjet(Theme);

impl Inkjet {
    pub fn new(theme: Theme) -> Self {
        Self(theme)
    }
}

// NOTE: `inkjet`'s `ThemedHtml` formatter only emits `pre` tags without `code` tags inside them. Semantic HTML however
//       (rightfully) demands the usage of `code` for representing code blocks. As such, I created this formatter that writes
//       both `pre` and `code` before delegating the rest of the formatting to `ThemedHtml`.
impl Formatter for Inkjet {
    fn write<W>(&self, source: &str, writer: &mut W, event: HighlightEvent) -> inkjet::Result<()>
    where
        W: fmt::Write,
    {
        let formatter = ThemedHtml::new(self.0.clone());
        formatter.write(source, writer, event)
    }

    fn start<W>(&self, _source: &str, writer: &mut W) -> inkjet::Result<()>
    where
        W: fmt::Write,
    {
        write!(
            writer,
            r#"<pre style="background-color: {}; color: {};"><code>"#,
            self.0.bg.into_hex(),
            self.0.fg.into_hex(),
        )?;
        Ok(())
    }

    fn finish<W>(&self, _source: &str, writer: &mut W) -> inkjet::Result<()>
    where
        W: fmt::Write,
    {
        write!(writer, "</code></pre>")?;
        Ok(())
    }
}

impl SyntaxHighlighterAdapter for Inkjet {
    fn write_highlighted(
        &self,
        mut output: &mut dyn io::Write,
        lang: Option<&str>,
        code: &str,
    ) -> io::Result<()> {
        let lang = lang
            .and_then(Language::from_token)
            .unwrap_or(Language::Plaintext);

        let mut highlighter = Highlighter::new();
        highlighter
            .highlight_to_writer(lang, self, code, &mut output)
            .map_err(io::Error::other)?;

        Ok(())
    }

    fn write_pre_tag(
        &self,
        _output: &mut dyn io::Write,
        _attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        Ok(())
    }

    fn write_code_tag(
        &self,
        _output: &mut dyn io::Write,
        _attributes: HashMap<String, String>,
    ) -> io::Result<()> {
        Ok(())
    }
}
