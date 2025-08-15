use std::borrow::Cow;
use std::fs::{read_to_string, write};
use std::str::FromStr;

use itertools::Itertools;
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
fn tag_change<'a>(tag: Option<&str>, value: &'a str) -> Cow<'a, str> {
    if let Some(tag) = tag {
        Cow::Owned(format!("<{tag}>{value}</{tag}>"))
    } else {
        Cow::Borrowed(value)
    }
}

#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a BumpArgs) -> Result<bool> {
    utpm_log!(trace, "executing bump command");
    let mut config = load_manifest!();

    let old_version = config.package.version.to_string();
    let new_version = cmd.new_version.as_str();
    let ver = PackageVersion::from_str(new_version).unwrap();

    let files = &cmd.include;

    for file in files {
        let mut string = read_to_string(file)?;
        utpm_log!(info, "Found {}", file);
        let old_version = tag_change(cmd.tag.as_deref(), &old_version);
        let new_version = tag_change(cmd.tag.as_deref(), new_version);
        string = string.replace(old_version.as_ref(), &new_version);
        if !get_dry_run() {
            write(file, string)?;
        }
        utpm_log!(info, "Modified {}", file);
    }

    // Borrow is very anoying sometimes, this hack is necessary
    let files = ["typst.toml"].into_iter()
            .chain(files.iter().map(AsRef::<str>::as_ref))
            .join(", ");

    config.package.version = ver;
    write_manifest!(&config);
    utpm_log!(info, format!("New version: {new_version}"), "version" => new_version, "files" => files);
    Ok(true)
}
