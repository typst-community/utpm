use toml::map::Map;
use tracing::instrument;

use crate::{
    load_manifest,
    utils::{
        specs::Extra,
        state::Result,
    },
    utpm_log, write_manifest,
};

use super::DeleteArgs;

/// Removes dependencies from the `typst.toml` manifest.
#[instrument(skip(cmd))]
pub async fn run(cmd: &mut DeleteArgs) -> Result<bool> {
    utpm_log!(trace, "executing delete command");
    let mut config = load_manifest!();

    // Check for the `[tool]` section in the manifest.
    if let Some(mut tool_section) = config.tool.take() {
        // Check for the `[tool.utpm]` subsection.
        if let Some(entry) = tool_section.keys.get_mut("utpm") {
            let mut extra: Extra = toml::from_str(&toml::to_string(entry)?)?;
            // Check for the `dependencies` key.
            if let Some(mut dependencies) = extra.dependencies.take() {
                // Iterate over the URIs to remove.
                for uri in &cmd.uri {
                    // Find and remove the dependency.
                    match dependencies.iter().position(|x| x == uri) {
                        Some(idx) => {
                            dependencies.remove(idx);
                            utpm_log!(info, "Removed {}", uri);
                        }
                        None => utpm_log!(warn, "Can't remove {} (not found)", uri),
                    }
                }
                extra.dependencies = Some(dependencies);
                // Update the `[tool.utpm]` section with the modified dependencies.
                tool_section
                    .keys
                    .insert("utpm".to_string(), Map::try_from(extra)?);
            } else {
                utpm_log!(
                    "Nothing has changed! There isn't a key for 'dependencies' in the typst.toml."
                )
            }
        } else {
            utpm_log!("Nothing has changed! There isn't a tool section dedicated to utpm in the typst.toml.")
        }
        config.tool = Some(tool_section);
    } else {
        utpm_log!("Nothing has changed! There isn't a tool section in the typst.toml.")
    }

    // Write the modified manifest back to the file.
    write_manifest!(&config);
    Ok(true)
}
