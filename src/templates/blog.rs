use std::fs;

use askama::Template;

use comrak::Arena;

use crate::markdown::Markdown;

use super::{archive, Status};

pub mod article;
use article::Article;

pub mod feed;

#[derive(Debug, Clone, Template)]
#[template(path = "blog.html")]
pub struct Blog<'a> {
    pub articles: Vec<Article<'a>>,
}

impl<'a> Blog<'a> {
    fn archive(&self) -> Vec<&Article<'a>> {
        archive(&self.articles, |article| &article.status)
    }
}
