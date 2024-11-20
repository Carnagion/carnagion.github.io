use askama::Template;

use comrak::nodes::NodeValue;

use jiff::Zoned;

use serde::{de::Error, Deserialize};

use crate::markdown::Markdown;

#[derive(Debug, Clone, Template)]
#[template(path = "article.html")]
pub struct Article<'a> {
    md: Markdown<'a>,
    pub meta: Meta,
}

impl<'a> Article<'a> {
    pub fn new(md: Markdown<'a>) -> Result<Self, toml::de::Error> {
        let meta = md
            .ast
            .descendants()
            .find_map(|node| {
                let node = &node.data.borrow().value;
                let front_matter = match node {
                    NodeValue::FrontMatter(front_matter) => Some(front_matter),
                    _ => None,
                }?;
                let front_matter =
                    front_matter.trim_matches(|c: char| c.is_whitespace() || c == '+');
                Some(toml::from_str(front_matter))
            })
            .unwrap_or_else(|| Err(toml::de::Error::custom("no front matter found")))?;

        Ok(Self { md, meta })
    }

    pub(crate) fn md(&self) -> &Markdown<'a> {
        &self.md
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Meta {
    pub title: String,
    pub description: String,
    pub status: Status,
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
