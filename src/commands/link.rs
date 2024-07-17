use owo_colors::OwoColorize;
use std::fs;
use typst_project::manifest::Manifest;

use crate::utils::{
    copy_dir_all,
    paths::{check_path_dir, d_packages, get_current_dir},
    specs::Extra,
    state::{Error, ErrorKind, Result},
    symlink_all,
};

use super::LinkArgs;

pub fn run(cmd: &LinkArgs, path: Option<String>) -> Result<bool> {
    let curr = path.unwrap_or(get_current_dir()?);

    let config = Manifest::try_find(&(curr.clone() + "/typst.toml"))?.unwrap();
    let namespace = if let Some(value) = config.tool {
        value
            .get_section("utpm")
            .unwrap()
            .unwrap_or(Extra::default())
            .namespace
            .unwrap_or("local".into())
    } else {
        "local".into()
    };

    let name = config.package.name;
    let version = config.package.version;
    let path = format!("{}/{}/{}/{}", d_packages(), namespace, name, version);
    let binding = "Info:".yellow();
    let info = binding.bold();
    if check_path_dir(&path) && !cmd.force {
        return Err(Error::empty(ErrorKind::AlreadyExist(
            name.into(),
            version,
            format!("{}", info),
        )));
    }

    fs::create_dir_all(&path)?;

    if cmd.force {
        fs::remove_dir_all(&path)?
    }

    if cmd.no_copy {
        symlink_all(&curr, &path)?;
        println!(
            "Project linked to: {} \nTry importing with:\n #import \"@{}/{}:{}\": *",
            path, namespace, name, version
        );
    } else {
        copy_dir_all(&curr, &path)?;
        println!(
            "Project copied to: {} \nTry importing with:\n #import \"@{}/{}:{}\": *",
            path, namespace, name, version
        );
    }
    Ok(true)
}
