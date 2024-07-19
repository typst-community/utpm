pub mod commands;
pub mod utils;

use clap::Parser;
use commands::{
    bulk_delete, create, install, link, list, package_path, tree, unlink, Cli, Commands, Packages,
    Workspace,
};

use utils::state::Error;

fn main() {
    let x = Cli::parse();

    let res: Result<bool, Error> = match &x.command {
        Commands::Workspace(w) => match w {
            Workspace::Link(cmd) => link::run(cmd, None, true),
            Workspace::Create(cmd) => create::run(&mut cmd.clone()),
            Workspace::Install(cmd) => install::run(cmd),
            Workspace::Add => todo!(),
            Workspace::Delete => todo!(),
            Workspace::Init(cmd) => create::run(&mut cmd.clone()),
            Workspace::Publish => todo!(),
        },
        Commands::Packages(p) => match p {
            Packages::Tree(cmd) => tree::run(cmd),
            Packages::List(cmd) => list::run(cmd),
            Packages::Path => package_path::run(),
            Packages::Unlink(cmd) => unlink::run(cmd),
            Packages::BulkDelete(cmd) => bulk_delete::run(cmd),
        },
    };

    //Commands::Install(cmd) => ,

    match res {
        Ok(_) => {}
        Err(val) => println!("{}", val),
    }
}
