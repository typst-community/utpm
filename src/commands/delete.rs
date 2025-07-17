use std::fs;

use toml::map::Map;
use tracing::instrument;
use typst_project::manifest::Manifest;

use crate::{
    load_manifest,
    utils::{
        paths::get_current_dir,
        specs::Extra,
        state::{Result, UtpmError},
    },
    utpm_log, write_manifest,
};

use super::DeleteArgs;

/// Removes dependencies from the `typst.toml` manifest.
#[instrument(skip(cmd))]
pub fn run(cmd: &mut DeleteArgs) -> Result<bool> {
    let mut config = load_manifest!();

    if let Some(mut tool_section) = config.tool.take() {
        if let Some(entry) = tool_section.keys.get_mut("utpm") {
            let mut extra: Extra = toml::from_str(&toml::to_string(entry)?)?;
            if let Some(mut dependencies) = extra.dependencies.take() {
                for uri in &cmd.uri {
                    match dependencies.iter().position(|x| x == uri) {
                        Some(idx) => {
                            dependencies.remove(idx);
                            utpm_log!("Removed {}", uri);
                        }
                        None => utpm_log!("Can't remove {} (not found)", uri),
                    }
                }
                extra.dependencies = Some(dependencies);
                // Mettre à jour l'entrée utpm dans tool_section avec la nouvelle structure modifiée
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

    // Sauvegarder la configuration modifiée
    write_manifest!(&config);
    Ok(true)
}