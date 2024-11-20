use askama::Template;

use jiff::Zoned;

use super::{article::Status, blog::Blog};

#[derive(Debug, Clone, Template)]
#[template(path = "atom.xml")]
pub struct Feed<'a> {
    pub blog: Blog<'a>,
}

impl<'a> Feed<'a> {
    fn last_updated(&self) -> Option<&Zoned> {
        self.blog
            .articles
            .iter()
            .filter_map(|article| match &article.meta.status {
                Status::Draft => None,
                Status::Published {
                    published,
                    updated: None,
                } => Some(published),
                Status::Published {
                    published,
                    updated: Some(updated),
                } => Some(published.max(updated)),
            })
            .max()
    }
}
