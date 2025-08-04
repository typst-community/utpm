use std::{fs::write, path::Path};

use ignore::WalkBuilder;
use tracing::instrument;

use std::result::Result as R;

use crate::{
    commands::get::get_packages_name_version, utils::{
        paths::{d_packages, get_current_dir},
        regex_import,
        state::Result,
    }, utpm_bail, utpm_log
};

use super::SyncArgs;

#[instrument(skip(cmd))]
pub async fn run<'a>(cmd: &'a SyncArgs) -> Result<bool> {
    if cmd.files.len() == 0 {
        default_run().await?;
        Ok(true)
    } else {
        files_run(&cmd.files).await?;
        Ok(true)
    }
}


async fn default_run() -> Result<bool> {
    let wb: WalkBuilder = WalkBuilder::new(&Path::new(&get_current_dir()?));
    for result in wb.build().collect::<R<Vec<_>, _>>()? {
        if let Some(file_type) = result.file_type() {
            if !file_type.is_dir() {
                utpm_log!(info, "Syncing {}...", result.file_name().to_str().unwrap());
                file_run(result.path()).await?;
            }
        }
    }
    Ok(true)
}

// TODO: Comments using utpm_log
async fn file_run(path: &Path) -> Result<bool> {
    let re = regex_import();
    let content_bytes = match std::fs::read(path) {
        Ok(bytes) => Ok(bytes),
        Err(e) => {
            utpm_log!(warn, "Skipping file {:?}, could not read: {}", path, e);
            Err(e)
        }
    }?;

    let mut string = match String::from_utf8(content_bytes) {
        Ok(s) => Ok(s),
        Err(e) => {
            utpm_log!(warn, "Skipping non-UTF-8 file: {:?}", path);
            Err(e)
        }
    }?;

    // Creating offset if there is multiple import and they don't have the same length
    let mut offset: isize = 0;
    let mut modified = false;

    // Matching regex
    let nws = string.clone();
    let res = re.find_iter(nws.as_str());
    let res2 = re.captures_iter(nws.as_str());
    utpm_log!(info, "Found imports");
    for e in res.zip(res2) {
        modified = true;
        // Set positions to rewrite the version
        let start = (e.0.start() as isize + offset) as usize;
        let end = (e.0.end() as isize + offset) as usize;
        let range = start..end;

        // Extract infos from the import
        let (_, [namespace, package, major, minor, patch]) = e.1.extract();

        let version = if namespace == "preview" {
            let pkgs = get_packages_name_version().await?;
            // TODO: check if the package exist
            Some(pkgs.get(package).unwrap().version.clone())
        } else {
            // TODO: Check here too
            let r = std::fs::read_dir(d_packages()? + format!("/{namespace}/{package}").as_str())?;
            let mut list_dir = r
                .into_iter()
                .map(|a| a.unwrap().file_name().to_str().unwrap().to_string())
                .collect::<Vec<_>>();
            list_dir.sort();
            Some(list_dir.last().unwrap().clone())
        };

        if version.is_none() {
            utpm_log!(error, "Can't find the package");
            utpm_bail!(PackageNotExist);
        }

        // Replace the import by the new
        let new_import: String = format!(
            "#import \"@{namespace}/{package}:{}\" /* From {major}.{minor}.{patch} */",
            version.unwrap()
        );
        utpm_log!(info, new_import);
        let old_len = string.len();
        string.replace_range(range, &new_import);

        // Set the offet and the new string.
        offset += (string.len() as isize) - (old_len as isize);
    }
    
    if modified {
        write(path, &string)?;
    }
    Ok(true)
}

async fn files_run(files: &Vec<String>) -> Result<bool> {
    for file in files {
        let path = Path::new(file.as_str());
        file_run(path).await?;
    }
    Ok(true)
}
