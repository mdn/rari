use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use comrak::create_formatter;
use comrak::html::{collect_text, render_math_code_block, render_sourcepos, write_opening_tag};
use comrak::nodes::NodeValue;
use itertools::Itertools;
use rari_types::locale::Locale;

use crate::anchor::anchorize;
use crate::ctype::isspace;
use crate::ext::DELIM_START;
use crate::node_card::{NoteCard, is_callout};
use crate::utils::{escape_href, tagfilter_block};

#[derive(Default)]
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
                    context.write_str("<div class=\"callout\"")?;
                    render_sourcepos(context, node)?;
                    context.write_str(">\n")?;

                }
                Some(NoteCard::Note) => {
                    context
                        .write_str("<div class=\"notecard note\" data-add-note")?;
                    render_sourcepos(context, node)?;
                    context.write_str(">\n")?;
                }
                Some(NoteCard::Warning) => {
                    context
                        .write_str("<div class=\"notecard warning\" data-add-warning")?;
                    render_sourcepos(context, node)?;
                    context.write_str(">\n")?;
                }
                None => {
                    context.write_str("<blockquote")?;
                    render_sourcepos(context, node)?;
                    context.write_str(">\n")?;
                }
            };
            context.user.stack.push(note_card)
        } else {
            let note_card = context.user.stack.pop();
            if note_card.unwrap_or_default().is_some() {
                context.write_str("</div>\n")?;
            } else {
                context.write_str("</blockquote>\n")?;
            }
        }
    },
    NodeValue::CodeBlock(ref ncb) => |context, node, entering| {
        if entering {
            if ncb.info.eq("math") {
                return render_math_code_block(context, node, &ncb.literal);
            } else {
                context.cr()?;

                let mut first_tag = 0;
                let mut pre_attributes: HashMap<&'static str, Cow<str>> = HashMap::new();
                let mut code_attributes: HashMap<&'static str, Cow<str>> = HashMap::new();

                let info = ncb.info.as_bytes();

                if !info.is_empty() {
                    while first_tag < info.len() && !isspace(info[first_tag]) {
                        first_tag += 1;
                    }

                    let lang_str = &ncb.info[..first_tag];
                    let info_str = ncb.info[first_tag..].trim();

                    if context.options.render.github_pre_lang {
                        pre_attributes.insert("lang", lang_str.to_string().into());

                        if context.options.render.full_info_string && !info_str.is_empty() {
                            pre_attributes.insert(
                                "data-meta",
                                info_str.trim().to_string().into(),
                            );
                        }
                    } else {
                        let code_attr = format!("language-{lang_str}");
                        code_attributes.insert("class", code_attr.into());

                        if context.options.render.full_info_string && !info_str.is_empty() {
                            code_attributes
                                .insert("data-meta", info_str.to_string().into());
                        }
                    }
                }

                if context.options.render.sourcepos {
                    let ast = node.data.borrow();
                    pre_attributes
                        .insert("data-sourcepos", ast.sourcepos.to_string().into());
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

                                *cls = format!("brush: {langs} notranslate").into();
                                &*ncb.info != "plain"
                            } else {
                                *cls = "notranslate".into();
                                false
                            }
                        } else {
                            pre_attributes.insert("class", "notranslate".into());
                            false
                        };
                        write_opening_tag(context, "pre", pre_attributes)?;
                        context.escape(&ncb.literal)?;
                        context.write_str("</pre>\n")?
                    }
                    Some(highlighter) => {
                        highlighter.write_pre_tag(context, pre_attributes)?;
                        highlighter.write_code_tag(context, code_attributes)?;

                        highlighter.write_highlighted(
                            context,
                            Some(&ncb.info[..first_tag]).filter(|s| !s.is_empty()),
                            &ncb.literal,
                        )?;

                        context.write_str("</code></pre>\n")?
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
                let raw_id = collect_text(node);

                let is_templ = raw_id.contains(DELIM_START);
                if is_templ {
                    write!(context, " data-update-id")?;
                } else {
                    let id = anchorize(&raw_id);
                    write!(context, " id=\"{id}\"")?;
                };
            }
            render_sourcepos(context, node)?;
            context.write_str(">")?;
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
            } else {
                &nhb.literal
            };
            if context.options.render.escape {
                context.escape(literal)?;
            } else if !context.options.render.r#unsafe {
                context.write_str("<!-- raw HTML omitted -->")?;
            } else if context.options.extension.tagfilter {
                tagfilter_block(literal, context)?;
            } else {
                context.write_str(literal)?;
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
                context.write_str("<a")?;
                if context.options.render.sourcepos {
                    render_sourcepos(context, node)?;
                }
                context.write_str(" href=\"")?;
                if context.options.render.r#unsafe {
                    if let Some(rewriter) = &context.options.extension.link_url_rewriter {
                        escape_href(context, &rewriter.to_html(&nl.url))?;
                    } else {
                        escape_href(context, &nl.url)?;
                    }
                }
                if !nl.title.is_empty() {
                    context.write_str("\" title=\"")?;
                    context.escape(&nl.title)?;
                }
                let text_content = collect_text(node);

                if text_content == *nl.url {
                    context.write_str("\" data-autolink=\"")?;
                }
                context.write_str("\">")?;
            } else {
                context.write_str("</a>")?;
            }
        }
    }
});
