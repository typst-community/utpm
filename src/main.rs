pub mod commands;
pub mod utils;

use clap::Parser;
use commands::{
    bulk_delete, create, install, link, list, package_path, tree, unlink, Cli, Commands
};

use utils::state::Error;

fn main() {
    let x = Cli::parse();

    let res: Result<bool, Error> = match &x.command {
        Commands::Create(cmd) => create::run(&mut cmd.clone()),
        Commands::Link(cmd) => link::run(cmd, None),
        Commands::Tree(cmd) => tree::run(cmd),
        Commands::List(cmd) => list::run(cmd),
        Commands::PackagesPath => package_path::run(),
        Commands::Unlink(cmd) => unlink::run(cmd),
        Commands::BulkDelete(cmd) => bulk_delete::run(cmd),
        Commands::Install(cmd) => install::run(cmd),
    };

    match res {
        Ok(_) => {},
        Err(val) => println!("{}", val),
    }
}
