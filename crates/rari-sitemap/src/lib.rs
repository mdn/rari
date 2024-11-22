use std::borrow::Cow;
use std::fmt::Write;
use std::fs::{self, File};
use std::io::{BufWriter, Write as _};
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use rari_doc::build::SitemapMeta;
use rari_types::error::EnvError;
use rari_types::globals::build_out_root;
use rari_types::locale::Locale;
use serde::Serialize;
use thiserror::Error;

pub mod ser;

use ser::prefix_base_url;

#[derive(Debug, Error)]
pub enum SitemapError {
    #[error("Error writing xml: {0}")]
    XmlSeError(#[from] quick_xml::SeError),
    #[error("Error swriting xml: {0}")]
    XmlFmtError(#[from] std::fmt::Error),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("env error: {0}")]
    EnvError(#[from] EnvError),
    #[error(transparent)]
    StripPrefixError(#[from] std::path::StripPrefixError),
}

#[derive(Serialize)]
#[serde(rename = "sitemapindex")]
pub struct SitemapIndex<'a> {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    sitemap: Vec<Url<'a>>,
}
impl<'a> SitemapIndex<'a> {
    pub fn new(urls: impl Into<Vec<Url<'a>>>) -> Self {
        Self {
            xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9",
            sitemap: urls.into(),
        }
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), SitemapError> {
        let file = File::create(path)?;
        let mut buffer = BufWriter::new(file);
        buffer.write_all(String::try_from(self)?.as_bytes())?;
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename = "urlset")]
pub struct Sitemap<'a> {
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    url: Vec<Url<'a>>,
}

#[derive(Serialize)]
pub struct Url<'a> {
    #[serde(serialize_with = "prefix_base_url")]
    loc: Cow<'a, str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lastmod: Option<NaiveDate>,
}

impl<'a> Sitemap<'a> {
    pub fn new(urls: impl Into<Vec<Url<'a>>>) -> Self {
        Sitemap {
            xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9",
            url: urls.into(),
        }
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), SitemapError> {
        let file = File::create(path)?;
        let mut buffer = BufWriter::new(file);
        buffer.write_all(String::try_from(self)?.as_bytes())?;
        Ok(())
    }

    pub fn gzip_to_path(&self, path: impl AsRef<Path>) -> Result<(), SitemapError> {
        let file = File::create(path)?;
        let buffer = BufWriter::new(file);
        let mut encoder = GzEncoder::new(buffer, Compression::default());
        encoder.write_all(String::try_from(self)?.as_bytes())?;
        Ok(())
    }
}

impl<'a, 'b: 'a> From<&'b SitemapMeta<'b>> for Url<'a> {
    fn from(value: &'b SitemapMeta<'b>) -> Self {
        Self {
            loc: Cow::Borrowed(value.url.as_ref()),
            lastmod: value.modified.map(NaiveDate::from),
        }
    }
}

impl<'a> TryFrom<&Sitemap<'a>> for String {
    type Error = SitemapError;

    fn try_from(value: &Sitemap<'a>) -> Result<Self, Self::Error> {
        let mut out = String::new();
        out.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
        quick_xml::se::to_writer(&mut out, &value)?;
        Ok(out)
    }
}
impl<'a> TryFrom<&SitemapIndex<'a>> for String {
    type Error = SitemapError;

    fn try_from(value: &SitemapIndex<'a>) -> Result<Self, Self::Error> {
        let mut out = String::new();
        out.write_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n")?;
        quick_xml::se::to_writer(&mut out, &value)?;
        Ok(out)
    }
}

pub struct Sitemaps<'a> {
    pub sitemap_meta: Vec<SitemapMeta<'a>>,
}

impl<'a> Sitemaps<'a> {
    pub fn write_sitemap_txt(&self, out_path: impl Into<PathBuf>) -> Result<PathBuf, SitemapError> {
        let mut all_urls = self
            .sitemap_meta
            .iter()
            .map(|meta| meta.url.as_ref())
            .collect::<Vec<_>>();
        all_urls.sort();
        let out_file = out_path.into().join("sitemap.txt");
        let file = File::create(&out_file)?;
        let mut buffed = BufWriter::new(file);
        for url in &all_urls {
            buffed.write_all(url.as_bytes())?;
            buffed.write_all(b"\n")?;
        }

        Ok(out_file)
    }

    pub fn write_sitemap_xml_gz(
        &self,
        out_path: impl Into<PathBuf>,
        locale: Locale,
    ) -> Result<PathBuf, SitemapError> {
        let mut locale_urls = self
            .sitemap_meta
            .iter()
            .filter(|meta| meta.locale == locale)
            .map(Url::from)
            .collect::<Vec<_>>();

        locale_urls.sort_by(|a, b| a.loc.cmp(&b.loc));

        let sitemap = Sitemap::new(locale_urls);
        let out_file = out_path.into().join("sitemap.xml.gz");
        sitemap.gzip_to_path(&out_file)?;
        Ok(out_file)
    }

    pub fn write_all_sitemaps(
        &self,
        out_path: impl Into<PathBuf>,
    ) -> Result<PathBuf, SitemapError> {
        let build_out_root = build_out_root()?;
        let out_path = out_path.into();
        self.write_sitemap_txt(&out_path)?;
        let sitemaps_out_path = out_path.join("sitemaps");
        let today = NaiveDate::from(Utc::now().naive_utc());
        let sitemaps = Locale::for_generic_and_spas()
            .iter()
            .map(|locale| {
                let out_path = sitemaps_out_path.join(locale.as_folder_str());
                fs::create_dir_all(&out_path)?;
                self.write_sitemap_xml_gz(&out_path, *locale)
                    .map_err(SitemapError::from)
                    .and_then(|path| {
                        Ok(Url {
                            loc: Cow::Owned(
                                PathBuf::from("/")
                                    .join(path.strip_prefix(build_out_root)?)
                                    .to_string_lossy()
                                    .to_string(),
                            ),
                            lastmod: Some(today),
                        })
                    })
            })
            .collect::<Result<Vec<_>, SitemapError>>()?;

        let sitemap_index = SitemapIndex::new(sitemaps);
        let out_file = out_path.join("sitemap.xml");
        sitemap_index.write_to_path(&out_file)?;
        Ok(out_file)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let urlset = Sitemap {
            xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9",
            url: vec![
                Url {
                    loc: Cow::Borrowed("foo"),
                    lastmod: Some(NaiveDate::default()),
                },
                Url {
                    loc: Cow::Borrowed("bar"),
                    lastmod: None,
                },
            ],
        };
        println!("{}", quick_xml::se::to_string(&urlset).unwrap());
    }
}
