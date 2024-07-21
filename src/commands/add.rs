use std::{collections::BTreeMap, fs};

use toml::map::Map;
use typst_project::manifest::{tool::Tool, Manifest};

use crate::{
    manifest,
    utils::{
        paths::get_current_dir,
        specs::Extra,
        state::{Error, ErrorKind, Result},
    },
    write_manifest,
};

use super::{install, AddArgs, InstallArgs};

pub fn run(cmd: &mut AddArgs) -> Result<bool> {
    let mut config: Manifest = manifest!();
    if cmd.uri.len() == 0 {
        return Err(Error::new(
            ErrorKind::NotEnoughArgs,
            "uri needs more than 0 arguments.".into(),
        ));
    }
    if let Some(mut tool) = config.clone().tool {
        if let Some(ex) = tool.keys.get("utpm") {
            let mut extra: Extra = toml::from_str(toml::to_string(ex)?.as_str())?; //Todo: change this hack
            if let Some(mut dependencies) = extra.dependencies.clone() {
                for e in &cmd.uri {
                    dependencies.push(e.clone());
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
                    toml::to_string(&Extra::new(None, Some(cmd.uri.clone())))?.as_str(),
                )?,
            );

        }
        config.tool = Some(tool);
    } else {
        let mut keys = BTreeMap::new();
        keys.insert(
            "utpm".to_string(),
            toml::from_str(toml::to_string(&Extra::new(None, Some(cmd.uri.clone())))?.as_str())?,
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
