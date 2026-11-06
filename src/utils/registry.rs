//! Access to the Typst Universe package registry.

use std::collections::HashMap;

use fmt_derive::Display;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;
use toml::to_string_pretty;

use crate::{build, utils::state::Result};

#[derive(Debug, Deserialize, Serialize, Clone, Display)]
#[display("{}", to_string_pretty(self).unwrap())]
pub struct RawPackage {
    pub name: String,
    pub version: String,
    pub entrypoint: String,
    pub authors: Vec<String>,
    pub license: String,
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub disciplines: Option<Vec<String>>,
    pub compiler: Option<String>,
    pub exclude: Option<Vec<String>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
}

/// Fetches all packages from the Typst Universe registry.
///
/// # Returns
/// A vector of all available packages from `packages.typst.org`.
///
/// # Errors
/// Returns an error if the HTTP request fails or the response cannot be parsed.
pub async fn get_all_packages() -> Result<Vec<RawPackage>> {
    let client = reqwest::Client::new();

    let packages: Vec<RawPackage> = client
        .get("https://packages.typst.org/preview/index.json")
        .header(
            reqwest::header::USER_AGENT,
            format!("utpm/{}", build::PKG_VERSION),
        )
        .send()
        .await?
        .json::<Vec<RawPackage>>()
        .await?;
    Ok(packages)
}

/// Fetches all packages and creates a hashmap for lookup by name or name:version.
///
/// The index is downloaded at most once per process; later calls return the same map.
///
/// # Returns
/// A hashmap where keys can be either:
/// - Package name (e.g., "mypackage") - returns the latest version
/// - Package name with version (e.g., "mypackage:1.0.0") - returns that specific version
///
/// # Errors
/// Returns an error if the HTTP request fails or the response cannot be parsed.
pub async fn get_packages_name_version() -> Result<&'static HashMap<String, RawPackage>> {
    static PACKAGES: OnceCell<HashMap<String, RawPackage>> = OnceCell::const_new();
    PACKAGES
        .get_or_try_init(|| async {
            let packages: Vec<RawPackage> = get_all_packages().await?;
            let mut hashmap: HashMap<String, RawPackage> =
                HashMap::with_capacity(packages.len() * 2);

            for pkg in packages {
                // Insert by name (latest version)
                let name = pkg.name.clone();
                let version_key = format!("{}:{}", pkg.name, pkg.version);

                hashmap.insert(name, pkg.clone());
                hashmap.insert(version_key, pkg);
            }

            Ok(hashmap)
        })
        .await
}
