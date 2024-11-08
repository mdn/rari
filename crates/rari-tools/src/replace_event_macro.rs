use std::collections::HashMap;
use std::{fs, path};

use rari_doc::pages::page::{Page, PageLike, PageWriter};
use rari_types::globals::content_translated_root;
use rari_types::locale::Locale;
use rari_utils::concat_strs;
use regex::Regex;

use crate::error::ToolError;
use crate::utils::{get_redirects_map, read_all_doc_pages, read_files_parallel};

pub fn replace_event_macro(locale: Locale) -> Result<(), ToolError> {
    let docs = read_all_doc_pages()?;
    let redirects_maps: HashMap<Locale, HashMap<String, String>> = [locale]
        .iter()
        .chain(std::iter::once(&Locale::EnUs))
        .map(|locale| {
            (
                *locale,
                get_redirects_map(*locale)
                    .iter()
                    .map(|(k, v)| (k.to_lowercase(), v.to_string()))
                    .collect(),
            )
        })
        .collect();

    let translated_root = content_translated_root()
        .expect("translated root not set")
        .to_str()
        .unwrap();

    let locale_root = concat_strs!(
        translated_root,
        &path::MAIN_SEPARATOR_STR,
        locale.as_folder_str()
    );

    let files = read_files_parallel(&[locale_root])?;
    let files = files
        .iter()
        .filter(|(p, _)| {
            let x = p.replace(translated_root, "");
            let x = x.trim_start_matches(path::MAIN_SEPARATOR).to_lowercase();
            x.starts_with(locale.as_folder_str())
        })
        .collect::<Vec<_>>();

    let re = Regex::new(r"\{\{event([^}]*)\}\}").unwrap();

    files.iter().for_each(|(path, content)| {
        let result = re.replace_all(content, |caps: &regex::Captures| {
            let args = caps.get(1).unwrap().as_str();
            // println!("args: {}", args);
            process_event_macro(args)
            // format!("EVENTMACRO MATCHED WITH ARGS {}", args)
        });
        if result != *content {
            println!("NEW CONTENT for {}", path);
            fs::write(path, &*result).expect("could not write file");
        }
    });

    Ok(())
}

fn process_event_macro(args: &str) -> String {
    let args = args
        .trim()
        .trim_end_matches(')')
        .trim_start_matches('(')
        .split(',')
        .map(|a| {
            a.trim()
                .trim_start_matches('\'')
                .trim_end_matches('\'')
                .trim_start_matches('"')
                .trim_end_matches('"')
        })
        .collect::<Vec<_>>();
    format!("EVENTMACRO MATCHED WITH ARGS {:?}", args)
}
