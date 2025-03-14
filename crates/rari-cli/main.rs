use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread::spawn;

use anyhow::{anyhow, Error};
use clap::{Args, Parser, Subcommand};
use clap_verbosity_flag::Verbosity;
use dashmap::DashMap;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use rari_doc::build::{
    build_blog_pages, build_contributor_spotlight_pages, build_curriculum_pages, build_docs,
    build_generic_pages, build_spas, build_top_level_meta,
};
use rari_doc::cached_readers::{read_and_cache_doc_pages, CACHED_DOC_PAGE_FILES};
use rari_doc::issues::IN_MEMORY;
use rari_doc::pages::json::BuiltPage;
use rari_doc::pages::page::Page;
use rari_doc::pages::types::doc::Doc;
use rari_doc::reader::read_docs_parallel;
use rari_doc::search_index::build_search_index;
use rari_doc::utils::TEMPL_RECORDER_SENDER;
use rari_sitemap::Sitemaps;
use rari_tools::add_redirect::add_redirect;
use rari_tools::fix::fixer::fix_all;
use rari_tools::history::gather_history;
use rari_tools::inventory::gather_inventory;
use rari_tools::r#move::r#move;
use rari_tools::redirects::{fix_redirects, validate_redirects};
use rari_tools::remove::remove;
use rari_tools::sidebars::{fmt_sidebars, sync_sidebars};
use rari_tools::sync_translated_content::sync_translated_content;
use rari_types::globals::{build_out_root, content_root, content_translated_root, SETTINGS};
use rari_types::locale::Locale;
use rari_types::settings::Settings;
use rari_utils::io::read_to_string;
use schemars::schema_for;
use self_update::cargo_crate_version;
use tabwriter::TabWriter;
use tracing::level_filters::LevelFilter;
use tracing::{info, Level};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{filter, Layer};

mod serve;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Skip updating dependencies (bcd, webref, ...)
    #[arg(short, long)]
    skip_updates: bool,
    #[command(flatten)]
    verbose: Verbosity,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build MDN.
    Build(BuildArgs),
    /// Run the local dev server.
    Serve(ServeArgs),
    /// Collect the git history.
    GitHistory,
    /// Self-update rari (caution if installed from npm)
    Update(UpdateArgs),
    /// Export json schema.
    ExportSchema(ExportSchemaArgs),
    /// Subcommands for altering content programmatically
    #[command(subcommand)]
    Content(ContentSubcommand),
}

