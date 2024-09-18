use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, RwLock};
use std::thread::spawn;

use anyhow::{anyhow, Error};
use clap::{Args, Parser, Subcommand};
use rari_doc::build::{
    build_blog_pages, build_contributor_spotlight_pages, build_curriculum_pages, build_docs,
    build_generic_pages, build_spas,
};
use rari_doc::cached_readers::{read_and_cache_doc_pages, CACHED_DOC_PAGE_FILES};
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_doc::search_index::build_search_index;
use rari_doc::utils::TEMPL_RECORDER_SENDER;
use rari_tools::history::gather_history;
use rari_tools::popularities::update_popularities;
use rari_types::globals::{build_out_root, content_root, content_translated_root, SETTINGS};
use rari_types::settings::Settings;
use self_update::cargo_crate_version;
use tabwriter::TabWriter;
use tracing_log::AsTrace;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod serve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[arg(short, long)]
    no_cache: bool,
    #[arg(long)]
    skip_updates: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build(BuildArgs),
    Foo(BuildArgs),
    Serve(ServeArgs),
    GitHistory,
    Popularities,
    Update(UpdateArgs),
}

#[derive(Args)]
struct UpdateArgs {
    #[arg(long)]
    version: Option<String>,
}

#[derive(Args)]
struct ServeArgs {
    #[arg(short, long)]
    deny_warnings: bool,
    #[arg(short, long)]
    cache_content: bool,
}

#[derive(Args)]
struct BuildArgs {
    #[arg(short, long)]
    files: Vec<PathBuf>,
    #[arg(short, long)]
    deny_warnings: bool,
    #[arg(short, long, default_value_t = true)]
    cache_content: bool,
    #[arg(long)]
    skip_content: bool,
    #[arg(long)]
    skip_contributors: bool,
    #[arg(long)]
    skip_search_index: bool,
    #[arg(long)]
    skip_blog: bool,
    #[arg(long)]
    skip_curriculum: bool,
    #[arg(long)]
    skip_spas: bool,
    #[arg(long)]
    skip_sitemap: bool,
    #[arg(long)]
    templ_stats: bool,
}

enum Cache {
    Static,
    Dynamic,
    None,
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    if !cli.skip_updates {
        rari_deps::webref_css::update_webref_css(rari_types::globals::data_dir())?;
        rari_deps::web_features::update_web_features(rari_types::globals::data_dir())?;
        rari_deps::bcd::update_bcd(rari_types::globals::data_dir())?;
        rari_deps::mdn_data::update_mdn_data(rari_types::globals::data_dir())?;
        rari_deps::web_ext_examples::update_web_ext_examples(rari_types::globals::data_dir())?;
    }

    let filter = filter::Targets::new()
        .with_target("rari_builder", cli.verbose.log_level_filter().as_trace())
        .with_target("rari_doc", cli.verbose.log_level_filter().as_trace())
        .with_target("rari", cli.verbose.log_level_filter().as_trace());

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .with(filter)
        .init();

