use std::cmp::Reverse;

use jiff::Zoned;

use serde::Deserialize;

pub mod index;

pub mod blog;

pub mod reviews;

pub mod not_found;

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

pub fn archive<T, F>(items: T, mut status: F) -> Vec<T::Item>
where
    T: IntoIterator,
    F: FnMut(&T::Item) -> &Status,
{
    let mut archive = items
        .into_iter()
        .filter(|item| status(item) != &Status::Draft)
        .collect::<Vec<_>>();

    archive.sort_unstable_by_key(|item| {
        Reverse(match status(item) {
            Status::Draft => None,
            Status::Published { published, .. } => Some(published).cloned(),
        })
    });

    archive
}

pub fn last_updated<T, F>(items: T, mut status: F) -> Option<Zoned>
where
    T: IntoIterator,
    F: FnMut(&T::Item) -> &Status,
{
    items
        .into_iter()
        .filter_map(|item| match status(&item) {
            Status::Draft => None,
            Status::Published {
                published,
                updated: None,
            } => Some(published).cloned(),
            Status::Published {
                published,
                updated: Some(updated),
            } => Some(published.max(updated)).cloned(),
        })
        .max()
}
