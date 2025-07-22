use semver::Version;
use tracing::instrument;

use crate::{utils::state::Result, utpm_log, write_manifest};

use super::BumpArgs;

use crate::load_manifest;

#[instrument(skip(cmd))]
pub fn run<'a>(cmd: &BumpArgs) -> Result<bool> {
    let mut config = load_manifest!();
    let ver = Version::parse(&cmd.new_version)?;
    config.package.version = ver;
    // Todo: dry run
    write_manifest!(&config);
    let strs = &cmd.new_version;
    let files: Vec<&'a str> = vec!["typst.toml"];
    utpm_log!(info, format!("New version: {strs}"), "version" => strs, "files" => files.join(", "));
    Ok(true)
}
