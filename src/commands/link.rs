use std::fs;
use tracing::instrument;
use typst_project::manifest::Manifest;

use crate::{
    load_manifest, utils::{
        copy_dir_all,
        paths::{c_packages, check_path_dir, d_packages, get_current_dir},
        specs::Extra,
        state::{Result, UtpmError},
        symlink_all,
    }, utpm_bail, utpm_println
};

use super::LinkArgs;

#[instrument(skip(cmd))]
pub fn run(cmd: &LinkArgs, path: Option<String>, pt: bool) -> Result<bool> {
    let curr = path.unwrap_or(get_current_dir()?);

    let config = load_manifest!(&curr);
    let namespace = if let Some(value) = config.tool {
        value
            .get_section("utpm")?
            .unwrap_or(Extra::default())
            .namespace
            .unwrap_or("local".into())
    } else {
        "local".into()
    };

    let name = config.package.name;
    let version = config.package.version;
    let path = if namespace != "preview" {
        format!("{}/{}/{}/{}", d_packages()?, namespace, name, version)
    } else {
        format!("{}/{}/{}/{}", c_packages()?, namespace, name, version)
    };
    if check_path_dir(&path) && !cmd.force {
        utpm_bail!(AlreadyExist, name.to_string(), version, "Info:".to_string());
    }

    fs::create_dir_all(&path)?;

    if cmd.force {
        fs::remove_dir_all(&path)?
    }

    if cmd.no_copy {
        symlink_all(&curr, &path)?;
        if pt {
            utpm_println!(
                "Project linked to: {}\nTry importing with: \n#import \"@{}/{}:{}\": *",
                path, namespace, name, version
            );
        }
    } else {
        copy_dir_all(&curr, &path)?;
        if pt {
            utpm_println!(
                "Project linked to: {}\nTry importing with: \n#import \"@{}/{}:{}\": *",
                path, namespace, name, version
            );
        }
    }
    Ok(true)
}

