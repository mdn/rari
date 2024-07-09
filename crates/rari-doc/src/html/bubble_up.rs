use scraper::{Html, Selector};

use super::modifier::add_attribute;
use crate::error::DocError;

pub fn bubble_up_curriculum_page(html: &mut Html) -> Result<(), DocError> {
    let co_selector = Selector::parse("span.curriculum-outcomes").unwrap();
    let mut rews = vec![];
    for ul in html.select(&co_selector) {
        if let Some(parent) = ul.parent() {
            rews.push((parent.id(), "curriculum-outcomes"))
        }
    }
    let co_selector = Selector::parse("span.curriculum-resources").unwrap();
    for ul in html.select(&co_selector) {
        if let Some(parent) = ul.parent() {
            rews.push((parent.id(), "curriculum-resources"))
        }
    }
    let co_selector = Selector::parse("li > a.external").unwrap();
    for ul in html.select(&co_selector) {
        if let Some(parent) = ul.parent() {
            rews.push((parent.id(), "curriculum-external-li"))
        }
    }
    for (id, value) in rews {
        add_attribute(html, id, "class", value);
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
            r#"<html><p>foo</p><p class="curriculum-resources"><span class="curriculum-resources">resources:</span></p><ul><li>42</li></ul></html>"#,
            fragment.html()
        );
        Ok(())
    }
}
