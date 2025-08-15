use std::{fs::{read_to_string, write}, str::FromStr};

use tracing::instrument;
use typst_syntax::package::PackageVersion;

use crate::{
    utils::{dryrun::get_dry_run, state::Result},
    utpm_log, write_manifest,
};

use super::BumpArgs;

use crate::load_manifest;

/// Check if there is a tag.
/// If there is, it creates a new string like an html element
/// If not, return value
fn tag_change(tag: Option<String>, value: &String) -> String {
    if let Some(tag) = tag {
        format!("<{tag}>{value}</{tag}>")
    } else {
        value.clone()
    }
}

#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a BumpArgs) -> Result<bool> {
    utpm_log!(trace, "executing bump command");
    let mut config = load_manifest!();

    let ver = PackageVersion::from_str(&cmd.new_version).unwrap();

    let strs = &cmd.new_version;
    let files = &cmd.include.clone();

    for file in files {
        let mut string = read_to_string(file)?;
        utpm_log!(info, "Found {}", file.clone());
        let ancien_version = &tag_change(cmd.tag.clone(), &config.package.version.to_string());
        let new_version = &tag_change(cmd.tag.clone(), strs);
        string = string.replace(ancien_version, new_version);
        if !get_dry_run() {
            write(file, string)?;
        }
        utpm_log!(info, "Modified {}", file.clone());
    }

    // Borrow is very annoying sometimes, this hack is necessary
    let mut real_files = vec!["typst.toml".to_string()];
    real_files.extend(files.iter().cloned());

    config.package.version = ver;
    write_manifest!(&config);
    utpm_log!(info, format!("New version: {strs}"), "version" => strs, "files" => real_files.join(", "));
    Ok(true)
}
