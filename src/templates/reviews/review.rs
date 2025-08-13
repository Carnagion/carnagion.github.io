use askama::Template;

use serde::Deserialize;

use thiserror::Error;

use ureq::Agent;

use uuid::Uuid;

use crate::{markdown::Markdown, templates::Status};

#[derive(Debug, Clone, Template)]
#[template(path = "reviews/review.html")]
pub struct Review<'a> {
    pub content: Markdown<'a>,
    pub mbid: Uuid,
    pub title: String,
    pub artists: String,
    pub rating: u8,
    pub status: Status,
}

impl<'a> Review<'a> {
    pub fn from_content(content: Markdown<'a>, agent: &Agent) -> Result<Self, FromMarkdownError> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields, rename_all = "kebab-case")]
        struct FrontMatter {
            mbid: Uuid,
            rating: u8,
            status: Status,
        }

        let meta = content.extract_front_matter::<FrontMatter>()?;

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct ReleaseGroup {
            title: String,
            artist_credit: Vec<ArtistCredit>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "kebab-case")]
        struct ArtistCredit {
            name: String,
            joinphrase: String,
        }

        // NOTE: See https://musicbrainz.org/doc/MusicBrainz_API for details on how the MusicBrainz API can
        //       be queried.
        let url = format!(
            "https://musicbrainz.org/ws/2/release-group/{}?inc=artists&fmt=json",
            meta.mbid,
        );

        let release_group = agent
            .get(url)
            .call()?
            .body_mut()
            .read_json::<ReleaseGroup>()?;

        let artists = release_group
            .artist_credit
            .into_iter()
            .flat_map(|artist| [artist.name, artist.joinphrase])
            .collect();

        Ok(Self {
            content,
            mbid: meta.mbid,
            title: release_group.title,
            artists,
            rating: meta.rating,
            status: meta.status,
        })
    }
}

#[derive(Debug, Error)]
pub enum FromMarkdownError {
    #[error("could not extract review metadata: {0}")]
    Meta(#[from] toml::de::Error),
    #[error("could not query MusicBrainz database: {0}")]
    Request(#[from] ureq::Error),
}
