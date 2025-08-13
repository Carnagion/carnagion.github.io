use askama::Template;

use jiff::Zoned;

use crate::templates::last_updated;

use super::{Blog, Status};

#[derive(Debug, Clone, Template)]
#[template(path = "blog/atom.xml")]
pub struct Feed<'a> {
    pub blog: Blog<'a>,
}

impl Feed<'_> {
    fn last_updated(&self) -> Option<Zoned> {
        last_updated(&self.blog.articles, |article| &article.status)
    }
}
