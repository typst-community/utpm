/// Linker: This module dynamically links all the command modules.
/// Each command is a separate module, conditionally compiled based on feature flags.
#[cfg(feature = "add")]
pub mod add;
#[cfg(feature = "bulk_delete")]
pub mod bulk_delete;
#[cfg(feature = "bump")]
pub mod bump;
#[cfg(feature = "clone")]
pub mod clone;
#[cfg(feature = "delete")]
pub mod delete;
#[cfg(feature = "generate")]
pub mod generate;
pub mod get;
#[cfg(feature = "init")]
pub mod init;
#[cfg(feature = "install")]
pub mod install;
#[cfg(feature = "link")]
pub mod link;
#[cfg(feature = "list")]
pub mod list;
#[cfg(feature = "path")]
pub mod package_path;
#[cfg(feature = "publish")]
pub mod publish;
#[cfg(feature = "sync")]
pub mod sync;
#[cfg(feature = "tree")]
pub mod tree;
#[cfg(feature = "unlink")]
pub mod unlink;

#[cfg(any(feature = "clone", feature = "publish"))]
use std::path::PathBuf;

use clap::{Parser, Subcommand};
#[cfg(feature = "generate")]
use clap_complete::Shell;
use tracing::Level;

use crate::build;
use crate::utils::output::OutputFormat;

/// Arguments for the `init` command.
/// This command initializes a new `typst.toml` manifest file in the current directory.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "init")]
pub struct InitArgs {
    /// Disable interactive session and use command-line arguments only.
    #[arg(short = 'm', long, requires = "ni")]
    cli: bool,

    /// Force the creation of the manifest file, overwriting if it exists.
    #[arg(short, long)]
    force: bool,

    /// Name of the project.
    #[arg(short, long, group = "ni")]
    name: Option<String>,

    /// Version of the project.
    #[arg(short = 'V', long, default_value_t=semver::Version::parse("1.0.0").unwrap())]
    version: semver::Version,

    /// Path to the main file of the project.
    #[arg(short, long, default_value_t=String::from("main.typ"))]
    entrypoint: String,

    /// Authors of the project.
    #[arg(short, long)]
    #[clap(value_delimiter = ',')]
    authors: Option<Vec<String>>,

    /// License of the project.
    #[arg(short, long)]
    license: Option<String>,

    /// A short description of the project.
    #[arg(short, long)]
    description: Option<String>,

    /// The link to your repository.
    #[arg(short, long)]
    repository: Option<String>,

    /// Link to your homepage.
    #[arg(short = 'H', long)]
    homepage: Option<String>,

    /// Keywords to find your project.
    #[arg(short, long)]
    #[clap(value_delimiter = ',')]
    keywords: Option<Vec<String>>,

    /// Minimum compiler version required.
    #[arg(short, long)]
    compiler: Option<semver::Version>,

    /// Files to exclude from the package.
    #[arg(short = 'x', long)]
    #[clap(value_delimiter = ',')]
    exclude: Option<Vec<String>>,

    /// Namespace to put your package in.
    #[arg(short = 'N', long)]
    namespace: Option<String>,

    /// Populate the project with example files.
    #[arg(short = 'p', long)]
    populate: bool,

    /// Categories to add to your typst.toml.
    #[arg(short = 'C', long)]
    #[clap(value_delimiter = ',')]
    categories: Option<Vec<String>>,

    /// Disciplines to add to your typst.toml.
    #[arg(long)]
    #[clap(value_delimiter = ',')]
    disciplines: Option<Vec<String>>,

    /// Path to a template file to use.
    #[arg(long)]
    template_path: Option<String>,

    /// Entrypoint for the template.
    #[arg(long)]
    template_entrypoint: Option<String>,

    /// Thumbnail for the template.
    #[arg(long)]
    template_thumbnail: Option<String>,
}

/// Arguments for the `link` command.
/// This command links a local project to the UTPM package directory.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "link")]
pub struct LinkArgs {
    /// Force the copy of the directory or creation of the symlink.
    #[arg(short, long)]
    pub force: bool,

    /// Create a symlink instead of copying the project files.
    #[arg(short, long)]
    pub no_copy: bool,
}

/// Arguments for the `list` and `tree` commands.
/// These commands display the packages in the local storage.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(any(feature = "list", feature = "tree"))]
pub struct ListTreeArgs {
    /// List all packages, including those in the `@preview` namespace.
    #[arg(short, long)]
    pub all: bool,

    /// Specify subdirectories to include in the list.
    #[arg(short, long, num_args = 1..)]
    pub include: Option<Vec<String>>,

    /// Display the packages as a tree. Only works with text output.
    #[arg(short, long)]
    pub tree: bool,
}

/// Arguments for the `bump` command.
/// This command bump the version of your package
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "bump")]
pub struct BumpArgs {
    /// The tag to look at when you bump other files.
    /// If the file is written in markdown or html, it will looks into the code to find `<tag>0.1.0</tag>`
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Files to include in the list. (typst.toml is already included)
    #[arg(short, long, num_args = 1..)]
    pub include: Vec<String>,

