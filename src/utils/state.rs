use serde::Serialize;
use thiserror::Error as TError;
use typst_project::manifest::{
    author::{ParseAuthorError, ParseWebsiteError},
    ident::ParseIdentError,
    license::ParseLicenseError,
    Error as ManifestError,
};

pub type Result<T> = anyhow::Result<T, UtpmError>;

use serde::ser::{SerializeStruct, Serializer};

#[derive(Debug, TError)]
pub enum UtpmError {
    #[error("Semantic version error: {0}")]
    SemVer(String),

    #[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
    #[error("Git error: {0}")]
    Git(String),

    #[cfg(any(feature = "init", feature = "unlink"))]
    #[error("Inquire error: {0}")]
    Questions(String),

    #[error("IO error: {0}")]
    IO(String),

    #[error("Manifest error: {0}")]
    General(String),

    #[error("Author parse error: {0}")]
    Author(String),

    #[error("Website parse error: {0}")]
    Website(String),

    #[error("Email parse error: {0}")]
    Email(String),

    #[error("GitHub handle parse error: {0}")]
    GithubHandle(String),

    #[error("Identifier parse error: {0}")]
    Ident(String),

    #[error("License parse error: {0}")]
    License(String),

    #[error("TOML serialization error: {0}")]
    Serialize(String),

    #[error("TOML deserialization error: {0}")]
    Deserialize(String),

    #[cfg(any(feature = "publish"))]
    #[error("Ignore crate error: {0}")]
    Ignore(String),

    #[cfg(any(feature = "publish"))]
    #[error("Octocrab error: {0}")]
    OctoCrab(String),

    #[error("Typst project error: {0}")]
    Project(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    // Unit variantes (sans champ)
    #[error("Missing namespace or package name.")]
    Namespace,

    #[error("Missing configuration file.")]
    ConfigFile,

    #[error("Couldn't find the current directory.")]
    CurrentDir,

    #[error("Failed to create directory.")]
    CreationDir,

    #[error("Could not determine home directory.")]
    HomeDir,

    #[error("Missing manifest.")]
    Manifest,

    #[error("Not enough arguments provided.")]
    NotEnoughArgs,

    #[error("Can't extract your package. Example of a package: @namespace/package:1.0.0")]
    PackageNotValid,

    #[error("This package doesn't exist. Verify on https://typst.app/universe to see if the package exist and/or the version is correct.")]
    PackageNotExist,

    #[error("We founded content. Cancelled the operation.")]
    ContentFound,

    #[error("{2} Package {0} with version {1} already exist.")]
    AlreadyExist(String, semver::Version, String),
}

impl From<std::io::Error> for UtpmError {
    fn from(err: std::io::Error) -> Self {
        UtpmError::IO(err.to_string())
    }
}

impl From<toml::ser::Error> for UtpmError {
    fn from(err: toml::ser::Error) -> Self {
        UtpmError::Serialize(err.to_string())
    }
}

impl From<toml::de::Error> for UtpmError {
    fn from(err: toml::de::Error) -> Self {
        UtpmError::Deserialize(err.to_string())
    }
}

impl From<semver::Error> for UtpmError {
    fn from(err: semver::Error) -> Self {
        UtpmError::SemVer(err.to_string())
    }
}

#[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
impl From<git2::Error> for UtpmError {
    fn from(err: git2::Error) -> Self {
        UtpmError::Git(err.to_string())
    }
}

#[cfg(any(feature = "init", feature = "unlink"))]
impl From<inquire::InquireError> for UtpmError {
    fn from(err: inquire::InquireError) -> Self {
        UtpmError::Questions(err.to_string())
    }
}

#[cfg(any(feature = "publish"))]
impl From<ignore::Error> for UtpmError {
    fn from(err: ignore::Error) -> Self {
        UtpmError::Ignore(err.to_string())
    }
}

#[cfg(any(feature = "publish"))]
impl From<octocrab::Error> for UtpmError {
    fn from(err: octocrab::Error) -> Self {
        UtpmError::OctoCrab(err.to_string())
    }
}

impl From<ManifestError> for UtpmError {
    fn from(err: ManifestError) -> Self {
        UtpmError::Project(err.to_string())
    }
}

impl From<ParseAuthorError> for UtpmError {
    fn from(err: ParseAuthorError) -> Self {
        UtpmError::Author(err.to_string())
    }
}

impl From<ParseWebsiteError> for UtpmError {
    fn from(err: ParseWebsiteError) -> Self {
        UtpmError::Website(err.to_string())
    }
}

impl From<ParseIdentError> for UtpmError {
    fn from(err: ParseIdentError) -> Self {
        UtpmError::Ident(err.to_string())
    }
}

impl From<ParseLicenseError> for UtpmError {
    fn from(err: ParseLicenseError) -> Self {
        UtpmError::License(err.to_string())
    }
}

// Custom serialize impl
impl Serialize for UtpmError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let mut st = serializer.serialize_struct("UtpmError", 2)?;
        st.serialize_field("type", self.variant_name())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

impl UtpmError {
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
            Author(_) => "Author",
            Website(_) => "Website",
            Email(_) => "Email",
            GithubHandle(_) => "GithubHandle",
            Ident(_) => "Ident",
            License(_) => "License",
            Serialize(_) => "Serialize",
            Deserialize(_) => "Deserialize",
            #[cfg(any(feature = "publish"))]
            Ignore(_) => "Ignore",
            #[cfg(any(feature = "publish"))]
            OctoCrab(_) => "OctoCrab",
            Project(_) => "Project",
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
        }
    }
}

impl From<anyhow::Error> for UtpmError {
    fn from(err: anyhow::Error) -> Self {
        UtpmError::Unknown(err.to_string())
    }
}
