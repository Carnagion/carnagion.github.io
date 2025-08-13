use askama::Template;

use serde::Deserialize;

use thiserror::Error;

use crate::{markdown::Markdown, templates::Status};

#[derive(Debug, Clone, Template)]
#[template(path = "blog/article.html")]
pub struct Article<'a> {
    pub content: Markdown<'a>,
    pub title: String,
    pub description: String,
    pub status: Status,
}

impl<'a> Article<'a> {
    pub fn from_content(content: Markdown<'a>) -> Result<Self, FromMarkdownError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "kebab-case")]
        struct FrontMatter {
            title: String,
            description: String,
            status: Status,
        }

        let meta = content.extract_front_matter::<FrontMatter>()?;
        Ok(Self {
            content,
            title: meta.title,
            description: meta.description,
            status: meta.status,
        })
    }
}

#[derive(Debug, Error)]
pub enum FromMarkdownError {
    #[error("could not extract article metadata: {0}")]
    Meta(#[from] toml::de::Error),
}