    pub new_version: String,
}

/// Arguments for the `publish` command.
/// This command publishes a package to the typst universe.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "publish")]
pub struct PublishArgs {
    /// Path to the project to publish. Defaults to the current directory.
    #[arg()]
    path: Option<PathBuf>,

    /// Use .ignore files to filter packaged files.
    #[arg(short = 'i', default_value_t = false)]
    ignore: bool,

    /// Use .gitignore files to filter packaged files.
    #[arg(short = 'g', default_value_t = true)]
    git_ignore: bool,

    /// Use .typstignore files to filter packaged files.
    #[arg(short = 't', default_value_t = true)]
    typst_ignore: bool,

    /// Use global .gitignore to filter packaged files.
    #[arg(short = 'G', default_value_t = true)]
    git_global_ignore: bool,

    /// Use .git/info/exclude files to filter packaged files.
    #[arg(short = 'x', default_value_t = true)]
    git_exclude: bool,

    /// Bypass the warning prompts.
    #[arg(default_value_t = false)]
    bypass_warning: bool,

    /// Path to a custom ignore file.
    #[arg(short = 'c')]
    custom_ignore: Option<PathBuf>,

    /// Specify a message for the new commit.
    #[arg(short = 'm')]
    message: Option<String>,

    /// Prepare the package for publishing without creating a pull request.
    #[arg(short = 'p', default_value_t = false)]
    prepare_only: bool,
}

/// Arguments for the `generate` command.
/// This command generates shell completion scripts.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "generate")]
pub struct GenerateArgs {
    /// The shell to generate a completion script for.
    #[arg(value_enum)]
    generator: Shell,
}

/// Arguments for the `clone` command.
/// This command clones a package from the typst universe or a local directory.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "clone")]
pub struct CloneArgs {
    /// The name of the package to clone (e.g., `@preview/example:1.0.0`).
    #[arg()]
    pub package: String,

    /// The directory to clone the package into.
    #[arg()]
    pub path: Option<PathBuf>,

    /// Download the package to the cache without copying it to the target directory.
    #[arg(short = 'd')]
    pub download_only: bool,

    /// Force cloning even if the destination path is not empty.
    #[arg(short = 'f')]
    pub force: bool,

    /// Force re-downloading the package even if it exists in the cache.
    #[arg(short = 'r')]
    pub redownload: bool,

    /// Create a symlink to the cloned package instead of copying.
    #[arg(short = 's')]
    pub symlink: bool,
}

/// Arguments for the `unlink` command.
/// This command removes a package from the local storage.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "unlink")]
pub struct UnlinkArgs {
    /// The name of the package to unlink.
    package: String,

    /// Confirm the deletion of the package directory without a prompt.
    #[arg(short, long)]
    yes: bool,
}

/// Arguments for the `bulk-delete` command.
/// This command removes multiple packages at once.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "bulk_delete")]
pub struct BulkDeleteArgs {
    /// A comma-separated list of package names to delete (e.g., `mypackage:1.0.0,another:2.1.0`).
    #[clap(value_delimiter = ',')]
    names: Vec<String>,

    /// The namespace to bulk-delete packages from.
    #[arg(short, long)]
    namespace: Option<String>,
}

/// Arguments for the `install` command.
/// This command installs dependencies from the manifest or a given URL.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "install")]
pub struct InstallArgs {
    /// URL or path to a specific package to install. If not provided, installs dependencies from the manifest.
    #[arg(num_args = 1..)]
    pub url: Option<String>,

    /// Force link commands for all dependencies.
    #[arg(short, long, default_value_t = false)]
    pub force: bool,
}

/// Arguments for the `delete` command.
/// This command removes dependencies from the manifest.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "delete")]
pub struct DeleteArgs {
    /// URIs of dependencies to remove from the manifest.
    #[clap(short, long, num_args = 1..)]
    pub uri: Vec<String>,
}

#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "sync")]
/// Arguments for the `sync` command.
/// This command synchronise package from the remote or the local .
/// Can't check remote unofficial packages.
pub struct SyncArgs {
    /// Files to sync packages. Default to all files
    #[clap(short, long)]
    pub files: Vec<String>,

    /// Only check if they are new versions and write them on the file itself
    #[clap(short, long)]
    pub check_only: bool,
}

#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "get")]
/// Arguments for the `get` command.
/// This command gets from the remote the package you research.
/// By default: Give you all packages available
pub struct GetArgs {
    /// Files to sync packages. Default to all files
    pub packages: Vec<String>,
}

/// Arguments for the `add` command.
/// This command adds dependencies to the manifest.
#[derive(Parser, Clone, Debug, PartialEq)]
#[cfg(feature = "add")]
pub struct AddArgs {
    /// The URL or path of the repository to add as a dependency.
    pub uri: Vec<String>,
}

