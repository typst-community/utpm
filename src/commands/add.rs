use std::{collections::BTreeMap, fs};

use toml::map::Map;
use typst_project::manifest::{tool::Tool, Manifest};

use crate::{manifest, utils::{
    paths::get_current_dir,
    specs::Extra,
    state::{Error, ErrorKind, Result},
}};

use super::{install, AddArgs, InstallArgs};



pub fn run(cmd: &AddArgs) -> Result<bool> {
    let mut config = manifest!();

    if let Some(mut tool) = config.clone().tool {
        if let Some(ex) = tool.keys.get("utpm") {
            let mut extra: Extra = toml::from_str(toml::to_string(ex)?.as_str())?; //Todo: change this hack
            if let Some(mut dependencies) = extra.dependencies.clone() {
                dependencies.push(cmd.uri.clone());
            } else {
                extra.dependencies = Some(vec![cmd.uri.clone()]);
            }
            tool.keys.insert(
                "utpm".to_string(),
                Map::try_from(extra)?
            );
        } else {
            tool.keys.insert(
                "utpm".to_string(),
                toml::from_str(
                    toml::to_string(&Extra::new(None, Some(vec![cmd.uri.clone()])))?.as_str(),
                )?,
            );
        }
    } else {
        let mut keys = BTreeMap::new();
        keys.insert(
            "utpm".to_string(),
            toml::from_str(
                toml::to_string(&Extra::new(None, Some(vec![cmd.uri.clone()])))?.as_str(),
            )?,
        );
        config.tool = Some(Tool { keys });
    }

    let tomlfy: String = toml::to_string_pretty(&config)?;
    fs::write("./typst.toml", tomlfy)?;
    
    install::run(&InstallArgs { force: false, url: Some(cmd.uri.clone())})?;

    Ok(true)
}
