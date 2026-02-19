//! Linker: This module dynamically links all the command modules.
//! Each command is a separate module, conditionally compiled based on feature flags.

pub mod bump;
pub mod clone;
pub mod generate;
pub mod get;
pub mod init;
pub mod install;
pub mod link;
pub mod list;
pub mod metadata;
pub mod package_path;
pub mod publish;
pub mod sync;
pub mod unlink;

use std::path::PathBuf;
use std::str::FromStr;

use clap::{Parser, Subcommand};
use clap_complete::Shell;
use tracing::Level;
use typst_syntax::package::{PackageVersion, VersionBound};

use crate::build;
use crate::utils::output::OutputFormat;

fn parse_eco<T>(s: &str) -> Result<T, String>
where
    T: FromStr<Err = ecow::EcoString>,
{
    T::from_str(s).map_err(Into::into)
}

/// Arguments for the `init` command.
/// This command initializes a new `typst.toml` manifest file in the current directory.
#[derive(Parser, Clone, Debug, PartialEq)]
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
    #[arg(short = 'V', long, default_value_t=PackageVersion::from_str("1.0.0").unwrap(), value_parser=parse_eco::<PackageVersion>)]
    version: PackageVersion,

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
    #[arg(short, long, value_parser=parse_eco::<VersionBound>)]
    compiler: Option<VersionBound>,

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
pub struct LinkArgs {
    /// Force the copy of the directory or creation of the symlink.
    #[arg(short, long)]
    pub force: bool,

    /// Create a symlink instead of copying the project files.
    #[arg(short, long)]
    pub no_copy: bool,

    /// Namespace
    pub namespace: Option<String>,

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
}

/// Arguments for the `list` and `tree` commands.
/// These commands display the packages in the local storage.
#[derive(Parser, Clone, Debug, PartialEq)]
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
    #[arg(long, default_value_t = false)]
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
pub struct GenerateArgs {
    /// The shell to generate a completion script for.
    #[arg(value_enum)]
    generator: Shell,
}

/// Arguments for the `clone` command.
/// This command clones a package from the typst universe or a local directory.
#[derive(Parser, Clone, Debug, PartialEq)]
pub struct CloneArgs {
    /// The package to clone.
    ///
    /// Format: @namespace/package:version
    /// Example: @preview/example:1.0.0
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
pub struct UnlinkArgs {
    /// The package to unlink.
    ///
    /// Formats accepted:
    ///   - Full: @namespace/package:1.0.0
    ///   - Package: @namespace/package (removes all versions)
    ///   - Namespace: @namespace (removes entire namespace)
    package: String,

    /// Confirm the deletion of the package directory without a prompt.
    #[arg(short, long)]
    yes: bool,
}

/// Arguments for the `install` command.
/// This command installs a package from a git repository, not from Typst Universe.
/// You will require to have git install on your machine.
#[derive(Parser, Clone, Debug, PartialEq)]
pub struct InstallArgs {
    /// URL or path to a specific package to install.
    #[arg(num_args = 1..)]
    pub url: String,

    /// The namespace you want to put your installed package. Default to local
    #[arg(short, long)]
    pub namespace: Option<String>,
}

#[derive(Parser, Clone, Debug, PartialEq)]
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
/// Arguments for the `get` command.
/// This command gets package information from Typst Universe.
/// By default: Lists all available packages.
pub struct GetArgs {
    /// Package names to query (e.g., @preview/example).
    /// Leave empty to list all packages.
    pub packages: Vec<String>,
}

/// Arguments for the `metadata` command.
/// This command extracts metadata from typst.toml.
#[derive(Parser, Clone, Debug, PartialEq)]
pub struct MetadataArgs {
    /// Path to the project directory. Defaults to the current directory.
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Specific field to extract (e.g., name, version, authors).
    /// If not specified, all metadata will be displayed.
    #[arg(short, long)]
    pub field: Option<String>,
}

/// An enumeration of subcommands for managing local packages.
#[derive(Subcommand, Debug, PartialEq)]
pub enum PackagesArgs {
    /// List all packages in your local storage.
    #[command(visible_alias = "l")]
    List(ListTreeArgs),

    /// Display the path to the typst packages folder.
    #[command(visible_alias = "p")]
    Path,

    /// Delete a package from your local storage.
    #[command(visible_alias = "u")]
    Unlink(UnlinkArgs),

    /// Get specific/all packages from the remote.
    #[command(visible_alias = "g")]
    Get(GetArgs),

    /// Install a package from a git repository into a namespace.
    #[command(visible_alias = "i")]
    Install(InstallArgs),
}

/// An enumeration of subcommands for managing the project project.
#[derive(Subcommand, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum ProjectArgs {
    /// Link the current project to the local package directory.
    #[command(visible_alias = "l")]
    Link(LinkArgs),

    /// Create a new `typst.toml` manifest for a project.
    #[command(visible_alias = "n")]
    Init(InitArgs),

    /// Publish your package to the Typst Universe.
    #[command(visible_alias = "p")]
    Publish(PublishArgs),

    /// Clone a package from the Typst Universe or a local directory.
    #[command()]
    #[command(visible_alias = "c")]
    Clone(CloneArgs),

    /// Bump the version of your package in `typst.toml` and other project files.
    #[command()]
    #[command(visible_alias = "b")]
    Bump(BumpArgs),

    /// Synchronise all your dependencies into their last version.
    #[command()]
    #[command(visible_alias = "s")]
    Sync(SyncArgs),

    /// Get metadata from typst.toml for use in scripts.
    #[command()]
    #[command(visible_alias = "m")]
    Metadata(MetadataArgs),
}

/// The main command-line interface for UTPM.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Subcommands for managing the current project.
    #[command(subcommand)]
    #[command(visible_alias = "prj")]
    Project(ProjectArgs),

    /// Subcommands for managing local packages.
    #[command(subcommand)]
    #[command(visible_alias = "pkg")]
    Packages(PackagesArgs),

    /// Generate shell completion scripts.
    #[command(visible_alias = "g")]
    Generate(GenerateArgs),
}

/// An unofficial typst package manager for your projects.
#[derive(Parser, Debug, PartialEq)]
#[command(author = "Thumuss & typst-community", version = build::PKG_VERSION)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose logging for debugging purposes.
    ///
    /// Levels: error, warn, info (default), debug, trace
    /// Example: utpm -v trace prj link
    #[arg(
        default_value = "info",
        short = 'v',
        long,
        global = true,
        env = "UTPM_DEBUG",
        value_enum
    )]
    pub verbose: Level,

    /// The output format for command results.
    ///
    /// Formats: text (default), json, yaml, toml, hjson
    /// Example: utpm -o json pkg list
    #[arg(default_value_t = OutputFormat::Text, short = 'o', long, global = true, value_enum)]
    pub output_format: OutputFormat,

    /// Preview changes without writing to disk (dry-run mode).
    ///
    /// Useful for testing commands before execution.
    /// Example: utpm --dry-run prj link
    #[arg(default_value_t = false, short = 'D', long, global = true)]
    pub dry_run: bool,
}