/// An enumeration of subcommands for managing local packages.
#[derive(Subcommand, Debug, PartialEq)]
#[cfg(any(
    feature = "tree",
    feature = "list",
    feature = "path",
    feature = "unlink",
    feature = "bulk_delete",
    feature = "get"
))]
pub enum Packages {
    /// [DEPRECATED] Display packages as a tree. Use `list --tree` instead.
    #[command(visible_alias = "t")]
    #[cfg(feature = "tree")]
    #[command(about = "[DEPRECIATED] Use list with --tree.")]
    Tree(ListTreeArgs),

    /// List all packages in your local storage.
    #[command(visible_alias = "l")]
    #[cfg(feature = "list")]
    List(ListTreeArgs),

    /// Display the path to the typst packages folder.
    #[command(visible_alias = "p")]
    #[cfg(feature = "path")]
    Path,

    /// Delete a package from your local storage.
    #[command(visible_alias = "u")]
    #[cfg(feature = "unlink")]
    Unlink(UnlinkArgs),

    /// Delete multiple packages or a whole namespace at once.
    #[command(visible_alias = "bd")]
    #[cfg(feature = "bulk_delete")]
    BulkDelete(BulkDeleteArgs),

    /// Get specific/all package from the remote
    #[command(visible_alias = "g")]
    #[cfg(feature = "get")]
    Get(GetArgs),
}

/// An enumeration of subcommands for managing the project workspace.
#[derive(Subcommand, Debug, PartialEq)]
#[cfg(any(
    feature = "link",
    feature = "init",
    feature = "install",
    feature = "add",
    feature = "delete",
    feature = "init",
    feature = "publish",
    feature = "bump",
    feature = "sync",
    feature = "clone"
))]
pub enum Workspace {
    /// Link the current project to the local package directory.
    #[command(visible_alias = "l")]
    #[cfg(feature = "link")]
    Link(LinkArgs),

    /// Install all dependencies from the `typst.toml` manifest.
    #[command(visible_alias = "i")]
    #[cfg(feature = "install")]
    Install(InstallArgs),

    /// Add dependencies to the manifest and then install them.
    #[command(visible_alias = "a")]
    #[cfg(feature = "add")]
    Add(AddArgs),

    /// Delete dependencies from the manifest.
    #[command(visible_alias = "d")]
    #[cfg(feature = "delete")]
    Delete(DeleteArgs),

    /// Create a new `typst.toml` manifest for a project.
    #[command(visible_alias = "n")]
    #[cfg(feature = "init")]
    Init(InitArgs),

    // Publish your package to the typst universe. (WIP)
    // #[command(visible_alias = "p")]
    // #[cfg(feature = "publish")]
    // Publish(PublishArgs),
    /// Clone a package from the typst universe or a local directory.
    #[command()]
    #[cfg(feature = "clone")]
    Clone(CloneArgs),

    /// Bump all version of your package into an other.
    #[command()]
    #[cfg(feature = "bump")]
    Bump(BumpArgs),

    /// Synchronise all your dependencies into their last version.
    #[command()]
    #[cfg(feature = "sync")]
    Sync(SyncArgs),
}

/// The main command-line interface for UTPM.
#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Subcommands for managing the project workspace.
    #[command(subcommand)]
    #[command(visible_alias = "ws")]
    #[cfg(any(
        feature = "link",
        feature = "init",
        feature = "install",
        feature = "add",
        feature = "delete",
        feature = "init",
        feature = "publish",
        feature = "bump",
        feature = "sync",
        feature = "clone"
    ))]
    Workspace(Workspace),

    /// Subcommands for managing local packages.
    #[command(subcommand)]
    #[command(visible_alias = "pkg")]
    #[cfg(any(
        feature = "tree",
        feature = "list",
        feature = "path",
        feature = "unlink",
        feature = "bulk_delete",
        feature = "get"
    ))]
    Packages(Packages),

    /// Generate shell completion scripts.
    #[command(visible_alias = "g")]
    #[cfg(feature = "generate")]
    Generate(GenerateArgs),
}

// maybe users want to also see the PKG_VERSION?
#[cfg(feature = "nightly")]
const VERSION: &str = build::COMMIT_HASH;

#[cfg(not(feature = "nightly"))]
const VERSION: &str = build::PKG_VERSION;

/// An unofficial typst package manager for your projects.
#[derive(Parser, Debug, PartialEq)]
#[command(author = "Thumuss & typst-community", version = VERSION)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging for debugging purposes.
    #[arg(short = 'v', long, global = true, value_enum)]
    pub verbose: Option<Level>,

    /// The output format for command results.
    #[arg(short = 'o', long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,

    /// If you don't want to write anything on your disk.
    #[arg(default_value_t = false, global = true, long, short = 'D')]
    pub dry_run: bool,
}
