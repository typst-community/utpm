use std::collections::HashMap;

use fmt_derive::Display;
use serde::{Deserialize, Serialize};
use toml::to_string_pretty;
use tracing::instrument;

use crate::{utils::state::Result, utpm_log};

use super::GetArgs;

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

pub async fn get_all_packages() -> Result<Vec<RawPackage>> {
    let packages: Vec<RawPackage> = reqwest::get("https://packages.typst.org/preview/index.json")
        .await?
        .json::<Vec<RawPackage>>()
        .await?;
    Ok(packages)
}

pub async fn get_packages_name_version() -> Result<HashMap<String, RawPackage>> {
    let packages: Vec<RawPackage> = get_all_packages().await?;
    let packages_version: Vec<RawPackage> = packages.clone();
    let mut hashmap: HashMap<String, RawPackage> =
        packages.into_iter().map(|e| (e.name.clone(), e)).collect();
    let hashmap_version: HashMap<String, RawPackage> = packages_version
        .into_iter()
        .map(|e| (format!("{}:{}", e.name.clone(), e.version.clone()), e))
        .collect();
    hashmap.extend(hashmap_version);
    Ok(hashmap)
}

#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a GetArgs) -> Result<bool> {
    utpm_log!(trace, "executing get command");
    if !cmd.packages.is_empty() {
        let packages: HashMap<String, RawPackage> = get_packages_name_version().await?;
        for e in &cmd.packages {
            if !packages.contains_key(e) {
                utpm_log!(warn, "Package not found", "input" => e);
                continue;
            }
            let package = packages.get(e).unwrap();
            utpm_log!(info, package);
        }
    } else {
        let packages: Vec<_> = get_all_packages().await?;
        for package in packages {
            utpm_log!(info, package);
        }
    }

    Ok(true)
}
