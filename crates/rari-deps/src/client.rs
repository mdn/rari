use reqwest::blocking::Response;
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
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            req_builder = req_builder.bearer_auth(token);
        }
    }

    Ok(req_builder.send()?)
}
