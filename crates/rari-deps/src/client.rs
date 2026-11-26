use std::error::Error as _;
use std::path::Path;
use std::sync::LazyLock;
use std::{env, fs};

use reqwest::Certificate;
use reqwest::blocking::{Client, ClientBuilder, Response};
use tracing::warn;
use url::Url;

use crate::error::DepsError;

const SSL_CERT_FILE: &str = "SSL_CERT_FILE";
const SSL_CERT_DIR: &str = "SSL_CERT_DIR";

static CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    let extra_roots = env::var_os(SSL_CERT_FILE)
        .filter(|path| !path.is_empty())
        .map(|path| load_certificates(Path::new(&path)))
        .unwrap_or_default();
    if !extra_roots.is_empty() {
        match build_client(&extra_roots) {
            Ok(client) => return Ok(client),
            Err(err) => warn!(
                "ignoring {SSL_CERT_FILE}: the client could not be built with its certificates: {err}"
            ),
        }
    }
    build_client(&[]).map_err(|err| err.to_string())
});

fn build_client(extra_roots: &[Certificate]) -> Result<Client, reqwest::Error> {
    let mut builder = ClientBuilder::new().user_agent("mdn/rari");
    for cert in extra_roots {
        builder = builder.add_root_certificate(cert.clone());
    }
    builder.build()
}

/// Reads a PEM bundle, warning (not failing) on problems so a stale path cannot break downloads that work without it.
fn load_certificates(path: &Path) -> Vec<Certificate> {
    let pem = match fs::read(path) {
        Ok(pem) => pem,
        Err(err) => {
            warn!("ignoring {SSL_CERT_FILE} ({}): {err}", path.display());
            return Vec::new();
        }
    };
    match Certificate::from_pem_bundle(&pem) {
        Ok(certs) if certs.is_empty() => {
            warn!(
                "ignoring {SSL_CERT_FILE} ({}): no PEM certificates found",
                path.display()
            );
            Vec::new()
        }
        Ok(certs) => certs,
        Err(err) => {
            warn!("ignoring {SSL_CERT_FILE} ({}): {err}", path.display());
            Vec::new()
        }
    }
}

fn is_unknown_issuer(err: &reqwest::Error) -> bool {
    let mut source = err.source();
    while let Some(cause) = source {
        if cause.to_string().contains("UnknownIssuer") {
            return true;
        }
        source = cause.source();
    }
    false
}

fn unknown_issuer_hint(cert_dir_set: bool) -> String {
    let mut hint = format!(
        "hint: if a proxy intercepts HTTPS traffic, set {SSL_CERT_FILE} to a PEM bundle containing its CA certificate"
    );
    if cert_dir_set {
        hint.push_str(&format!(" ({SSL_CERT_DIR} is set but not supported)"));
    }
    hint
}

pub fn get(url: impl AsRef<str>) -> Result<Response, DepsError> {
    let url = Url::parse(url.as_ref())?;
    let client = CLIENT
        .as_ref()
        .map_err(|err| DepsError::ClientError(err.clone()))?;
    let mut req_builder = client.get(url.as_ref());

    // check if the URL's host is api.github.com
    if url.host_str() == Some("api.github.com") {
        // Use GH_TOKEN or GITHUB_TOKEN if set to avoid rate limiting.
        if let Ok(token) = env::var("GH_TOKEN").or_else(|_| env::var("GITHUB_TOKEN")) {
            req_builder = req_builder.bearer_auth(token);
        } else if env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
            eprintln!(
                "::warning::Cannot authenticate GitHub API request. (Provide GH_TOKEN or GITHUB_TOKEN to get a higher rate limit.)"
            );
        }
    }

    req_builder.send().map_err(|source| {
        if is_unknown_issuer(&source) {
            DepsError::UnknownIssuer {
                hint: unknown_issuer_hint(env::var_os(SSL_CERT_DIR).is_some()),
                source,
            }
        } else {
            DepsError::FetchError(source)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CERT: &str = include_str!("../tests/fixtures/test-ca.pem");

    #[test]
    fn load_certificates_cases() {
        struct Case {
            name: &'static str,
            content: Option<String>,
            expected: usize,
        }
        let cases = [
            Case {
                name: "single cert",
                content: Some(CERT.into()),
                expected: 1,
            },
            Case {
                name: "two certs",
                content: Some(format!("{CERT}\n{CERT}")),
                expected: 2,
            },
            Case {
                name: "empty file",
                content: Some(String::new()),
                expected: 0,
            },
            Case {
                name: "garbage",
                content: Some("not a certificate".into()),
                expected: 0,
            },
            Case {
                name: "missing file",
                content: None,
                expected: 0,
            },
        ];
        let dir = tempfile::tempdir().unwrap();
        for case in cases {
            let path = dir.path().join("bundle.pem");
            let _ = fs::remove_file(&path);
            if let Some(content) = &case.content {
                fs::write(&path, content).unwrap();
            }
            assert_eq!(
                load_certificates(&path).len(),
                case.expected,
                "{}",
                case.name
            );
        }
    }

    #[test]
    fn client_builds_with_extra_root() {
        let certs = Certificate::from_pem_bundle(CERT.as_bytes()).unwrap();
        assert!(build_client(&certs).is_ok());
    }

    #[test]
    fn hint_mentions_cert_dir_only_when_set() {
        assert!(!unknown_issuer_hint(false).contains(SSL_CERT_DIR));
        assert!(unknown_issuer_hint(true).contains("not supported"));
    }
}
