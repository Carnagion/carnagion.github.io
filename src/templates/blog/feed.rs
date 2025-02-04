use askama::Template;

use jiff::Zoned;

use super::{article::Status, Blog};

#[derive(Debug, Clone, Template)]
#[template(path = "blog/atom.xml")]
pub struct Feed<'a> {
    pub blog: Blog<'a>,
}

impl Feed<'_> {
    fn last_updated(&self) -> Option<&Zoned> {
        self.blog
            .articles
            .iter()
            .filter_map(|article| match &article.status {
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
