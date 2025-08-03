use std::fs::{read_to_string, write};

use semver::Version;
use tracing::instrument;

use crate::{utils::state::Result, utpm_log, write_manifest};

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
pub fn run<'a>(cmd: &'a BumpArgs) -> Result<bool> {
    let mut config = load_manifest!();
    let ver = Version::parse(&cmd.new_version)?;
 
    let strs = &cmd.new_version;
    let files = &cmd.include.clone();

    // Todo: dry run
    for file in files {
        let mut string = read_to_string(file)?;
        utpm_log!(info, "Found {}", file.clone());
        let ancien_version = &tag_change(cmd.tag.clone(), &config.package.version.to_string());
        let new_version = &tag_change(cmd.tag.clone(), strs);
        string = string.replace(ancien_version, new_version);
        write(file, string)?;
    }

    // Borrow is very anoying sometimes, this hack is necessary
    let mut real_files = vec!["typst.toml".to_string()];
    real_files.extend(files.iter().cloned());

    config.package.version = ver;
    write_manifest!(&config);
    utpm_log!(info, format!("New version: {strs}"), "version" => strs, "files" => real_files.join(", "));
    Ok(true)
}
