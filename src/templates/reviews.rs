use std::{cmp::Reverse, fs};

use askama::Template;

use comrak::Arena;

use crate::Markdown;

use super::blog::article::Status;

pub mod review;
use review::Review;

#[derive(Debug, Clone, Template)]
#[template(path = "reviews.html")]
pub struct Reviews<'a> {
    pub reviews: Vec<Review<'a>>,
}

impl<'a> Reviews<'a> {
    fn archive(&self) -> Vec<Review<'a>> {
        let mut reviews = self
            .reviews
            .iter()
            .filter(|review| review.status != Status::Draft)
            .cloned()
            .collect::<Vec<_>>();

        reviews.sort_unstable_by_key(|review| {
            Reverse(match &review.status {
                Status::Draft => None,
                Status::Published { published, .. } => Some(published).cloned(),
            })
        });

        reviews
    }
}