    match cli.command {
        Commands::Foo(_args) => {}
        Commands::Build(args) => {
            let mut settings = Settings::new()?;
            settings.deny_warnings = args.deny_warnings;
            settings.cache_content = args.cache_content;
            let _ = SETTINGS.set(settings);

            let templ_stats = if args.templ_stats {
                let (tx, rx) = channel::<String>();
                TEMPL_RECORDER_SENDER
                    .set(tx.clone())
                    .expect("unable to create templ recorder");
                let recorder_handler = spawn(move || {
                    let mut stats = HashMap::new();
                    while let Ok(t) = rx.recv() {
                        if t == "∞" {
                            break;
                        }
                        let t = t.to_lowercase();
                        if let Some(n) = stats.get_mut(&t) {
                            *n += 1usize;
                        } else {
                            stats.insert(t, 1usize);
                        }
                    }
                    let mut out = stats.into_iter().collect::<Vec<(String, usize)>>();
                    out.sort_by(|(_, a), (_, b)| b.cmp(a));
                    println!("--- templ summary ---");
                    let mut tw = TabWriter::new(vec![]);
                    for (i, (templ, count)) in out.iter().enumerate() {
                        writeln!(&mut tw, "{:2}\t{templ}\t{count:4}", i + 1)
                            .expect("unable to write");
                    }
                    print!("{}", String::from_utf8_lossy(&tw.into_inner().unwrap()));
                });
                Some((recorder_handler, tx))
            } else {
                None
            };

            let cache = match (args.files.is_empty(), cli.no_cache) {
                (_, true) => Cache::None,
                (true, false) => Cache::Static,
                (false, false) => Cache::Dynamic,
            };

            if matches!(cache, Cache::Dynamic) {
                CACHED_DOC_PAGE_FILES
                    .set(Arc::new(RwLock::new(HashMap::new())))
                    .unwrap();
            }
            let mut urls = Vec::new();
            let mut docs = Vec::new();
            println!("Building everything 🛠️");
            if !args.skip_content {
                let start = std::time::Instant::now();
                docs = if !args.files.is_empty() {
                    read_docs_parallel::<Doc>(&args.files, None)?
                } else if !args.cache_content {
                    let files: &[_] = if let Some(translated_root) = content_translated_root() {
                        &[content_root(), translated_root]
                    } else {
                        &[content_root()]
                    };
                    read_docs_parallel::<Doc>(files, None)?
                } else {
                    read_and_cache_doc_pages()?
                };
                println!("Took: {: >10.3?} for {}", start.elapsed(), docs.len());
            }
            if !args.skip_spas && args.files.is_empty() {
                let start = std::time::Instant::now();
                urls.extend(build_spas()?);
                urls.extend(build_generic_pages()?);
                println!("Took: {: >10.3?} to build spas", start.elapsed());
            }
            if !args.skip_content {
                let start = std::time::Instant::now();
                urls.extend(build_docs(&docs)?);
                println!("Took: {: >10.3?} to build content", start.elapsed());
            }
            if !args.skip_search_index && args.files.is_empty() {
                let start = std::time::Instant::now();
                build_search_index(&docs)?;
                println!("Took: {: >10.3?} to build search index", start.elapsed());
            }
            if !args.skip_curriculum && args.files.is_empty() {
                let start = std::time::Instant::now();
                urls.extend(build_curriculum_pages()?);
                println!("Took: {: >10.3?} to build curriculum", start.elapsed());
            }
            if !args.skip_blog && args.files.is_empty() {
                let start = std::time::Instant::now();
                urls.extend(build_blog_pages()?);
                println!("Took: {: >10.3?} to build blog", start.elapsed());
            }
            if !args.skip_contributors && args.files.is_empty() {
                let start = std::time::Instant::now();
                urls.extend(build_contributor_spotlight_pages()?);
                println!(
                    "Took: {: >10.3?} to build contributor spotlight",
                    start.elapsed()
                );
            }
            if !args.skip_sitemap && args.files.is_empty() && !urls.is_empty() {
                let start = std::time::Instant::now();
                let out_path = build_out_root()?;
                fs::create_dir_all(out_path).unwrap();
                let out_file = out_path.join("sitemap.txt");
                let file = File::create(out_file).unwrap();
                let mut buffed = BufWriter::new(file);
                urls.sort();
                for url in &urls {
                    buffed.write_all(url.as_bytes())?;
                    buffed.write_all(b"\n")?;
                }
                println!(
                    "Took: {: >10.3?} to write sitemap.txt ({})",
                    start.elapsed(),
                    urls.len()
                );
            }
            if let Some((recorder_handler, tx)) = templ_stats {
                tx.send("∞".to_string())?;
                recorder_handler
                    .join()
                    .expect("unable to close templ recorder");
            }
        }
        Commands::Serve(args) => {
            let mut settings = Settings::new()?;
            settings.deny_warnings = args.deny_warnings;
            settings.cache_content = args.cache_content;
            let _ = SETTINGS.set(settings);
            serve::serve()?
        }
        Commands::GitHistory => {
            println!("Gathering histroy 📜");
            let start = std::time::Instant::now();
            gather_history();
            println!("Took: {:?}", start.elapsed());
        }
        Commands::Popularities => {
            println!("Calculating popularities 🥇");
            let start = std::time::Instant::now();
            update_popularities(20000);
            println!("Took: {:?}", start.elapsed());
        }
        Commands::Update(args) => update(args.version)?,
    }
    Ok(())
}

fn update(version: Option<String>) -> Result<(), Error> {
    let mut rel_builder = self_update::backends::github::ReleaseList::configure();
    rel_builder.repo_owner("mdn");

    let releases = rel_builder.repo_name("rari").build()?.fetch()?;

    let mut update_builder = self_update::backends::github::Update::configure();
    update_builder
        .repo_owner("mdn")
        .repo_name("rari")
        .bin_name("rari")
        .show_output(false)
        .no_confirm(true)
        .show_download_progress(true)
        .current_version(cargo_crate_version!());
    let target_release = if let Some(version) = version {
        if let Some(release) = releases.iter().find(|release| release.version == version) {
            update_builder.target_version_tag(&release.name);
            Some(release)
        } else {
            return Err(anyhow!("No version {version}"));
        }
    } else {
        None
    };
    let update = update_builder.build()?;
    let latest = update.get_latest_release().ok();

    let target_version = match (&latest, &target_release) {
        (None, None) => return Err(anyhow!("No latest release, specigy a version!")),
        (None, Some(target)) => {
            println!("Updating rari to {}", target.version);
            &target.version
        }
        (Some(latest), None) => {
            println!("Updating rari to {} (latest)", latest.version);
            &latest.version
        }
        (Some(latest), Some(target)) if latest.version == target.version => {
            println!("Updating rari to {} (latest)", latest.version);
            &latest.version
        }
        (Some(latest), Some(target)) => {
            println!(
                "Updating rari to {} (latest {})",
                target.version, latest.version
            );
            &target.version
        }
    };

    println!("rari `{target_version}` will be downloaded/extracted.");
    println!(
        "The current rari ({}) at version `{}` will be replaced.",
        update.bin_install_path().to_string_lossy(),
        update.current_version()
    );
    print!("Do you want to continue? [Y/n] ");
    io::stdout().flush()?;

    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let s = s.trim().to_lowercase();
    if !s.is_empty() && s != "y" {
        return Err(anyhow!("Update aborted"));
    }
    let status = update.update()?;
    println!("\n\nrari updated to `{}`", status.version());
    Ok(())
}
