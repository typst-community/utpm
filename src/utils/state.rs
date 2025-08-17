use serde::Serialize;
use thiserror::Error as TError;

/// A specialized `Result` type for UTPM operations.
pub type Result<T> = anyhow::Result<T, UtpmError>;

use serde::ser::{SerializeStruct, Serializer};
use typst_syntax::package::PackageVersion;

/// The error type for UTPM operations.
///
/// This enum consolidates all possible errors that can occur within the application,
/// providing a single, consistent error handling mechanism.
#[derive(Debug, TError)]
pub enum UtpmError {
    /// A semantic versioning error.
    #[error("Semantic version error: {0}")]
    SemVer(#[from] semver::Error),

    /// A git-related error.
    #[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
    #[error("Git error: {0}")]
    Git(String),

    /// A git-related error.
    #[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
    #[error("We didn't find git on your path. Try to add it.")]
    GitNotFound,

    /// An error from the `inquire` crate, used for interactive prompts.
    #[cfg(any(feature = "init", feature = "unlink"))]
    #[error("Inquire error: {0}")]
    Questions(#[from] inquire::InquireError),

    /// An I/O error.
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// An error indicating that a git rebase is required but not yet supported.
    #[error("We can't rebase (for now)")]
    Rebase,

    /// A general-purpose error.
    #[error("General error: {0}")]
    General(String),

    /// An error during JSON serialization or deserialization.
    #[cfg(feature = "output_json")]
    #[error("Can't parse to json: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// An error during Hjson serialization or deserialization.
    #[cfg(feature = "output_hjson")]
    #[error("Can't parse to hjson: {0}")]
    HJsonParse(#[from] serde_hjson::Error),

    /// An error during YAML serialization or deserialization.
    #[cfg(feature = "output_yaml")]
    #[error("Can't parse to yaml: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    /// An error parsing an email address.
    #[error("Email parse error: {0}")]
    Email(String),

    /// An error parsing a GitHub handle.
    #[error("GitHub handle parse error: {0}")]
    GithubHandle(String),

    /// An error during TOML serialization.
    #[error("TOML serialization error: {0:?}")]
    Serialize(#[from] toml::ser::Error),

    /// An error during TOML deserialization.
    #[error("TOML deserialization error: {0}")]
    Deserialize(#[from] toml::de::Error),

    /// An error during TOML deserialization.
    #[error("TOML deserialization error: {0}")]
    DeserializeMut(#[from] toml_edit::TomlError),

    /// An error from the `ignore` crate.
    #[cfg(any(feature = "publish", feature = "sync"))]
    #[error("Ignore crate error: {0}")]
    Ignore(#[from] ignore::Error),

    /// An error from the `octocrab` crate for GitHub API interactions.
    #[cfg(any(feature = "publish"))]
    #[error("Octocrab error: {0}")]
    OctoCrab(#[from] octocrab::Error),

    /// An unknown or unexpected error.
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// An error for a missing namespace or package name.
    #[error("Missing namespace or package name.")]
    Namespace,

    /// An error for a missing configuration file.
    #[error("Missing configuration file.")]
    ConfigFile,

    /// An error when the current directory cannot be determined.
    #[error("Couldn't find the current directory.")]
    CurrentDir,

    /// An error when a directory fails to be created.
    #[error("Failed to create directory.")]
    CreationDir,

    /// An error when the home directory cannot be determined.
    #[error("Could not determine home directory.")]
    HomeDir,

    /// An error for a missing `typst.toml` manifest.
    #[error("Missing manifest.")]
    Manifest,

    /// An error for not enough arguments provided to a command.
    #[error("Not enough arguments provided.")]
    NotEnoughArgs,

    /// An error for an invalid package string.
    #[error("Can't extract your package. Example of a package: @namespace/package:1.0.0")]
    PackageNotValid,

    /// An error for an invalid package string.
    #[error("Package didn't match, the name or the version is incorrect.")]
    PackageFormatError,

    #[error("There is no files in the new package. You should change your ignored files.")]
    NoFiles,

    #[error("Can't find `typst.toml` file in {0}. Did you omit it in your ignored files?")]
    OmitedTypstFile(String),

    #[error("Can't find {0} file in {1}. Did you omit it in your ignored files?")]
    OmitedEntryfile(String, String),

    /// An error when a specified package does not exist.
    #[error(
        "This package doesn't exist. Verify on https://typst.app/universe to see if the package exist and/or the version is correct."
    )]
    PackageNotExist,

    /// An error when content is found in a directory that should be empty.
    #[error("We founded content. Cancelled the operation.")]
    ContentFound,

    /// An error when a package to be linked already exists.
    #[error("{2} Package {0} with version {1} already exist.")]
    AlreadyExist(String, PackageVersion, String),

    /// An error when no URIs are provided for a command that requires them.
    #[error("No URI were found. Please check your typst.toml")]
    NoURIFound,

    #[error("Reqwest Error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("FromUTF8 Error: {0}")]
    FromUTF8Error(#[from] std::string::FromUtf8Error),

    /// A wrapper for any other error.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Custom `Serialize` implementation for `UtpmError`.
// This allows errors to be serialized into structured formats like JSON.
impl Serialize for UtpmError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut st = serializer.serialize_struct("UtpmError", 2)?;
        st.serialize_field("type", self.variant_name())?;
        st.serialize_field("message", &self.to_string())?;
        return st.end();
    }
}

impl UtpmError {
    /// Returns the string representation of the error variant.
    fn variant_name(&self) -> &'static str {
        use UtpmError::*;
        match self {
            SemVer(_) => "SemVer",
            #[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
            Git(_) => "Git",
            #[cfg(any(feature = "init", feature = "unlink"))]
            Questions(_) => "Questions",
            IO(_) => "IO",
            General(_) => "General",
            Email(_) => "Email",
            GithubHandle(_) => "GithubHandle",
            Serialize(_) => "Serialize",
            Deserialize(_) => "Deserialize",
            DeserializeMut(_) => "DeserializeMut",
            #[cfg(any(feature = "publish", feature = "sync"))]
            Ignore(_) => "Ignore",
            #[cfg(any(feature = "publish"))]
            OctoCrab(_) => "OctoCrab",
            Unknown(_) => "Unknown",
            Namespace => "Namespace",
            ConfigFile => "ConfigFile",
            CurrentDir => "CurrentDir",
            CreationDir => "CreationDir",
            HomeDir => "HomeDir",
            Manifest => "Manifest",
            NotEnoughArgs => "NotEnoughArgs",
            PackageNotValid => "PackageNotValid",
            PackageNotExist => "PackageNotExist",
            ContentFound => "ContentFound",
            AlreadyExist(_, _, _) => "AlreadyExist",
            Other(_) => "Other",
            Rebase => "Rebase",
            JsonParse(_) => "JSONParse",
            #[cfg(feature = "output_hjson")]
            HJsonParse(_) => "HJSONParse",
            #[cfg(feature = "output_yaml")]
            YamlParse(_) => "YamlParse",
            NoURIFound => "NoURIFound",
            ReqwestError(_) => "ReqwestError",
            FromUTF8Error(_) => "FromUTF8Error",
            GitNotFound => "GitNotFound",
            PackageFormatError => "PackageFormatError",
            NoFiles => "NoFiles",
            OmitedTypstFile(_) => "OmitedTypstFile",
            OmitedEntryfile(_, _) => "OmitedEntryfile",
        }
    }
}
