use askama::Template;

use jiff::Zoned;

use crate::templates::blog::article::Status;

use super::Reviews;

#[derive(Debug, Clone, Template)]
#[template(path = "reviews/atom.xml")]
pub struct Feed<'a> {
    pub reviews: Reviews<'a>,
}

impl Feed<'_> {
    fn last_updated(&self) -> Option<&Zoned> {
        self.reviews
            .reviews
            .iter()
            .filter_map(|review| match &review.status {
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
