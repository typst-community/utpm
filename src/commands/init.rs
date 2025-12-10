use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    str::FromStr,
};

use inquire::{Select, Text, required, validator::Validation};
use toml::Table;
use tracing::instrument;
use typst_syntax::package::{PackageInfo, PackageManifest, PackageVersion, ToolInfo, VersionBound};

use crate::{
    utils::{
        dryrun::get_dry_run,
        paths::{check_path_file, get_current_dir},
        specs::Extra,
        state::Result,
        write_manifest,
    },
    utpm_bail, utpm_log,
};

use super::InitArgs;

/// Build the package metadata through an interactive prompt
fn interactive_pkg_info(cmd: &mut InitArgs) -> Result<PackageInfo> {
    let choice = vec!["yes", "no"];
    let public = Select::new(
        "Do you want to make your package public? Questions are on authors, license, description",
        choice.clone(),
    )
    .prompt()?;
    let more = Select::new("Do you want more questions to customise your package? Questions are on repository url, homepage url, keywords, compiler version, excluded files, categories and disciplines", choice.clone()).prompt()?;
    let template = Select::new("Do you want to create a template?", choice.clone()).prompt()?;

    if template == "yes" {
        //TODO: Implement template creation.
        utpm_bail!(Unknown, "Template creation is not implemented yet.".into());
    }

    let popu = Select::new(
        "Do you want to populate your package? Files like index.typ will be created",
        choice,
    )
    .prompt()?;

    if popu == "yes" {
        cmd.populate = true;
    }

    let name = Text::new("Name: ")
        .with_validator(required!("This field is required"))
        .with_help_message("e.g. my_example")
        .prompt()?
        .into();

    let version = PackageVersion::from_str(
        &Text::new("Version: ")
            .with_validator(required!("This field is required"))
            .with_validator(|obj: &str| match PackageVersion::from_str(obj) {
                Ok(_) => Ok(Validation::Valid),
                Err(_) => Ok(Validation::Invalid(
                    "A correct version must be typed (check semVer)".into(),
                )),
            })
            .with_help_message("e.g. 1.0.0 (SemVer)")
            .with_default("1.0.0")
            .prompt()?,
    )
    .unwrap();

    let entrypoint = Text::new("Entrypoint: ")
        .with_validator(required!("This field is required"))
        .with_help_message("e.g. main.typ")
        .with_default("main.typ")
        .prompt()?
        .into();

    let mut pkg = PackageInfo {
        name,
        version,
        entrypoint,
        authors: vec![],
        license: None,
        description: None,
        repository: None,
        homepage: None,
        keywords: vec![],
        compiler: None,
        exclude: vec![],
        categories: vec![],
        disciplines: vec![],
        unknown_fields: BTreeMap::new(),
    };

    if public == "yes" {
        pkg.authors = Text::new("Authors: ")
            .with_help_message("e.g. Thumus,Somebody,Somebody Else")
            .prompt()?
            .split(",")
            .map(|f| f.into())
            .collect::<Vec<_>>();

        pkg.license = Some(
            Text::new("License: ")
                .with_help_message("e.g. MIT")
                .with_default("Unlicense")
                .with_validator(|obj: &str| match spdx::Expression::parse(obj) {
                    Ok(val) => {
                        for x in val.requirements() {
                            let id = x.req.license.id().unwrap();
                            if !id.is_osi_approved() {
                                return Ok(Validation::Invalid(
                                    "It must be an OSI approved!".into(),
                                ));
                            }
                        }
                        Ok(Validation::Valid)
                    },
                    Err(_) => Ok(Validation::Invalid("Can't parse your expression".into())),
                })
                .prompt()?
                .into(),
        );

        pkg.description = Some(
            Text::new("description: ")
                .with_help_message("e.g. A package")
                .prompt()?
                .into(),
        )
    }
    if more == "yes" {
        pkg.repository = Some(
            Text::new("URL of the repository: ")
                .with_help_message("e.g. https://github.com/typst-community/utpm")
                .prompt()?
                .into(),
        );
        pkg.homepage = Some(
            Text::new("Homepage: ")
                .with_help_message("e.g. anything")
                .prompt()?
                .into(),
        );
        pkg.keywords = Text::new("Keywords: ")
            .with_help_message("e.g. touying, slide, theme, ...")
            .prompt()?
            .split(",")
            .filter(|f| !f.trim().is_empty())
            .map(|f| f.trim().into())
            .collect::<Vec<_>>();

        let compiler_version = Text::new("Min Compiler Version (leave empty to skip): ")
            .with_validator(|obj: &str| {
                if obj.is_empty() {
                    Ok(Validation::Valid)
                } else {
                    match VersionBound::from_str(obj) {
                        Ok(_) => Ok(Validation::Valid),
                        Err(_) => Ok(Validation::Invalid(
                            "A correct version bound must be typed (check semVer)".into(),
                        )),
                    }
                }
            })
            .with_help_message("e.g. 1.0.0 (SemVer)")
            .prompt()?;
        if !compiler_version.is_empty() {
            pkg.compiler = Some(VersionBound::from_str(&compiler_version).unwrap());
        }

        pkg.exclude = Text::new("Exclude: ")
            .with_help_message("e.g. backup/mypassword.txt, .env")
            .prompt()?
            .split(",")
            .filter(|f| !f.trim().is_empty())
            .map(|f| f.trim().into())
            .collect::<Vec<_>>();
    }

    Ok(pkg)
}