#[derive(Args)]
struct FixFlawsArgs {
    #[arg(short, long, help = "Only fix flaws for <LOCALE>")]
    locale: Option<Locale>,
}
#[derive(Args)]
struct ExportSchemaArgs {
    output_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum ContentSubcommand {
    /// Moves content pages from one slug to another.
    Move(MoveArgs),
    /// Deletes content pages.
    Delete(DeleteArgs),
    /// Adds a redirect from->to pair to the redirect map.
    ///
    /// The locale is inferred from the from_url.
    AddRedirect(AddRedirectArgs),
    /// Syncs translated content for all or a list of locales.
    SyncTranslatedContent(SyncTranslatedContentArgs),
    /// Formats all sidebars.
    FmtSidebars,
    /// Sync sidebars with redirects
    SyncSidebars,
    /// Fixes redirects across all locales.
    ///
    /// This shortens multiple redirect chains to single ones.
    /// This is also run as part of sync_translated_content.
    FixRedirects(FixRedirectArgs),
    /// Validate redirects.
    ValidateRedirects(ValidateRedirectArgs),
    /// Create content inventory as JSON
    Inventory,
    /// Fix all flaws (currently only broken_links)
    FixFlaws(FixFlawsArgs),
}

#[derive(Args)]
struct MoveArgs {
    old_slug: String,
    new_slug: String,
    locale: Option<Locale>,
    #[arg(short = 'y', long, help = "Assume yes to all prompts")]
    assume_yes: bool,
}

#[derive(Args)]
struct DeleteArgs {
    slug: String,
    locale: Option<Locale>,
    #[arg(short, long, default_value_t = false)]
    recursive: bool,
    #[arg(long)]
    redirect: Option<String>,
    #[arg(short = 'y', long, help = "Assume yes to all prompts")]
    assume_yes: bool,
}

#[derive(Args)]
struct AddRedirectArgs {
    from_url: String,
    to_url: String,
}

#[derive(Args)]
struct ValidateRedirectArgs {
    locales: Option<Vec<Locale>>,
}

#[derive(Args)]
struct FixRedirectArgs {
    locales: Option<Vec<Locale>>,
}

#[derive(Args)]
struct SyncTranslatedContentArgs {
    locales: Option<Vec<Locale>>,
}

#[derive(Args)]
struct UpdateArgs {
    #[arg(long)]
    version: Option<String>,
}

#[derive(Args)]
struct ServeArgs {
    #[arg(long, help = "Caution! Don't use when editing content.")]
    cache: bool,
}

#[derive(Args)]
struct BuildArgs {
    #[arg(short, long, help = "Build only content <FILES>")]
    files: Vec<PathBuf>,
    #[arg(short, long, help = "Build only content listed in <FILE_LIST>")]
    file_list: Option<PathBuf>,
    #[arg(short, long, help = "Abort build on warnings")]
    deny_warnings: bool,
    #[arg(long, help = "Disable caching (only for debugging)")]
    no_cache: bool,
    #[arg(long, help = "Build everything")]
    all: bool,
    #[arg(
        long,
        help = "Don't automatically build basics: content, spas, search-index"
    )]
    no_basic: bool,
    #[arg(long, help = "Build content")]
    content: bool,
    #[arg(long, help = "Build spas")]
    spas: bool,
    #[arg(long, help = "Build search-index")]
    search_index: bool,
    #[arg(long, help = "Build contributor spotlights")]
    spotlights: bool,
    #[arg(long, help = "Build blog")]
    blog: bool,
    #[arg(long, help = "Build curriculum")]
    curriculum: bool,
    #[arg(long, help = "Build generic-content")]
    generics: bool,
    #[arg(long, help = "Build sitemaps")]
    sitemaps: bool,
    #[arg(long, help = "Display template statistics (debugging")]
    templ_stats: bool,
    #[arg(long, help = "Write all issues to path <ISSUES>")]
    issues: Option<PathBuf>,
    #[arg(long, help = "Annotate html with 'data-flaw' attributes")]
    data_issues: bool,
    #[arg(long, help = "Add flaws field to index.json for docs")]
    json_issues: bool,
    #[arg(
        short,
        long,
        help = "Noop flag to legacy compatibility (has no effect on build)"
    )]
    noop: bool,
}

#[derive(Debug)]
enum Cache {
    Static,
    Dynamic,
    None,
}

