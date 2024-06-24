use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use clap::{Args, Parser, Subcommand};
use rari_doc::build::{build_blog_pages, build_curriculum_pages, build_docs};
use rari_doc::cached_readers::{CACHED_PAGE_FILES, STATIC_PAGE_FILES};
use rari_doc::docs::doc::Doc;
use rari_doc::docs::page::PageLike;
use rari_doc::walker::read_docs_parallel;
use rari_tools::history::gather_history;
use rari_tools::popularities::update_popularities;
use rari_types::globals::SETTINGS;
use rari_types::settings::Settings;
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
    skip_blog: bool,
    #[arg(long)]
    skip_curriculum: bool,
}

enum Cache {
    Static,
    Dynamic,
    None,
}

fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    rari_deps::webref_css::update_webref_css(rari_types::globals::data_dir())?;
    rari_deps::web_features::update_web_features(rari_types::globals::data_dir())?;
    rari_deps::bcd::update_bcd(rari_types::globals::data_dir())?;

    let filter = filter::Targets::new()
        .with_target("rari_builder", cli.verbose.log_level_filter().as_trace())
        .with_target("rari_doc", cli.verbose.log_level_filter().as_trace());

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
            let cache = match (args.files.is_empty(), cli.no_cache) {
                (_, true) => Cache::None,
                (true, false) => Cache::Static,
                (false, false) => Cache::Dynamic,
            };

            if matches!(cache, Cache::Dynamic) {
                CACHED_PAGE_FILES
                    .set(Arc::new(RwLock::new(HashMap::new())))
                    .unwrap();
            }
            println!("Building everything 🛠️");
            let start = std::time::Instant::now();
            let docs = read_docs_parallel::<Doc>(&args.files, None)?;
            if matches!(cache, Cache::Static) {
                STATIC_PAGE_FILES
                    .set(
                        docs.iter()
                            .cloned()
                            .map(|doc| (doc.full_path().to_owned(), doc))
                            .collect(),
                    )
                    .unwrap();
            }
            println!("Took: {:?} for {}", start.elapsed(), docs.len());
            if !args.skip_content {
                let start = std::time::Instant::now();
                build_docs(docs)?;
                println!("Took: {:?} to build content", start.elapsed());
            }
            if !args.skip_curriculum && args.files.is_empty() {
                let start = std::time::Instant::now();
                build_curriculum_pages()?;
                println!("Took: {:?} to build curriculum", start.elapsed());
            }
            if !args.skip_blog && args.files.is_empty() {
                let start = std::time::Instant::now();
                build_blog_pages()?;
                println!("Took: {:?} to build blog", start.elapsed());
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
    }
    Ok(())
}
