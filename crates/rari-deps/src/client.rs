use reqwest::blocking::Response;
use std::env;
use url::Url;

use crate::error::DepsError;

pub fn get(url: impl AsRef<str>) -> Result<Response, DepsError> {
    let url = Url::parse(url.as_ref())?;
    let mut req_builder = reqwest::blocking::ClientBuilder::new()
        .user_agent("mdn/rari")
        .build()?
        .get(url.as_ref());

    // check if the URL's host is api.github.com
    if url.host_str() == Some("api.github.com") {
        // get the GitHub token from the environment
        if let Ok(token) = env::var("GITHUB_TOKEN") {
            req_builder = req_builder.bearer_auth(token);
        } else if env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
            eprintln!(
                "::warning::Cannot authenticate GitHub API request. (Provide GITHUB_TOKEN to get a higher rate limit.)"
            );
        }
    }

    Ok(req_builder.send()?)
}
