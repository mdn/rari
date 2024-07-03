use std::ops::Deref;

use scraper::{ElementRef, Html, Selector};

use crate::error::DocError;

#[derive(Debug, Clone, Copy)]
pub enum BuildSectionType {
    Prose,
    Compat,
    Specification,
    Unknown,
}

pub struct BuildSection<'a> {
    pub heading: Option<ElementRef<'a>>,
    pub body: Vec<String>,
    pub query: Option<String>,
    pub spec_urls: Option<String>,
    pub is_h3: bool,
    pub typ: BuildSectionType,
    pub id: Option<String>,
}

pub struct Splitted<'a> {
    pub sections: Vec<BuildSection<'a>>,
    pub summary: Option<String>,
    pub sidebar: Option<String>,
}

pub fn split_sections(html: &Html) -> Result<Splitted, DocError> {
    let root_children = html.root_element().children();
    let raw_sections = root_children;
    let summary_selector = Selector::parse("p").unwrap();
    let summary = html.select(&summary_selector).find_map(|s| {
        let text = s.text().collect::<String>();
        if !text.is_empty() {
            Some(text)
        } else {
            None
        }
    });
    let mut sidebar = None;

    let (mut sections, mut last) = raw_sections.fold(
        (Vec::new(), None::<BuildSection>),
        |(mut sections, mut maybe_section), current| {
            match current.value() {
                scraper::Node::Comment(comment) => {
                    let comment = format!("<!-- {} -->", comment.deref());
                    if let Some(ref mut section) = maybe_section.as_mut().and_then(|section| {
                        if !matches!(section.typ, BuildSectionType::Compat) {
                            Some(section)
                        } else {
                            None
                        }
                    }) {
                        section.body.push(comment);
                    } else {
                        if let Some(compat_section) = maybe_section.take() {
                            sections.push(compat_section);
                        }
                        let _ = maybe_section.insert(BuildSection {
                            heading: None,
                            body: vec![comment],
                            is_h3: false,
                            typ: BuildSectionType::Unknown,
                            query: None,
                            spec_urls: None,
                            id: None,
                        });
                    }
                }
                scraper::Node::Text(text) => {
                    let text = text.deref().trim_matches('\n').to_string();
                    if !text.is_empty() {
                        if let Some(ref mut section) = maybe_section {
                            section.body.push(text);
                        } else {
                            let _ = maybe_section.insert(BuildSection {
                                heading: None,
                                body: vec![text],
                                is_h3: false,
                                typ: BuildSectionType::Unknown,
                                query: None,
                                spec_urls: None,
                                id: None,
                            });
                        }
                    }
                }

                scraper::Node::Element(element) => match element.name() {
                    h @ "h2" | h @ "h3" => {
                        if let Some(section) = maybe_section.take() {
                            sections.push(section);
                        }
                        let heading = ElementRef::wrap(current).unwrap();
                        let id = heading.attr("id").map(String::from);
                        let _ = maybe_section.insert(BuildSection {
                            heading: Some(heading),
                            body: vec![],
                            is_h3: h == "h3",
                            typ: BuildSectionType::Prose,
                            query: None,
                            spec_urls: None,
                            id,
                        });
                    }
                    "section" if element.id() == Some("Quick_links") => {
                        if let Some(section) = maybe_section.take() {
                            sections.push(section);
                        }
                        let html = ElementRef::wrap(current).unwrap().inner_html();
                        sidebar = Some(html)
                    }
                    _ => {
                        let (typ, query, urls) = if element.classes().any(|cls| cls == "bc-data") {
                            (BuildSectionType::Compat, element.attr("data-query"), None)
                        } else if element.classes().any(|cls| cls == "bc-specs") {
                            (
                                BuildSectionType::Specification,
                                element.attr("data-bcd-query"),
                                element.attr("data-spec-urls"),
                            )
                        } else {
                            (BuildSectionType::Unknown, None, None)
                        };
                        match (typ, query, urls) {
                            (BuildSectionType::Compat, Some(query), _) => {
                                if let Some("true") = element.attr("data-multiple") {
                                    if let Some(section) = maybe_section.take() {
                                        sections.push(section);
                                    }
                                    sections.push(BuildSection {
                                        heading: None,
                                        body: vec![],
                                        is_h3: true,
                                        typ: BuildSectionType::Compat,
                                        query: Some(query.into()),
                                        spec_urls: None,
                                        id: None,
                                    });
                                } else if let Some(ref mut section) = maybe_section {
                                    if section.body.is_empty() {
                                        section.typ = BuildSectionType::Compat;
                                        section.query = Some(query.into());
                                    } else {
                                        // We have already something in body.
                                        // Yari does something weird so we do that to:
                                        // We push compat section and put prose after that 🤷.

                                        let heading = section.heading.take();
                                        let is_h3 = section.is_h3;
                                        let id = section.id.take();

                                        section.is_h3 = false;

                                        sections.push(BuildSection {
                                            heading,
                                            body: vec![],
                                            is_h3,
                                            typ: BuildSectionType::Compat,
                                            query: Some(query.into()),
                                            spec_urls: None,
                                            id,
                                        });
                                    }
                                }
                            }
                            (BuildSectionType::Specification, query, urls)
                                if query.is_some() || urls.is_some() =>
                            {
                                if let Some(ref mut section) = maybe_section {
                                    if section.body.is_empty() {
                                        section.typ = BuildSectionType::Specification;
                                        section.query = query.map(String::from);
                                        section.spec_urls = urls.map(String::from);
                                    } else {
                                        // We have already something in body.
                                        // Yari does something weird so we do that to:
                                        // We push compat section and put prose after that 🤷.

                                        let heading = section.heading.take();
                                        let is_h3 = section.is_h3;
                                        let id = section.id.take();

                                        section.is_h3 = false;

                                        sections.push(BuildSection {
                                            heading,
                                            body: vec![],
                                            is_h3,
                                            typ: BuildSectionType::Specification,
                                            query: query.map(String::from),
                                            spec_urls: urls.map(String::from),
                                            id,
                                        });
                                    }
                                }
                            }
                            _ => {
                                let html = ElementRef::wrap(current).unwrap().html();
                                if let Some(ref mut section) =
                                    maybe_section.as_mut().and_then(|section| {
                                        if !matches!(section.typ, BuildSectionType::Compat) {
                                            Some(section)
                                        } else {
                                            None
                                        }
                                    })
                                {
                                    section.body.push(html)
                                } else {
                                    if let Some(compat_section) = maybe_section.take() {
                                        sections.push(compat_section);
                                    }
                                    let _ = maybe_section.insert(BuildSection {
                                        heading: None,
                                        body: vec![html],
                                        is_h3: false,
                                        typ: BuildSectionType::Unknown,
                                        query: None,
                                        spec_urls: None,
                                        id: None,
                                    });
                                }
                            }
                        }
                    }
                },
                _ => {}
            }
            (sections, maybe_section)
        },
    );
    if let Some(section) = last.take() {
        sections.push(section);
    }
    Ok(Splitted {
        sections,
        summary,
        sidebar,
    })
}
