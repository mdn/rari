use std::fs;
use std::path::Path;

use rari_doc::error::DocError;
use rari_doc::html::sidebar::build_sidebar;
use rari_doc::pages::page::{Page, PageBuilder, PageReader};
use rari_doc::pages::types::doc::Doc;
use rari_doc::pages::types::utils::FmTempl;
use rari_doc::reader::read_docs_parallel;
use rari_doc::templ::templs::invoke;
use rari_types::globals::SETTINGS;
use rari_types::settings::Settings;
use rari_types::{Arg, Quotes, RariEnv};

struct Case {
    name: &'static str,
    tree: &'static str,
    valid_slug: &'static str,
    argument: &'static str,
    recovered_slug: &'static str,
    recovered_arg: &'static str,
}

fn write_doc(root: &Path, folder: &str, slug: &str) {
    let path = root.join("en-us").join(folder).join("index.md");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("---\ntitle: Example\nslug: {slug}\n---\n")).unwrap();
}

#[test]
fn reference_macros_survive_invalid_pages_and_retry_failed_initialization() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("en-us")).unwrap();
    fs::write(root.path().join("en-us/_redirects.txt"), "").unwrap();
    SETTINGS
        .set(Settings {
            content_root: root.path().to_path_buf(),
            reader_ignores_gitignore: true,
            ..Default::default()
        })
        .unwrap();
    let env = RariEnv::default();

    for Case {
        name,
        tree,
        valid_slug,
        argument,
        recovered_slug,
        recovered_arg,
    } in [
        Case {
            name: "domxref",
            tree: "Web/API",
            valid_slug: "Web/API/Example_API/Reference/Example",
            argument: "Example",
            recovered_slug: "Web/API/Example_API/Reference/Recovered",
            recovered_arg: "Recovered",
        },
        Case {
            name: "cssxref",
            tree: "Web/CSS/Reference",
            valid_slug: "Web/CSS/Reference/Properties/example",
            argument: "example",
            recovered_slug: "Web/CSS/Reference/Properties/recovered",
            recovered_arg: "recovered",
        },
        Case {
            name: "jsxref",
            tree: "Web/JavaScript/Reference",
            valid_slug: "Web/JavaScript/Reference/Global_Objects/Example",
            argument: "Example",
            recovered_slug: "Web/JavaScript/Reference/Global_Objects/Recovered",
            recovered_arg: "Recovered",
        },
    ] {
        let folder = tree.to_lowercase();
        write_doc(root.path(), &folder, "Wrong");
        write_doc(root.path(), &valid_slug.to_lowercase(), valid_slug);
        let broken_folder = recovered_slug.to_lowercase();
        write_doc(root.path(), &broken_folder, &format!("{tree}/Copied"));
        let render = || {
            invoke(
                &env,
                name,
                vec![Some(Arg::String(argument.into(), Quotes::Double))],
            )
            .map(|(html, _)| html)
        };

        assert!(matches!(render(), Err(DocError::ReferenceIndexError(_))));
        write_doc(root.path(), &folder, tree);
        assert!(matches!(render(), Err(DocError::ReferenceIndexError(_))));
        let page_path = root.path().join("en-us").join(name).join("index.md");
        fs::create_dir_all(page_path.parent().unwrap()).unwrap();
        fs::write(
            &page_path,
            format!("---\ntitle: Example\nslug: {name}\n---\n\n{{{{{name}(\"{argument}\")}}}}\n"),
        )
        .unwrap();
        let page = Page::read(&page_path, None).unwrap();
        let error = page
            .build()
            .expect_err("a malformed reference page must fail the build");
        assert!(
            error
                .to_string()
                .contains("doesn't match its disk folder name")
        );
        let Page::Doc(doc) = &page else {
            unreachable!()
        };
        let sidebar = FmTempl::WithArgs {
            name: name.to_string(),
            args: vec![argument.to_string()],
        };
        assert!(
            build_sidebar(&sidebar, doc).is_err(),
            "{name}: sidebar errors must fail builds too"
        );

        let broken_path = root
            .path()
            .join("en-us")
            .join(&broken_folder)
            .join("index.md");
        assert!(matches!(
            Doc::read(&broken_path, None),
            Err(DocError::SlugFolderMismatch(..))
        ));
        assert!(matches!(
            read_docs_parallel::<Page, Doc>(&[&broken_path], None),
            Err(DocError::DocsReadError)
        ));

        write_doc(root.path(), &broken_folder, recovered_slug);
        page.build()
            .expect("correcting the child must allow building without restarting");
        let html = render().unwrap();
        assert!(
            html.contains(&format!("href=\"/en-US/docs/{valid_slug}\"")),
            "{name}: {html}"
        );
        let (html, _) = invoke(
            &env,
            name,
            vec![Some(Arg::String(recovered_arg.into(), Quotes::Double))],
        )
        .unwrap();
        assert!(
            html.contains(&format!("href=\"/en-US/docs/{recovered_slug}\"")),
            "{name}: {html}"
        );
    }
}
