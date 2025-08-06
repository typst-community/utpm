use toml::map::Map;
use tracing::instrument;
use typst_syntax::package::PackageManifest;

use crate::{
    load_manifest,
    utils::{specs::Extra, state::Result},
    utpm_bail, utpm_log, write_manifest,
};

use super::{install, AddArgs, InstallArgs};

/// Adds dependencies to the `typst.toml` manifest and installs them.
#[instrument]
pub async fn run(cmd: &mut AddArgs) -> Result<bool> {
    utpm_log!(trace, "executing add command");
    // Load the manifest from the current directory.
    let mut config: PackageManifest = load_manifest!();
    if cmd.uri.len() == 0 {
        utpm_log!(debug, "0 URI found in cmd.uri");
        utpm_bail!(NoURIFound);
    }

    utpm_log!(
        debug,
        "{} URIs found: {}",
        cmd.uri.len(),
        cmd.uri.join(", ")
    );

    // Check if the manifest has a `[tool]` section.
    let mut tool = config.clone().tool;
    utpm_log!(trace, "- tool section found");
    // Check for the `[tool.utpm]` subsection.
    if let Some(ex) = tool.sections.get("utpm") {
        utpm_log!(trace, "- utpm section found in tool");
        // Deserialize the `[tool.utpm]` section into our `Extra` struct.
        let mut extra: Extra = toml::from_str(toml::to_string(ex)?.as_str())?; //Todo: change this hack
        utpm_log!(trace, "hacky conversion done");

        // Add the new URIs to the dependencies list.
        if let Some(mut dependencies) = extra.dependencies.clone() {
            utpm_log!(trace, "- dependencies found, adding uris");
            for e in &cmd.uri {
                // Avoid adding duplicate dependencies.
                match dependencies.iter().position(|x| x == e) {
                    Some(_) => {
                        utpm_log!(info, "{e} dependency already in the load_manifest skipping");
                    }
                    None => {
                        utpm_log!(trace, "{e} added");
                        dependencies.push(e.clone())
                    }
                };
            }
            extra.dependencies = Some(dependencies);
        } else {
            // If no dependencies existed, create a new list.
            extra.dependencies = Some(cmd.uri.clone());
        }
        // Update the `[tool.utpm]` section in the manifest.
        tool.sections.insert("utpm".into(), Map::try_from(extra)?);
    } else {
        // If `[tool.utpm]` doesn't exist, create it.
        tool.sections.insert(
            "utpm".into(),
            toml::from_str(
                toml::to_string(&Extra::new(None, Some(cmd.uri.clone()), None))?.as_str(),
            )?,
        );
    }
    config.tool = tool;

    // Write the updated manifest back to the file.
    write_manifest!(&config);

    // Run the `install` command to download and link the new dependencies.
    install::run(&InstallArgs {
        force: false,
        url: None,
    })
    .await?;

    Ok(true)
}
