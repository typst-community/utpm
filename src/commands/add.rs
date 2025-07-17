use std::{collections::BTreeMap, fs};

use toml::map::Map;
use tracing::instrument;
use typst_project::manifest::{tool::Tool, Manifest};

use crate::{
    load_manifest,
    utils::{
        paths::get_current_dir,
        specs::Extra,
        state::{Result, UtpmError},
    },
    utpm_bail, utpm_log, write_manifest,
};

use super::{install, AddArgs, InstallArgs};

#[instrument]
pub fn run(cmd: &mut AddArgs) -> Result<bool> {
    let mut config: Manifest = load_manifest!();
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
    if let Some(mut tool) = config.clone().tool {
        utpm_log!(trace, "- tool section found");
        if let Some(ex) = tool.keys.get("utpm") {
            utpm_log!(trace, "- utpm section found in tool");
            let mut extra: Extra = toml::from_str(toml::to_string(ex)?.as_str())?; //Todo: change this hack
            utpm_log!(trace, "hacky conversion done");
            if let Some(mut dependencies) = extra.dependencies.clone() {
                utpm_log!(trace, "- dependencies found, adding uris");
                for e in &cmd.uri {
                    match dependencies.iter().position(|x| x == e) {
                        Some(_) => {
                            utpm_log!(
                                trace,
                                "{e} dependency already in the load_manifest skipping"
                            );
                        }
                        None => {
                            utpm_log!(trace, "{e} added");
                            dependencies.push(e.clone())
                        }
                    };
                }
                extra.dependencies = Some(dependencies);
            } else {
                extra.dependencies = Some(cmd.uri.clone());
            }
            tool.keys.insert("utpm".to_string(), Map::try_from(extra)?);
        } else {
            tool.keys.insert(
                "utpm".to_string(),
                toml::from_str(
                    toml::to_string(&Extra::new(None, Some(cmd.uri.clone()), None))?.as_str(),
                )?,
            );
        }
        config.tool = Some(tool);
    } else {
        let mut keys = BTreeMap::new();
        keys.insert(
            "utpm".to_string(),
            toml::from_str(
                toml::to_string(&Extra::new(None, Some(cmd.uri.clone()), None))?.as_str(),
            )?,
        );
        config.tool = Some(Tool { keys });
    }

    write_manifest!(&config);

    install::run(&InstallArgs {
        force: false,
        url: None,
    })?;

    Ok(true)
}