fn main() -> Result<(), Error> {
    if let Ok(env_file) = dotenvy::from_filename(
        env::var("DOT_FILE")
            .map(Cow::Owned)
            .unwrap_or(Cow::Borrowed(".env"))
            .as_ref(),
    ) {
        info!("Using env_file: {}", env_file.display())
    }
    let cli = Cli::parse();
    let fmt_filter =
        filter::Targets::new().with_target("rari_doc", cli.verbose.tracing_level_filter());

    let cli_level = if cli.verbose.is_silent() {
        LevelFilter::OFF
    } else {
        LevelFilter::INFO
    };
    let cli_filter = filter::Targets::new()
        .with_target("rari", cli_level)
        .with_target("rari_tools", cli_level)
        .with_target("rari_deps", cli_level)
        .with_target("rari_doc", LevelFilter::OFF);

    let memory_filter = filter::Targets::new()
        .with_target("rari_doc", Level::WARN)
        .with_target("rari", Level::WARN);

    let memory_layer = IN_MEMORY.clone();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_filter(fmt_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_level(false)
                .compact()
                .with_target(false)
                .with_filter(cli_filter),
        )
        .with(memory_layer.clone().with_filter(memory_filter))
        .init();

    if !cli.skip_updates {
        rari_deps::webref_css::update_webref_css(rari_types::globals::data_dir())?;
        rari_deps::web_features::update_web_features(rari_types::globals::data_dir())?;
        rari_deps::bcd::update_bcd(rari_types::globals::data_dir())?;
        rari_deps::mdn_data::update_mdn_data(rari_types::globals::data_dir())?;
        rari_deps::web_ext_examples::update_web_ext_examples(rari_types::globals::data_dir())?;
        rari_deps::popularities::update_popularities(rari_types::globals::data_dir())?;
    }

    match cli.command {
        Commands::Build(args) => {
            let mut settings = Settings::new()?;
            settings.deny_warnings = args.deny_warnings;
            settings.cache_content = !args.no_cache;
            settings.data_issues = args.data_issues;
            settings.json_issues = args.json_issues;
            let _ = SETTINGS.set(settings);

            let mut arg_files = args
                .files
                .iter()
                .map(|path| path.canonicalize())
                .collect::<Result<Vec<PathBuf>, _>>()?;

            if let Some(file_list) = args.file_list {
                arg_files.extend(
                    read_to_string(&file_list)?
                        .lines()
                        .filter(|s| !s.is_empty())
                        .map(|line| Ok(PathBuf::from_str(line)?.canonicalize()?))
                        .collect::<Result<Vec<PathBuf>, Error>>()?,
                );
            }

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
                    info!("--- templ summary ---");
                    let mut tw = TabWriter::new(vec![]);
                    for (i, (templ, count)) in out.iter().enumerate() {
                        writeln!(&mut tw, "{:2}\t{templ}\t{count:4}", i + 1)
                            .expect("unable to write");
                    }
                    info!("{}", String::from_utf8_lossy(&tw.into_inner().unwrap()));
                });
                Some((recorder_handler, tx))
            } else {
                None
            };

            let cache = match (arg_files.is_empty(), args.no_cache) {
                (_, true) => Cache::None,
                (true, false) => Cache::Static,
                (false, false) => Cache::Dynamic,
            };

            if matches!(cache, Cache::Dynamic) {
                CACHED_DOC_PAGE_FILES.set(Arc::new(DashMap::new())).unwrap();
            }
            let mut urls = Vec::new();
            let mut docs = Vec::new();
            info!("Building everything 🛠️");
            if args.all || !args.no_basic || args.content || !arg_files.is_empty() {
                let start = std::time::Instant::now();
                docs = if !arg_files.is_empty() {
                    read_docs_parallel::<Page, Doc>(&arg_files, None)?
                } else if args.no_cache {
                    let files: &[_] = if let Some(translated_root) = content_translated_root() {
                        &[content_root(), translated_root]
                    } else {
                        &[content_root()]
                    };
                    read_docs_parallel::<Page, Doc>(files, None)?
                } else {
                    read_and_cache_doc_pages()?
                };
                info!(
                    "Took: {: >10.3?} for reading {} docs",
                    start.elapsed(),
                    docs.len()
                );
            }
            if args.all || !args.no_basic || args.spas {
                let start = std::time::Instant::now();
                let spas = build_spas()?;
                let num = spas.len();
                urls.extend(spas);
                info!("Took: {: >10.3?} to build spas ({num})", start.elapsed(),);
            }
            if args.all || !args.no_basic || args.content || !arg_files.is_empty() {
                let start = std::time::Instant::now();
                let (docs, meta) = build_docs(&docs)?;
                build_top_level_meta(meta)?;
                let num = docs.len();
                urls.extend(docs);
                info!(
                    "Took: {: >10.3?} to build content docs ({num})",
                    start.elapsed()
                );
            }
            if args.all || !args.no_basic || args.search_index {
                let start = std::time::Instant::now();
                build_search_index(&docs)?;
                info!("Took: {: >10.3?} to build search index", start.elapsed());
            }
            if args.all || args.generics {
                let start = std::time::Instant::now();
                let generic_pages = build_generic_pages()?;
                let num = generic_pages.len();
                urls.extend(generic_pages);
                info!(
                    "Took: {: >10.3?} to build generic pages ({num})",
                    start.elapsed()
                );
            }
            if args.all || args.curriculum {
                let start = std::time::Instant::now();
                let curriculum_pages = build_curriculum_pages()?;
                let num = curriculum_pages.len();
                urls.extend(curriculum_pages);
                info!(
                    "Took: {: >10.3?} to build curriculum pages ({num})",
                    start.elapsed()
                );
            }
            if args.all || args.blog {
                let start = std::time::Instant::now();
                let blog_pages = build_blog_pages()?;
                let num = blog_pages.len();
                urls.extend(blog_pages);
                info!("Took: {: >10.3?} to build blog ({num})", start.elapsed());
            }
            if args.all || args.spotlights {
                let start = std::time::Instant::now();
                let contributor_spotlight_pages = build_contributor_spotlight_pages()?;
                let num = contributor_spotlight_pages.len();
                urls.extend(contributor_spotlight_pages);
                info!(
                    "Took: {: >10.3?} to build contributor spotlights ({num})",
                    start.elapsed()
                );
            }
            if args.all || args.sitemaps && !urls.is_empty() {
                let sitemaps = Sitemaps { sitemap_meta: urls };
                let start = std::time::Instant::now();
                let out_path = build_out_root()?;
                fs::create_dir_all(out_path).unwrap();
                sitemaps.write_all_sitemaps(out_path)?;
                info!(
                    "Took: {: >10.3?} to write sitemaps ({})",
                    start.elapsed(),
                    sitemaps.sitemap_meta.len()
                );
            }
            if let Some((recorder_handler, tx)) = templ_stats {
                tx.send("∞".to_string())?;
                recorder_handler
                    .join()
                    .expect("unable to close templ recorder");
            }

            if let Some(issues_path) = args.issues {
                let events = memory_layer.get_events();
                let file = File::create(issues_path).unwrap();
                let mut buffed = BufWriter::new(file);
                serde_json::to_writer_pretty(&mut buffed, &*events).unwrap();
            }
        }
        Commands::Serve(args) => {
            let mut settings = Settings::new()?;
            settings.cache_content = args.cache;
            settings.data_issues = true;
            settings.blog_unpublished = true;
            let _ = SETTINGS.set(settings);
            serve::serve()?
        }
        Commands::GitHistory => {
            info!("Gathering history 📜");
            let start = std::time::Instant::now();
            gather_history()?;
            info!("Took: {:?}", start.elapsed());
        }
        Commands::Content(content_subcommand) => match content_subcommand {
            ContentSubcommand::Move(args) => {
                r#move(&args.old_slug, &args.new_slug, args.locale, args.assume_yes)?;
            }
            ContentSubcommand::Delete(args) => {
                remove(
                    &args.slug,
                    args.locale,
                    args.recursive,
                    args.redirect.as_deref(),
                    args.assume_yes,
                )?;
            }
            ContentSubcommand::AddRedirect(args) => {
                add_redirect(&args.from_url, &args.to_url)?;
            }
            ContentSubcommand::SyncTranslatedContent(args) => {
                let locales = args.locales.as_deref().unwrap_or(Locale::translated());
                sync_translated_content(locales, cli.verbose.is_present())?;
            }
            ContentSubcommand::FmtSidebars => {
                fmt_sidebars()?;
            }
            ContentSubcommand::SyncSidebars => {
                sync_sidebars()?;
            }
            ContentSubcommand::FixRedirects(args) => {
                fix_redirects(args.locales.as_deref())?;
            }
            ContentSubcommand::ValidateRedirects(args) => {
                validate_redirects(args.locales.as_deref())?;
            }
            ContentSubcommand::Inventory => {
                gather_inventory()?;
            }
            ContentSubcommand::FixFlaws(args) => {
                let start = std::time::Instant::now();
                let mut settings = Settings::new()?;
                settings.cache_content = true;
                let _ = SETTINGS.set(settings);
                let docs = read_and_cache_doc_pages()?;
                info!(
                    "Took: {: >10.3?} for reading {} docs",
                    start.elapsed(),
                    docs.len()
                );
                let start = std::time::Instant::now();
                let fixed = fix_all(&docs, args.locale)?;
                info!(
                    "Took: {: >10.3?} for fixing {} docs",
                    start.elapsed(),
                    fixed.len()
                );
            }
        },
        Commands::Update(args) => update(args.version)?,
        Commands::ExportSchema(args) => export_schema(args)?,
    }
    Ok(())
}

