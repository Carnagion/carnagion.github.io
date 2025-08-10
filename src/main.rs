#![deny(rust_2018_idioms)]

use std::{fs, io, path::Path};

use askama::Template;

use comrak::Arena;

use ureq::Agent;

use walkdir::WalkDir;

mod markdown;
use markdown::Markdown;

mod templates;
use templates::{
    blog::{article::Article, feed::Feed, Blog},
    index::Index,
    not_found::NotFound,
    reviews::{review::Review, Reviews},
};

fn main() -> anyhow::Result<()> {
    reset_output_dir()?;

    render_blog()?;

    render_reviews()?;

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
    fs::create_dir("docs/reviews/")?;
    fs::create_dir("docs/coverart/")?;

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
        let article = Article::from_content(md)?;

        let dst = Path::new("docs/blog/")
            .join(slug::slugify(&article.title))
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

fn render_reviews() -> anyhow::Result<()> {
    let agent = Agent::new_with_defaults();

    let arena = Arena::new();

    // Render music reviews
    let mut reviews = Vec::new();
    for entry in WalkDir::new("content/reviews/") {
        let entry = entry?;
        let src = entry.path();

        // Ignore directories and non-Markdown files
        if src.extension().is_none_or(|ext| ext != "md") {
            continue;
        }

        let content = fs::read_to_string(src)?;
        let md = Markdown::new(&content, &arena);
        let review = Review::from_content(md, &agent)?;

        let url = format!(
            "https://coverartarchive.org/release-group/{}/front",
            review.mbid,
        );

        // Get the coverart image for self-hosting
        let coverart = agent.get(url).call()?.body_mut().read_to_vec()?;
        let dst = format!("docs/coverart/{}.jpg", review.mbid);
        fs::write(dst, coverart)?;

        let dst = Path::new("docs/reviews/")
            .join(format!(
                "{}-{}",
                slug::slugify(&review.title),
                slug::slugify(&review.artists),
            ))
            .with_extension("html");
        fs::write(dst, review.render()?)?;

        reviews.push(review);
    }

    // Render reviews page
    let reviews = Reviews { reviews };
    fs::write("docs/reviews.html", reviews.render()?)?;

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
