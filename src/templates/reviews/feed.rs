use askama::Template;

use jiff::Zoned;

use crate::templates::{last_updated, Status};

use super::Reviews;

#[derive(Debug, Clone, Template)]
#[template(path = "reviews/atom.xml")]
pub struct Feed<'a> {
    pub reviews: Reviews<'a>,
}

impl Feed<'_> {
    fn last_updated(&self) -> Option<Zoned> {
        last_updated(&self.reviews.reviews, |review| &review.status)
    }
}