fn export_schema(args: ExportSchemaArgs) -> Result<(), Error> {
    let out_path = args
        .output_file
        .unwrap_or_else(|| PathBuf::from("schema.json"));
    let schema = schema_for!(BuiltPage);
    fs::write(out_path, serde_json::to_string_pretty(&schema)?)?;
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
    // Use GITHUB_TOKEN if it's set to avoid rate limiting
    if let Ok(gh_token) = env::var("GITHUB_TOKEN") {
        update_builder.auth_token(gh_token.as_str());
    }
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
        (None, None) => return Err(anyhow!("No latest release, specify a version!")),
        (None, Some(target)) => {
            info!("Updating rari to {}", target.version);
            &target.version
        }
        (Some(latest), None) => {
            info!("Updating rari to {} (latest)", latest.version);
            &latest.version
        }
        (Some(latest), Some(target)) if latest.version == target.version => {
            info!("Updating rari to {} (latest)", latest.version);
            &latest.version
        }
        (Some(latest), Some(target)) => {
            info!(
                "Updating rari to {} (latest {})",
                target.version, latest.version
            );
            &target.version
        }
    };

    info!("rari `{target_version}` will be downloaded/extracted.");
    info!(
        "The current rari ({}) at version `{}` will be replaced.",
        update.bin_install_path().to_string_lossy(),
        update.current_version()
    );
    if !Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to continue?")
        .default(true)
        .interact()
        .unwrap_or_default()
    {
        return Err(anyhow!("Update aborted"));
    }
    let status = update.update()?;
    info!("\n\nrari updated to `{}`", status.version());
    Ok(())
}
