use scraper::{ElementRef, Html, Selector};

use super::modifier::insert_attribute;
use crate::error::DocError;

pub fn bubble_up_curriculum_page(html: &mut Html) -> Result<(), DocError> {
    let mut rews = vec![];
    let co_selector = Selector::parse("p + ul").unwrap();
    for ul in html.select(&co_selector) {
        let prev_p = ul.prev_siblings().find(|s| {
            s.value()
                .as_element()
                .map(|e| e.name() == "p")
                .unwrap_or_default()
        });
        if let Some(p) = prev_p.and_then(ElementRef::wrap) {
            match p.text().last().map(str::trim) {
                Some("Learning outcomes:") => rews.push((p.id(), "curriculum-outcomes")),
                Some("Resources:" | "General resources:") => {
                    rews.push((p.id(), "curriculum-resources"));
                    let a_external_selector = Selector::parse("li > a.external").unwrap();
                    for a in ul.select(&a_external_selector) {
                        if let Some(li) = a.parent() {
                            rews.push((li.id(), "curriculum-external-li"))
                        }
                    }
                }
                _ => {}
            }
        }
    }

    let note_selector = Selector::parse("blockquote > p > strong").unwrap();
    for strong in html.select(&note_selector) {
        if matches!(
            strong.text().next(),
            Some("Notes" | "Note" | "General notes")
        ) {
            if let Some(bq) = strong.parent().and_then(|p| p.parent()) {
                rews.push((bq.id(), "curriculum-notes"))
            }
        }
    }

    //let next_ul = parent.next_siblings().find(|s| s.value().is_element());

    for (id, value) in rews {
        insert_attribute(html, id, "class", value);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_annotate() -> Result<(), DocError> {
        let html = r#"<p>foo</p><p><span class="curriculum-resources">resources:</span></p><ul><li>42</li></ul>"#;
        let mut fragment = Html::parse_fragment(html);
        bubble_up_curriculum_page(&mut fragment)?;

        assert_eq!(
            r#"<html><p>foo</p><p><span class="curriculum-resources">resources:</span></p><ul><li>42</li></ul></html>"#,
            fragment.html()
        );
        Ok(())
    }

    #[test]
    fn test_bq() -> Result<(), DocError> {
        let html = r#"<blockquote><p><strong>Notes</strong>:</p><ul><li>One key point to understand here is the difference between semantic and presentational markup, what these terms mean, and why semantic markup is important to SEO and accessibility.</li></ul></blockquote>"#;

        let mut fragment = Html::parse_fragment(html);
        bubble_up_curriculum_page(&mut fragment)?;

        assert_eq!(
            r#"<html><blockquote class="curriculum-notes"><p><strong>Notes</strong>:</p><ul><li>One key point to understand here is the difference between semantic and presentational markup, what these terms mean, and why semantic markup is important to SEO and accessibility.</li></ul></blockquote></html>"#,
            fragment.html()
        );
        Ok(())
    }
}
