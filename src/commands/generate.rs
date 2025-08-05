use std::io;

use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator};
use tracing::instrument;

use crate::utils::state::Result;

use super::{Cli, GenerateArgs};

/// Prints the generated completions to stdout.
fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

/// Generates shell completion scripts.
#[instrument(skip(cmd))]
pub async fn run(cmd: &GenerateArgs) -> Result<bool> {
    utpm_log!(trace, "executing generate command");
    let generator = cmd.generator;
    let mut cmd: Command = Cli::command();
    eprintln!("Generating completion file for {generator:?}...");
    print_completions(generator, &mut cmd);
    Ok(true)
}
