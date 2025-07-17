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
    SemVer(#[from]  semver::Error),

    #[cfg(any(feature = "install", feature = "clone", feature = "publish"))]
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[cfg(any(feature = "init", feature = "unlink"))]
    #[error("Inquire error: {0}")]
    Questions(#[from] inquire::InquireError),

    #[error("IO error: {0}")]
    IO (#[from] std::io::Error),

    #[error("We can't rebase (for now)")]
    Rebase,

    #[error("General error: {0}")]
    General(String),

    #[cfg(feature = "output_json")]
    #[error("Can't parse to json: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[cfg(feature = "output_hjson")]
    #[error("Can't parse to hjson: {0}")]
    HJsonParse(#[from] serde_hjson::Error),

    #[cfg(feature = "output_yaml")]
    #[error("Can't parse to yaml: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Author parse error: {0}")]
    Author(#[from] ParseAuthorError),

    #[error("Website parse error: {0}")]
    Website(#[from] ParseWebsiteError),

    #[error("Email parse error: {0}")]
    Email(String),

    #[error("GitHub handle parse error: {0}")]
    GithubHandle(String),

    #[error("Identifier parse error: {0}")]
    Ident(#[from] ParseIdentError),

    #[error("License parse error: {0}")]
    License(#[from] ParseLicenseError),

    #[error("TOML serialization error: {0:?}")]
    Serialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    Deserialize(#[from] toml::de::Error),

    #[cfg(any(feature = "publish"))]
    #[error("Ignore crate error: {0}")]
    Ignore(#[from] ignore::Error),

    #[cfg(any(feature = "publish"))]
    #[error("Octocrab error: {0}")]
    OctoCrab(#[from] octocrab::Error),

    #[error("Typst project error: {0}")]
    Project(#[from] ManifestError),

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

    #[error("No URI were found. Please check your typst.toml")]
    NoURIFound,


    #[error(transparent)]
    Other(#[from] anyhow::Error)
}


// Custom serialize impl
impl Serialize for UtpmError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where S: Serializer,
    {
        let mut st = serializer.serialize_struct("UtpmError", 2)?;
        st.serialize_field("type", self.variant_name())?;
        st.serialize_field("message", &self.to_string())?;
        return st.end();
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
            Other(_) => "Other",
            Rebase => "Rebase",
            JsonParse(_) => "JSONParse",
            HJsonParse(_) => "HJSONParse",
            YamlParse(_) => "YamlParse",
            NoURIFound => "NoURIFound",
        }
    }
}
