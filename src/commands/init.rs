use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    str::FromStr,
};

use inquire::{Select, Text, required, validator::Validation};
use semver::Version;
use toml::Table;
use tracing::instrument;
use typst_syntax::package::{PackageInfo, PackageManifest, PackageVersion, ToolInfo, VersionBound};

use crate::{
    utils::{
        dryrun::get_dry_run,
        paths::{check_path_file, get_current_dir},
        specs::Extra,
        state::Result,
    },
    utpm_log, write_manifest,
};

use super::InitArgs;

/// Initializes a new typst project by creating a `typst.toml` manifest.
///
/// This function can run in interactive mode, prompting the user for configuration details,
/// or in non-interactive mode using command-line arguments.
#[instrument(skip(cmd))]
pub async fn run(cmd: &mut InitArgs) -> Result<bool> {
    utpm_log!(trace, "executing init command");
    let curr = get_current_dir()?;
    utpm_log!(info, "Current dir: {}", curr);
    let typ = curr.clone() + "/typst.toml";
    utpm_log!(info, "Current typst file: {}", typ);

    // Build the package metadata from command-line arguments.
    let mut pkg = PackageInfo {
        name: cmd.name.clone().unwrap_or("".into()).into(),
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
        license: if let Some(yes) = &cmd.license {
            Some(yes.into())
        } else {
            None
        },
        description: if let Some(yes) = &cmd.description {
            Some(yes.into())
        } else {
            None
        },
        repository: if let Some(yes) = &cmd.repository {
            Some(yes.into())
        } else {
            None
        },
        homepage: if let Some(yes) = &cmd.homepage {
            Some(yes.into())
        } else {
            None
        },
        keywords: if let Some(yes) = &cmd.keywords {
            yes.iter().map(|f| f.into()).collect::<Vec<_>>()
        } else {
            vec![]
        },
        compiler: if let Some(yes) = &cmd.compiler {
            Some(VersionBound {
                major: yes.major as u32,
                minor: Some(yes.minor as u32),
                patch: Some(yes.patch as u32),
            })
        } else {
            None
        },
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
    };

    // TODO: Implement template handling.
    //let mut tmpl: Template = Template::new(cmd.template, entrypoint, thumbnail)

    // Check if manifest already exists.
    if check_path_file(&typ) && !cmd.force {
        return Ok(false);
    }

    if cmd.force {
        utpm_log!(warn, "--force is a dangerous flag, use it cautiously");
    }

    // Interactive mode for gathering project metadata.
    if !cmd.cli {
        let choice = vec!["yes", "no"];
        let public = Select::new("Do you want to make your package public? Questions are on authors, license, description", choice.clone()).prompt()?;
        let more = Select::new("Do you want more questions to customise your package? Questions are on repository url, homepage url, keywords, compiler version, excluded files, categories and disciplines", choice.clone()).prompt()?;
        let template = Select::new("Do you want to create a template?", choice.clone()).prompt()?;
        let popu = Select::new(
            "Do you want to populate your package? Files like index.typ will be created",
            choice,
        )
        .prompt()?;

        if popu == "yes" {
            cmd.populate = true;
        }

        pkg.name = Text::new("Name: ")
            .with_validator(required!("This field is required"))
            .with_help_message("e.g. my_example")
            .prompt()?
            .as_str()
            .into();

        pkg.version = PackageVersion::from_str(
            &Text::new("Version: ")
                .with_validator(required!("This field is required"))
                .with_validator(|obj: &str| match Version::parse(obj) {
                    Ok(_) => Ok(Validation::Valid),
                    Err(_) => Ok(Validation::Invalid(
                        "A correct version must be types (check semVer)".into(),
                    )),
                })
                .with_help_message("e.g. 1.0.0 (SemVer)")
                .with_default("1.0.0")
                .prompt()?,
        )
        .unwrap();

        pkg.entrypoint = Text::new("Entrypoint: ")
            .with_validator(required!("This field is required"))
            .with_help_message("e.g. main.typ")
            .with_default("main.typ")
            .prompt()?
            .into();

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
                    .with_validator(&|obj: &str| match spdx::Expression::parse(obj) {
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
                        }
                        Err(_) => Ok(Validation::Invalid("Can't parse your expression".into())),
                    })
                    .prompt()?
                    .as_str()
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
                    .with_help_message("e.g. https://github.com/Thumuss/utpm")
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
                .with_help_message("e.g. Typst,keyword")
                .prompt()?
                .split(",")
                .map(|f| f.into())
                .collect::<Vec<_>>();

            pkg.compiler = Some(
                VersionBound::from_str(
                    &Text::new("Version: ")
                        .with_validator(required!("This field is required"))
                        .with_validator(|obj: &str| match Version::parse(obj) {
                            Ok(_) => Ok(Validation::Valid),
                            Err(_) => Ok(Validation::Invalid(
                                "A correct version must be types (check semVer)".into(),
                            )),
                        })
                        .with_help_message("e.g. 1.0.0 (SemVer)")
                        .with_default("1.0.0")
                        .prompt()?,
                )
                .unwrap(),
            );

            pkg.exclude = Text::new("Exclude: ")
                .with_help_message("e.g. backup/mypassword.txt,.env")
                .prompt()?
                .split(",")
                .filter(|f| f.len() > 0)
                .map(|f| f.into())
                .collect::<Vec<_>>();
        }

        if template == "yes" {
            //TODO: Implement template creation.
        }
    }

    // Populate the project with default files if requested.
    if cmd.populate && !get_dry_run() {
        let mut file = File::create(curr.clone() + "/README.md")?; // README.md
        file.write_all(("# ".to_string() + &pkg.name.clone()).as_bytes())?;
        if let Some(exp) = spdx::license_id(pkg.clone().license.unwrap().to_string().as_str()) {
            file = File::create(curr.clone() + "/LICENSE")?; // LICENSE
            file.write_all(exp.text().as_bytes())?;
        }

        create_dir_all(curr.clone() + "/examples")?; // examples
        let examples = curr.clone() + "/examples";
        file = File::create(examples + "/tests.typ")?; // examples/tests.typ
        let fm = format!(
            "#import \"@local/{}:{}\": *\nDo...",
            pkg.name.clone(),
            pkg.version.clone().to_string()
        );
        file.write_all(fm.as_bytes())?;
        file = File::create(Path::new(&pkg.entrypoint.to_string()))?; // main.typ
        file.write_all(
            b"// This file is generated by UTPM (https://github.com/typst-community/utpm)",
        )?;
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
    write_manifest!(&manif);

    utpm_log!(info, "File created to {typ}");
    Ok(true)
}
