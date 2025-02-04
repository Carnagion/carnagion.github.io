#![deny(rust_2018_idioms)]

use std::{fs, io, path::Path};

use askama::Template;

use comrak::Arena;

use walkdir::WalkDir;

mod markdown;
use markdown::Markdown;

mod templates;
use templates::{
    blog::{
        article::{Article, Status},
        feed::Feed,
        Blog,
    },
    index::Index,
    not_found::NotFound,
};

fn main() -> anyhow::Result<()> {
    reset_output_dir()?;

    render_blog()?;

    // Render landing page
    let index = Index;
    fs::write("docs/index.html", index.render()?)?;

    // Render 404 page
    let not_found = NotFound;
    fs::write("docs/404.html", not_found.render()?)?;

    copy_assets()?;

    Ok(())
}

fn reset_output_dir() -> anyhow::Result<()> {
    // Remove the output directory and its contents, ignoring errors if it did not exist
    match fs::remove_dir_all("docs/") {
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }?;

    // Re-create the output directory and necessary sub-directories
    fs::create_dir("docs/")?;
    fs::create_dir("docs/blog/")?;

    Ok(())
}

fn render_blog() -> anyhow::Result<()> {
    let arena = Arena::new();

    // Render blog posts
    let mut articles = Vec::new();
    for entry in WalkDir::new("content/blog/") {
        let entry = entry?;
        let src = entry.path();

        // Ignore directories and non-Markdown files
        if src.extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        let content = fs::read_to_string(src)?;
        let md = Markdown::new(&content, &arena);
        let article = Article::new(md)?;

        // Ignore drafts
        if article.meta.status == Status::Draft {
            continue;
        }

        let dst = Path::new("docs/blog/")
            .join(slug::slugify(&article.meta.title))
            .with_extension("html");
        fs::write(dst, article.render()?)?;

        articles.push(article);
    }

    // Render blog page
    let blog = Blog { articles };
    fs::write("docs/blog.html", blog.render()?)?;

    // Render Atom feed
    let feed = Feed { blog };
    fs::write("docs/blog/atom.xml", feed.render()?)?;

    Ok(())
}

fn copy_assets() -> anyhow::Result<()> {
    // Copy over asset files
    for entry in WalkDir::new("assets/") {
        let entry = entry?;

        if !entry.file_type().is_file() {
            continue;
        }

        let src = entry.path();
        let dst = Path::new("docs/").join(src.strip_prefix("assets/")?);
        fs::copy(src, dst)?;
    }

    Ok(())
}
