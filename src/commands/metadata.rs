use serde::Serialize;
use tracing::instrument;

use crate::{
    utils::{
        output::{OutputFormat, get_output_format},
        paths::get_current_dir,
        state::Result,
        try_find,
    },
    utpm_log,
};

use super::MetadataArgs;

/// Metadata information extracted from typst.toml
#[derive(Serialize, Clone, Debug)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disciplines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

/// Get metadata from typst.toml
#[instrument(skip(cmd))]
pub async fn run(cmd: &MetadataArgs) -> Result<bool> {
    utpm_log!(trace, "executing metadata command");

    // Get the current directory or use provided path
    let curr = cmd
        .path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| get_current_dir().unwrap().to_string_lossy().to_string());

    // Load the manifest
    let config = try_find(&curr)?;

    // Extract metadata
    let metadata = PackageMetadata {
        name: config.package.name.to_string(),
        version: config.package.version.to_string(),
        entrypoint: config.package.entrypoint.to_string(),
        authors: config
            .package
            .authors
            .iter()
            .map(|a| a.to_string())
            .collect(),
        license: config.package.license.map(|l| l.to_string()),
        description: config.package.description.map(|d| d.to_string()),
        repository: config.package.repository.map(|r| r.to_string()),
        homepage: config.package.homepage.map(|h| h.to_string()),
        keywords: if config.package.keywords.is_empty() {
            None
        } else {
            Some(
                config
                    .package
                    .keywords
                    .iter()
                    .map(|k| k.to_string())
                    .collect(),
            )
        },
        categories: if config.package.categories.is_empty() {
            None
        } else {
            Some(
                config
                    .package
                    .categories
                    .iter()
                    .map(|c| c.to_string())
                    .collect(),
            )
        },
        disciplines: if config.package.disciplines.is_empty() {
            None
        } else {
            Some(
                config
                    .package
                    .disciplines
                    .iter()
                    .map(|d| d.to_string())
                    .collect(),
            )
        },
        compiler: config.package.compiler.map(|v| v.to_string()),
        exclude: crate::utils::specs::Extra::from(config.tool)
            .exclude
            .map(|v| v.iter().cloned().collect::<Vec<_>>()),
    };

    // Handle specific field request
    if let Some(field) = &cmd.field {
        let value = match field.as_str() {
            "name" => Some(metadata.name.clone()),
            "version" => Some(metadata.version.clone()),
            "entrypoint" => Some(metadata.entrypoint.clone()),
            "authors" => Some(metadata.authors.join(", ")),
            "license" => metadata.license.clone(),
            "description" => metadata.description.clone(),
            "repository" => metadata.repository.clone(),
            "homepage" => metadata.homepage.clone(),
            "keywords" => metadata.keywords.as_ref().map(|k| k.join(", ")),
            "categories" => metadata.categories.as_ref().map(|c| c.join(", ")),
            "disciplines" => metadata.disciplines.as_ref().map(|d| d.join(", ")),
            "compiler" => metadata.compiler.clone(),
            "exclude" => metadata.exclude.as_ref().map(|e| e.join(", ")),
            _ => {
                utpm_log!(error, "Unknown field: {}", field);
                return Ok(false);
            },
        };

        if let Some(val) = value {
            // For single field extraction, output plain text for all formats
            // TOML/YAML/HJSON don't support serializing plain strings without a key
            println!("{}", val);
        } else {
            utpm_log!(info, "Field '{}' is not set", field);
        }
    } else {
        // Output all metadata
        match get_output_format() {
            OutputFormat::Text => {
                println!("Package Metadata:");
                println!("  Name: {}", metadata.name);
                println!("  Version: {}", metadata.version);
                println!("  Entrypoint: {}", metadata.entrypoint);
                println!("  Authors: {}", metadata.authors.join(", "));
                if let Some(license) = &metadata.license {
                    println!("  License: {}", license);
                }
                if let Some(description) = &metadata.description {
                    println!("  Description: {}", description);
                }
                if let Some(repository) = &metadata.repository {
                    println!("  Repository: {}", repository);
                }
                if let Some(homepage) = &metadata.homepage {
                    println!("  Homepage: {}", homepage);
                }
                if let Some(keywords) = &metadata.keywords {
                    println!("  Keywords: {}", keywords.join(", "));
                }
                if let Some(categories) = &metadata.categories {
                    println!("  Categories: {}", categories.join(", "));
                }
                if let Some(disciplines) = &metadata.disciplines {
                    println!("  Disciplines: {}", disciplines.join(", "));
                }
                if let Some(compiler) = &metadata.compiler {
                    println!("  Compiler: {}", compiler);
                }
                if let Some(exclude) = &metadata.exclude {
                    println!("  Exclude: {}", exclude.join(", "));
                }
            },
            #[cfg(feature = "output_json")]
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
            },
            #[cfg(feature = "output_yaml")]
            OutputFormat::Yaml => {
                println!("{}", serde_yaml::to_string(&metadata).unwrap());
            },
            OutputFormat::Toml => {
                println!("{}", toml::to_string_pretty(&metadata).unwrap());
            },
            #[cfg(feature = "output_hjson")]
            OutputFormat::Hjson => {
                println!("{}", serde_hjson::to_string(&metadata).unwrap());
            },
        }
    }

    Ok(true)
}
