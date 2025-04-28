use core::str;
use std::collections::HashMap;
use std::io::Write;

use comrak::create_formatter;
use comrak::html::{collect_text, render_math_code_block, render_sourcepos, write_opening_tag};
use comrak::nodes::NodeValue;
use itertools::Itertools;
use rari_types::locale::Locale;

use crate::anchor::anchorize;
use crate::ctype::isspace;
use crate::ext::DELIM_START;
use crate::node_card::{is_callout, NoteCard};
use crate::utils::{escape_href, tagfilter_block};

pub struct RariContext {
    pub stack: Vec<Option<NoteCard>>,
    pub locale: Locale,
}

create_formatter!(CustomFormatter<RariContext>, {
    NodeValue::BlockQuote => |context, node, entering| {
        context.cr()?;
        if entering {
            let note_card = is_callout(node, context.user.locale);
            match note_card {
                Some(NoteCard::Callout) => {
                    context.write_all(b"<div class=\"callout\"")?;
                    render_sourcepos(context, node)?;
                    context.write_all(b">\n")?;

                }
                Some(NoteCard::Note) => {
                    context
                        .write_all(b"<div class=\"notecard note\" data-add-note")?;
                    render_sourcepos(context, node)?;
                    context.write_all(b">\n")?;
                }
                Some(NoteCard::Warning) => {
                    context
                        .write_all(b"<div class=\"notecard warning\" data-add-warning")?;
                    render_sourcepos(context, node)?;
                    context.write_all(b">\n")?;
                }
                None => {
                    context.write_all(b"<blockquote")?;
                    render_sourcepos(context, node)?;
                    context.write_all(b">\n")?;
                }
            };
            context.user.stack.push(note_card)
        } else {
            let note_card = context.user.stack.pop();
            if note_card.unwrap_or_default().is_some() {
                context.write_all(b"</div>\n")?;
            } else {
                context.write_all(b"</blockquote>\n")?;
            }
        }
    },
    NodeValue::CodeBlock(ref ncb) => |context, node, entering| {
        if entering {
            if ncb.info.eq("math") {
                render_math_code_block(context, node, &ncb.literal)?;
            } else {
                context.cr()?;

                let mut first_tag = 0;
                let mut pre_attributes: HashMap<String, String> = HashMap::new();
                let mut code_attributes: HashMap<String, String> = HashMap::new();
                let code_attr: String;

                let literal = &ncb.literal.as_bytes();
                let info = &ncb.info.as_bytes();

                if !info.is_empty() {
                    while first_tag < info.len() && !isspace(info[first_tag]) {
                        first_tag += 1;
                    }

                    let lang_str = str::from_utf8(&info[..first_tag]).unwrap();
                    let info_str = str::from_utf8(&info[first_tag..]).unwrap().trim();

                    if context.options.render.github_pre_lang {
                        pre_attributes.insert(String::from("lang"), lang_str.to_string());

                        if context.options.render.full_info_string && !info_str.is_empty() {
                            pre_attributes.insert(
                                String::from("data-meta"),
                                info_str.trim().to_string(),
                            );
                        }
                    } else {
                        code_attr = format!("language-{}", lang_str);
                        code_attributes.insert(String::from("class"), code_attr);

                        if context.options.render.full_info_string && !info_str.is_empty() {
                            code_attributes
                                .insert(String::from("data-meta"), info_str.to_string());
                        }
                    }
                }

                if context.options.render.sourcepos {
                    let ast = node.data.borrow();
                    pre_attributes
                        .insert("data-sourcepos".to_string(), ast.sourcepos.to_string());
                }

                match context.plugins.render.codefence_syntax_highlighter {
                    None => {
                        pre_attributes.extend(code_attributes);
                        let _with_code = if let Some(cls) = pre_attributes.get_mut("class")
                        {
                            if !ncb.info.is_empty() {
                                let langs = ncb
                                    .info
                                    .split_ascii_whitespace()
                                    .map(|s| s.strip_suffix("-nolint").unwrap_or(s))
                                    .join(" ");

                                *cls = format!("brush: {langs} notranslate",);
                                &ncb.info != "plain"
                            } else {
                                *cls = "notranslate".to_string();
                                false
                            }
                        } else {
                            pre_attributes.insert("class".into(), "notranslate".into());
                            false
                        };
                        write_opening_tag(context, "pre", pre_attributes)?;
                        context.escape(literal)?;
                        context.write_all(b"</pre>\n")?
                    }
                    Some(highlighter) => {
                        highlighter.write_pre_tag(context, pre_attributes)?;
                        highlighter.write_code_tag(context, code_attributes)?;

                        highlighter.write_highlighted(
                            context,
                            str::from_utf8(&info[..first_tag]).ok(),
                            &ncb.literal,
                        )?;

                        context.write_all(b"</code></pre>\n")?
                    }
                }
            }
        }
    },
    NodeValue::Heading(ref nch) => |context, node, entering| {
        if entering {
            context.cr()?;
            write!(context, "<h{}", nch.level)?;
            if context.options.extension.header_ids.is_some() {
                let mut text_content = Vec::with_capacity(20);
               collect_text(node, &mut text_content);

                let raw_id = String::from_utf8(text_content).unwrap();
                let is_templ = raw_id.contains(DELIM_START);
                if is_templ {
                    write!(context, " data-update-id")?;
                } else {
                    let id = anchorize(&raw_id);
                    write!(context, " id=\"{}\"", id)?;
                };
            }
            render_sourcepos(context, node)?;
            context.write_all(b">")?;
        } else {
            writeln!(context, "</h{}>", nch.level)?;
        }
    },

    NodeValue::HtmlBlock(ref nhb) => |context, entering| {
        // No sourcepos.
        if entering {
            let is_marco = nhb.literal.starts_with("<!-- ks____");
            if !is_marco {
                context.cr()?;
            }
            let literal = if is_marco {
                nhb.literal
                    .strip_suffix('\n')
                    .unwrap_or(&nhb.literal)
                    .as_bytes()
            } else {
                nhb.literal.as_bytes()
            };
            if context.options.render.escape {
                context.escape(literal)?;
            } else if !context.options.render.unsafe_ {
                context.write_all(b"<!-- raw HTML omitted -->")?;
            } else if context.options.extension.tagfilter {
                tagfilter_block(literal, context)?;
            } else {
                context.write_all(literal)?;
            }
            if !is_marco {
                context.cr()?;
            }
        }
    },
    NodeValue::Link(ref nl) => |context, node, entering| {
        // Unreliable sourcepos.
        let parent_node = node.parent();

        if !context.options.parse.relaxed_autolinks
            || (parent_node.is_none()
                || !matches!(
                    parent_node.unwrap().data.borrow().value,
                    NodeValue::Link(..)
                ))
        {
            if entering {
                context.write_all(b"<a")?;
                if context.options.render.sourcepos {
                    render_sourcepos(context, node)?;
                }
                context.write_all(b" href=\"")?;
                let url = nl.url.as_bytes();
                if context.options.render.unsafe_  {
                    if let Some(rewriter) = &context.options.extension.link_url_rewriter {
                        escape_href(context, rewriter.to_html(&nl.url).as_bytes())?;
                    } else {
                        escape_href(context, url)?;
                    }
                }
                if !nl.title.is_empty() {
                    context.write_all(b"\" title=\"")?;
                    context.escape(nl.title.as_bytes())?;
                }
                let mut text_content = Vec::with_capacity(20);
                collect_text(node, &mut text_content);

                if text_content == url {
                    context.write_all(b"\" data-autolink=\"")?;
                }
                context.write_all(b"\">")?;
            } else {
                context.write_all(b"</a>")?;
            }
        }
    }
});
