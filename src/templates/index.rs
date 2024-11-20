use std::fs;

use askama::Template;

use comrak::Arena;

use crate::markdown::Markdown;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Template)]
#[template(path = "index.html")]
pub struct Index;
