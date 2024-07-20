// Linker
pub mod bulk_delete;
pub mod create;
pub mod install;
pub mod link;
pub mod list;
pub mod package_path;
pub mod tree;
pub mod unlink;
pub mod add;

use clap::{Parser, Subcommand};
use typst_project::manifest::{categories::Category, disciplines::Discipline};

#[derive(Parser, Clone, Debug)]
pub struct CreateInitArgs {
    /// Desactivate interactive session
    #[arg(short = 'm', long, requires = "ni")]
    cli: bool,

    /// Force the creation of a file
    #[arg(short, long)]
    force: bool,

    /// Name of the project
    #[arg(short, long, group = "ni")]
    name: Option<String>,

    /// Version of the project
    #[arg(short, long, default_value_t=semver::Version::parse("1.0.0").unwrap())]
    version: semver::Version,

    /// Path to the main file of the project
    #[arg(short, long, default_value_t=String::from("main.typ"))]
    entrypoint: String,

    /// Authors of the project
    #[arg(short, long)]
    #[clap(value_delimiter = ',')]
    authors: Option<Vec<String>>,

    /// License
    #[arg(short, long)]
    license: Option<String>,

    /// A little description
    #[arg(short, long)]
    description: Option<String>,

    /// The link to your repository
    #[arg(short, long)]
    repository: Option<String>,

    /// Link to your homepage
    #[arg(short = 'H', long)]
    homepage: Option<String>,

    /// Keywords to find your project
    #[arg(short, long)]
    #[clap(value_delimiter = ',')]
    keywords: Option<Vec<String>>,

    /// Minimum compiler version
    #[arg(short, long)]
    compiler: Option<semver::Version>,

    /// Excludes files
    #[arg(short = 'x', long)]
    #[clap(value_delimiter = ',')]
    exclude: Option<Vec<String>>,

    /// Namespace
    #[arg(short = 'N', long)]
    namespace: Option<String>,

    /// Populate
    #[arg(short = 'p', long)]
    populate: bool,

    #[arg(short = 'C', long)]
    #[clap(value_delimiter = ',')]
    categories: Option<Vec<Category>>,

    #[arg(short = 'D', long)]
    #[clap(value_delimiter = ',')]
    disciplines: Option<Vec<Discipline>>,

    #[arg(long, requires = "template")]
    template_path: Option<String>,

    #[arg(long, requires = "template")]
    template_entrypoint: Option<String>,

    #[arg(long)]
    template_thumbnail: Option<String>,
}

#[derive(Parser, Clone, Debug)]
pub struct LinkArgs {
    /// Force the copy of the dir / creation of the symlink
    #[arg(short, long)]
    pub force: bool,

    /// Will create a symlink instead of copying
    #[arg(short, long)]
    pub no_copy: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct ListTreeArgs {
    /// Will list all packages including @preview
    #[arg(short, long)]
    pub all: bool,

    /// List all subdirectory you want
    #[arg(short, long, num_args = 1..)]
    pub include: Option<Vec<String>>,
}

#[derive(Parser, Clone, Debug)]
pub struct UnlinkArgs {
    /// The name of the package
    name: Option<String>,

    /// Namespace, where your packages are install (default local)
    #[arg(short, long)]
    namespace: Option<String>,

    /// Do you want to delete the namespace or not
    #[arg(short, long)]
    delete_namespace: bool,

    /// The version you want to delete, if nothing has been provided it will be the whole package
    #[arg(short, long)]
    version: Option<semver::Version>,

    /// Confirm the deletion of a dir
    #[arg(short, long)]
    yes: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct BulkDeleteArgs {
    /// Names of your packages, use version with this syntax: mypackage:1.0.0
    #[clap(value_delimiter = ',')]
    names: Vec<String>,

    /// The namespace you want to bulk-delete
    #[arg(short, long)]
    namespace: Option<String>,
}

#[derive(Parser, Clone, Debug)]
pub struct InstallArgs {
    /// If you want to install a specific package
    pub url: Option<String>,

    /// Passed force to all link commands
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

#[derive(Parser, Clone, Debug)]
pub struct AddArgs {
    /// The url or path of your repository. 
    pub uri: String,
}

/// Commands to use packages related to typst
#[derive(Subcommand, Debug)]
pub enum Packages {
    /// List all of packages from your dir, in a form of a tree
    #[command(visible_alias = "t")]
    Tree(ListTreeArgs),

    /// List all of packages from your dir, in a form of a list
    #[command(visible_alias = "l")]
    List(ListTreeArgs),

    /// Display path to typst packages folder
    #[command(visible_alias = "p")]
    Path,

    /// Delete package previously install with utpm
    #[command(visible_alias = "u")]
    Unlink(UnlinkArgs),

    #[command(visible_alias = "bd")]
    BulkDelete(BulkDeleteArgs),
}

/// Commands to create, edit, delete your workspace for your package.
#[derive(Subcommand, Debug)]
pub enum Workspace {
    /// Link your project to your dirs
    #[command(visible_alias = "l")]
    Link(LinkArgs),

    /// Create a file for your project (typst.toml)
    /// (deprecated, use `utpm workspace init`)
    #[command(visible_alias = "c")]
    Create(CreateInitArgs),

    /// Install all dependencies from the `typst.toml`
    #[command(visible_alias = "i")]
    Install(InstallArgs),

    /// WIP
    #[command(visible_alias = "a")]
    Add(AddArgs),

    /// WIP
    #[command(visible_alias = "d")]
    Delete,

    /// Create your workspace to start a typst package
    Init(CreateInitArgs),

    /// WIP
    #[command(visible_alias = "p")]
    Publish,
}

#[derive(Subcommand, Debug)]
pub enum Commands {

    #[command(subcommand)]
    #[command(visible_alias = "ws")]
    Workspace(Workspace),

    #[command(subcommand)]
    #[command(visible_alias = "pkg")]
    Packages(Packages),
}

#[derive(Parser)]
#[command(author = "Thumus", version = "3.0.0")]
/// An unofficial typst package manager for your projects.
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
