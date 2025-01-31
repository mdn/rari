use reqwest::blocking::Response;

pub fn get(url: impl AsRef<str>) -> reqwest::Result<Response> {
    reqwest::blocking::ClientBuilder::new()
        .user_agent("mdn/rari")
        .build()?
        .get(url.as_ref())
        .send()
}
