use anyhow::Result;
use shadow_rs::shadow;
shadow!(build);

pub mod commands;
pub mod utils;

use std::{env, str::FromStr};

use utils::output::OUTPUT_FORMAT;

use clap::Parser;
#[cfg(any(
    feature = "tree",
    feature = "list",
    feature = "path",
    feature = "unlink",
    feature = "bulk_delete"
))]
use commands::Packages;
//todo: workspace
#[cfg(any(
    feature = "link",
    feature = "init",
    feature = "install",
    feature = "add",
    feature = "delete",
    feature = "init",
    feature = "publish",
    feature = "clone"
))]
use commands::Workspace;
use commands::{Cli, Commands};

use utils::state::UtpmError;

use tracing::{Level, error, instrument, level_filters::LevelFilter};
use tracing_subscriber::{self, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::utils::{
    dryrun::{DRYRUN, get_dry_run},
    output::{OutputFormat, get_output_format},
};

/// The main entry point of the UTPM application.
///
/// This function initializes the command-line interface, parses arguments,
/// sets up logging, and dispatches to the appropriate command handler.
#[instrument]
#[tokio::main]
async fn main() {
    // Parse command-line arguments.
    let x = Cli::parse();

    // Set up logging level from `UTPM_DEBUG` env var or default to `info`.
    let debug_str: String = match env::var("UTPM_DEBUG") {
        Err(_) => "info".into(),
        Ok(val) => val,
    };

    // Convert the log level string to a `LevelFilter`.
    let level_filter = match Level::from_str(debug_str.as_str()) {
        Ok(val) => val,
        Err(_) => Level::INFO,
    };

    // Set the global output format.
    OUTPUT_FORMAT
        .set(x.output_format.unwrap_or(OutputFormat::Text))
        .unwrap();

    // Set the dry-run boolean
    DRYRUN.set(x.dry_run).unwrap();

    if get_dry_run() {
        utpm_log!(info, "Using dry-run")
    }

    // Initialize the tracing subscriber based on the output format.
    if get_output_format() != OutputFormat::Text {
        // Use JSON format for logs if output is not plain text.
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_filter(LevelFilter::from_level(if let Some(debug) = x.verbose {
                        debug
                    } else {
                        level_filter
                    })),
            )
            .init();
    } else {
        // Use standard format for text output.
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer().with_filter(LevelFilter::from_level(
                    if let Some(debug) = x.verbose {
                        debug
                    } else {
                        level_filter
                    },
                )),
            )
            .init();
    }

    // Dispatch the command to its handler.
    let res = async move {
        match &x.command {
            #[cfg(any(
                feature = "link",
                feature = "init",
                feature = "install",
                feature = "add",
                feature = "delete",
                feature = "init",
                feature = "publish",
                feature = "sync",
                feature = "bump",
                feature = "clone"
            ))]
            Commands::Workspace(w) => match w {
                #[cfg(feature = "link")]
                Workspace::Link(cmd) => commands::link::run(cmd, None, true).await,
                #[cfg(feature = "install")]
                Workspace::Install(cmd) => commands::install::run(cmd).await,
                #[cfg(feature = "add")]
                Workspace::Add(cmd) => commands::add::run(&mut cmd.clone()).await,
                #[cfg(feature = "delete")]
                Workspace::Delete(cmd) => commands::delete::run(&mut cmd.clone()).await,
                #[cfg(feature = "init")]
                Workspace::Init(cmd) => commands::init::run(&mut cmd.clone()).await,
                #[cfg(feature = "clone")]
                Workspace::Clone(cmd) => commands::clone::run(cmd).await,
                #[cfg(feature = "bump")]
                Workspace::Bump(cmd) => commands::bump::run(cmd).await,
                #[cfg(feature = "sync")]
                Workspace::Sync(cmd) => commands::sync::run(cmd).await,
            },
            #[cfg(any(
                feature = "tree",
                feature = "list",
                feature = "path",
                feature = "unlink",
                feature = "bulk_delete",
                feature = "get"
            ))]
            Commands::Packages(p) => {
                match p {
                    // TODO: Consider a `move` command to change namespace, name, or version.
                    #[cfg(feature = "tree")]
                    Packages::Tree(cmd) => commands::tree::run(cmd).await,
                    #[cfg(feature = "list")]
                    Packages::List(cmd) => commands::list::run(cmd).await,
                    #[cfg(feature = "path")]
                    Packages::Path => commands::package_path::run().await,
                    #[cfg(feature = "unlink")]
                    Packages::Unlink(cmd) => commands::unlink::run(cmd).await,
                    #[cfg(feature = "bulk_delete")]
                    Packages::BulkDelete(cmd) => commands::bulk_delete::run(cmd).await,
                    #[cfg(feature = "get")]
                    Packages::Get(cmd) => commands::get::run(cmd).await,
                }
            }
            #[cfg(feature = "generate")]
            Commands::Generate(cmd) => commands::generate::run(cmd).await,
        }
    }
    .await;

    // Handle any errors that occurred during command execution.
    if let Err(err) = res {
        match check_errors(err) {
            Ok(_) => (),
            Err(err2) => error!("{err2}"),
        };
    }
}

/// A fallback mechanism to print errors if the primary logging fails.
///
/// If the command execution results in an error, this function is called
/// to log the error to the console.
fn check_errors(err: UtpmError) -> Result<()> {
    utpm_log!(@f error, "{err}");
    return Ok(());
}


