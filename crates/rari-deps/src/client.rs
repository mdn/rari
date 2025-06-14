use reqwest::blocking::Response;
use url::{Url, Host};

pub fn get(url: impl AsRef<str>) -> reqwest::Result<Response> {
    let mut client = reqwest::blocking::ClientBuilder::new().user_agent("mdn/rari");

    // check if the URL's host is api.github.com
    if is_github_url(url.as_ref()) {
        // get the GitHub token from the environment
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            client = client
                .default_headers(
                    reqwest::header::HeaderMap::from_iter(vec![(
                        reqwest::header::AUTHORIZATION,
                        format!("Bearer {}", token).parse().unwrap(),
                    )]),
                );
        }
    }

    client.build()?
        .get(url.as_ref())
        .send()
}

fn is_github_url(url: impl AsRef<str>) -> bool {
    let url = Url::parse(url.as_ref());
    if let Ok(url) = url {
        match url.host() {
            Some(Host::Domain(host)) => {
                host == "api.github.com"
            }
            _ => false,
        }
    } else {
        false
    }
}
