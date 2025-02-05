use std::{cmp::Reverse, fs};

use askama::Template;

use comrak::Arena;

use crate::markdown::Markdown;

pub mod article;
use article::{Article, Status};

pub mod feed;

#[derive(Debug, Clone, Template)]
#[template(path = "blog.html")]
pub struct Blog<'a> {
    pub articles: Vec<Article<'a>>,
}

impl<'a> Blog<'a> {
    fn archive(&self) -> Vec<Article<'a>> {
        let mut articles = self
            .articles
            .iter()
            .filter(|article| article.status != Status::Draft)
            .cloned()
            .collect::<Vec<_>>();

        articles.sort_unstable_by_key(|article| {
            Reverse(match &article.status {
                Status::Draft => None,
                Status::Published { published, .. } => Some(published).cloned(),
            })
        });

        articles
    }
}
