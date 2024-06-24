use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PrevNextBlog {
    pub previous: Option<SlugNTitle>,
    pub next: Option<SlugNTitle>,
}

impl PrevNextBlog {
    pub fn is_none(&self) -> bool {
        self.previous.is_none() && self.next.is_none()
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct SlugNTitle {
    pub title: String,
    pub slug: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
#[serde(default)]
pub struct PrevNextCurriculum {
    pub prev: Option<UrlNTitle>,
    pub next: Option<UrlNTitle>,
}
#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct UrlNTitle {
    pub title: String,
    pub url: String,
}
