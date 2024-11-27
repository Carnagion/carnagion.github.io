use askama::Template;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Template)]
#[template(path = "404.html")]
pub struct NotFound;