/// Build the package metadata from command-line arguments.
fn cmd_pkg_info(cmd: &InitArgs) -> Result<PackageInfo> {
    Ok(PackageInfo {
        name: <std::option::Option<std::string::String> as Clone>::clone(&cmd.name)
            .unwrap()
            .into(),
        version: PackageVersion {
            major: cmd.version.major as u32,
            minor: cmd.version.minor as u32,
            patch: cmd.version.patch as u32,
        },
        entrypoint: cmd.entrypoint.to_owned().into(),
        authors: if let Some(yes) = &cmd.authors {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        license: cmd.license.as_ref().map(|yes| yes.into()),
        description: cmd.description.as_ref().map(|yes| yes.into()),
        repository: cmd.repository.as_ref().map(|yes| yes.into()),
        homepage: cmd.homepage.as_ref().map(|yes| yes.into()),
        keywords: if let Some(yes) = &cmd.keywords {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        compiler: cmd.compiler.as_ref().map(|yes| VersionBound {
            major: yes.major as u32,
            minor: Some(yes.minor as u32),
            patch: Some(yes.patch as u32),
        }),
        exclude: if let Some(yes) = &cmd.exclude {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        categories: if let Some(yes) = &cmd.categories {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        disciplines: if let Some(yes) = &cmd.disciplines {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        unknown_fields: BTreeMap::new(),
    })
}

fn create_pkg_info(cmd: &mut InitArgs) -> Result<PackageInfo> {
    if !cmd.cli {
        interactive_pkg_info(cmd)
    } else {
        cmd_pkg_info(cmd)
    }
}

fn populate_project_files(project_dir: &Path, pkg: &PackageInfo) -> Result<()> {
    let mut file = File::create(project_dir.join("README.md"))?; // README.md
    file.write_all(format!("# {}", pkg.name).as_bytes())?;
    if let Some(ref license) = pkg.license
        && let Some(exp) = spdx::license_id(license.as_ref())
    {
        file = File::create(project_dir.join("LICENSE"))?; // LICENSE
        file.write_all(exp.text().as_bytes())?;
    }

    let examples = project_dir.join("examples");
    create_dir_all(&examples)?; // examples
    file = File::create(examples.join("tests.typ"))?; // examples/tests.typ
    write!(
        file,
        "#import \"@local/{}:{}\":\n Do...",
        pkg.name, pkg.version
    )?;
    file = File::create(project_dir.join(pkg.entrypoint.to_string()))?; // main.typ
    write!(
        file,
        "// This file is generated by UTPM (https://github.com/typst-community/utpm)"
    )?;
    Ok(())
}

/// Initializes a new typst project by creating a `typst.toml` manifest.
///
/// This function can run in interactive mode, prompting the user for configuration details,
/// or in non-interactive mode using command-line arguments.
#[instrument(skip(cmd))]
pub async fn run(cmd: &mut InitArgs) -> Result<bool> {
    utpm_log!(trace, "executing init command");
    let project_dir = get_current_dir()?;
    utpm_log!(info, "Current dir: {}", project_dir.display());
    let typ = project_dir.join(crate::utils::paths::MANIFEST_FILE);
    utpm_log!(info, "Current typst file: {}", typ.display());

    // TODO: Implement template handling.
    //let mut tmpl: Template = Template::new(cmd.template, entrypoint, thumbnail)

    // Check if manifest already exists.
    if check_path_file(&typ) && !cmd.force {
        utpm_log!(
            error,
            "typst.toml already exists. Use --force to overwrite it."
        );
        return Ok(false);
    }

    // Build the package metadata from command-line arguments.
    let pkg = create_pkg_info(cmd)?;

    // Populate the project with default files if requested.
    if cmd.populate && !get_dry_run() {
        populate_project_files(&project_dir, &pkg)?;
    }

    // Create the `[tool.utpm]` table.
    let mut keys: BTreeMap<_, Table> = BTreeMap::new();
    keys.insert("utpm".into(), Table::try_from(Extra::default())?);

    // Construct the final manifest.
    let manif = PackageManifest {
        package: pkg,
        tool: ToolInfo { sections: keys },
        template: None,
        unknown_fields: BTreeMap::new(),
    };

    // Write the manifest to `typst.toml`.
    write_manifest(&manif)?;

    utpm_log!(info, "File created to {typ}");
    Ok(true)
}
