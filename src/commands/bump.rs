use std::borrow::Cow;
use std::io::Write;
use std::fs::{File, read_to_string, write};
use std::str::FromStr;

use itertools::Itertools;
use toml_edit::{DocumentMut, value};
use tracing::instrument;
use typst_syntax::package::PackageVersion;

use crate::{
    utils::{self, dryrun::get_dry_run, state::Result},
    utpm_log,
};

use super::BumpArgs;

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
    let manifest_path = utils::try_find_path(&utils::paths::get_current_dir()?)?;
    let mut config = read_to_string(&manifest_path)?.parse::<DocumentMut>()?;

    let old_version = config["package"]["version"].as_str().unwrap();
    let new_version = cmd.new_version.as_str();
    PackageVersion::from_str(new_version).unwrap();

    let files = &cmd.include;

    for file in files {
        let mut string = read_to_string(file)?;
        utpm_log!(info, "Found {}", file);
        let old_version = tag_change(cmd.tag.as_deref(), old_version);
        let new_version = tag_change(cmd.tag.as_deref(), new_version);
        string = string.replace(old_version.as_ref(), &new_version);
        if !get_dry_run() {
            write(file, string)?;
        }
        utpm_log!(info, "Modified {}", file);
    }

    let files = ["typst.toml"].into_iter()
            .chain(files.iter().map(AsRef::<str>::as_ref))
            .join(", ");

    config["package"]["version"] = value(new_version);
    let mut file = File::create(manifest_path)?;
    write!(file, "{}", config)?;
    file.sync_all()?;

    utpm_log!(info, format!("New version: {new_version}"), "version" => new_version, "files" => files);
    Ok(true)
}
