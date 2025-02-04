use askama::Template;

use comrak::nodes::NodeValue;

use jiff::Zoned;

use serde::Deserialize;

use thiserror::Error;

use crate::markdown::Markdown;

#[derive(Debug, Clone, Template)]
#[template(path = "blog/article.html")]
pub struct Article<'a> {
    pub md: Markdown<'a>,
    pub title: String,
    pub description: String,
    pub status: Status,
}

impl<'a> Article<'a> {
    pub fn new(md: Markdown<'a>) -> Result<Self, FromMarkdownError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "kebab-case")]
        struct FrontMatter {
            title: String,
            description: String,
            status: Status,
        }

        let meta = md.extract_front_matter::<FrontMatter>()?;
        Ok(Self {
            md,
            title: meta.title,
            description: meta.description,
            status: meta.status,
        })
    }

    fn reading_time(&self) -> (usize, usize) {
        // NOTE: These statistics are taken from Firefox's source code, which uses them for its reader view.
        //       See https://searchfox.org/mozilla-central/source/mobile/android/android-components/components/feature/readerview/src/main/assets/extensions/readerview/readerview.js#221.
        let cpm = 987;
        let variance = 118;

        let chars = self
            .md
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

#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Status {
    #[default]
    Draft,
    #[serde(untagged)]
    Published {
        published: Zoned,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated: Option<Zoned>,
    },
}

#[derive(Debug, Error)]
pub enum FromMarkdownError {
    #[error("could not extract article metadata: {0}")]
    Meta(#[from] toml::de::Error),
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
