use std::{
    collections::BTreeMap,
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    str::FromStr,
};

use inquire::{Select, Text, required, validator::Validation};
use tracing::instrument;
use typst_syntax::package::{
    PackageInfo, PackageManifest, PackageVersion, TemplateInfo, ToolInfo, VersionBound,
};

use crate::{
    utils::{
        dryrun::get_dry_run,
        paths::{check_path_file, get_current_dir},
        state::Result,
        write_manifest,
    },
    utpm_bail, utpm_log,
};

use super::InitArgs;

/// Build the package metadata through an interactive prompt
fn interactive_pkg_info(cmd: &mut InitArgs) -> Result<(PackageInfo, Option<TemplateInfo>)> {
    fn select_yes_no(message: &str) -> Result<bool> {
        let choice = Select::new(message, vec!["yes", "no"]).prompt()?;
        Ok(choice == "yes")
    }

    fn split_csv(input: &str) -> Vec<ecow::EcoString> {
        input
            .split(",")
            .filter(|f| !f.trim().is_empty())
            .map(|f| f.trim().into())
            .collect()
    }

    let public = select_yes_no(
        "Do you want to make your package public? Questions are on authors, license, description",
    )?;
    let more = select_yes_no(
        "Do you want more questions to customise your package? Questions are on repository url, homepage url, keywords, compiler version, excluded files, categories and disciplines",
    )?;
    let template = select_yes_no("Do you want to create a template?")?;

    let populate = select_yes_no(
        "Do you want to populate your package? Files like index.typ will be created",
    )?;

    if populate {
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

    if public {
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
    if more {
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

        pkg.categories = split_csv(
            &Text::new("Categories: ")
                .with_help_message("e.g. paper, report (see CATEGORIES.md)")
                .prompt()?,
        );

        pkg.disciplines = split_csv(
            &Text::new("Disciplines: ")
                .with_help_message(
                    "e.g. computer-science (see DISCIPLINES.md), leave empty if generally applicable",
                )
                .prompt()?,
        );
    }

    // Template configuration: template packages must declare the `[template]`
    // section with a directory, an entrypoint relative to it and, optionally, a
    // thumbnail relative to the package root.
    let template_info = if template {
        let mut template = TemplateInfo::new(
            Text::new("Template directory: ")
                .with_validator(required!("This field is required"))
                .with_help_message("e.g. template")
                .with_default("template")
                .prompt()?,
            Text::new("Template entrypoint: ")
                .with_validator(required!("This field is required"))
                .with_help_message("e.g. main.typ (relative to the template directory)")
                .with_default("main.typ")
                .prompt()?,
        );

        let thumbnail = Text::new("Thumbnail (leave empty to skip): ")
            .with_help_message("e.g. thumbnail.png (relative to the package root)")
            .prompt()?;
        if !thumbnail.is_empty() {
            template.thumbnail = Some(thumbnail.into());
        }

        // Template packages must specify at least one category.
        if pkg.categories.is_empty() {
            pkg.categories = split_csv(
                &Text::new("Categories (required for templates): ")
                    .with_validator(required!("This field is required"))
                    .with_help_message("e.g. paper (see CATEGORIES.md)")
                    .prompt()?,
            );
        }

        Some(template)
    } else {
        None
    };

    Ok((pkg, template_info))
}

/// Build the package metadata from command-line arguments.
fn cmd_pkg_info(cmd: &InitArgs) -> Result<PackageInfo> {
    Ok(PackageInfo {
        name: <std::option::Option<std::string::String> as Clone>::clone(&cmd.name)
            .unwrap()
            .into(),
        version: cmd.version,
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
        compiler: cmd.compiler,
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

/// Builds the package metadata and the optional template configuration.
///
/// The interactive mode prompts for every manifest field, including the
/// `[template]` section; the CLI mode reads them from the command arguments.
fn create_manifest_parts(cmd: &mut InitArgs) -> Result<(PackageInfo, Option<TemplateInfo>)> {
    if !cmd.cli {
        interactive_pkg_info(cmd)
    } else {
        Ok((cmd_pkg_info(cmd)?, cmd_template(cmd)?))
    }
}

/// Builds the `[template]` section from command-line arguments.
///
/// `--template-path` and `--template-entrypoint` are required together;
/// `--template-thumbnail` is optional.
fn cmd_template(cmd: &InitArgs) -> Result<Option<TemplateInfo>> {
    if cmd.template_path.is_none()
        && cmd.template_entrypoint.is_none()
        && cmd.template_thumbnail.is_none()
    {
        return Ok(None);
    }

    let (Some(path), Some(entrypoint)) = (&cmd.template_path, &cmd.template_entrypoint) else {
        utpm_bail!(TemplateIncomplete);
    };

    let mut template = TemplateInfo::new(path.clone(), entrypoint.clone());
    if let Some(thumbnail) = &cmd.template_thumbnail {
        template.thumbnail = Some(thumbnail.clone().into());
    }
    Ok(Some(template))
}

/// Populates the project directory with default files.
///
/// Creates a `README.md`, a `LICENSE` (when the license is recognized by
/// SPDX), an `examples/tests.typ` and the package entrypoint. When a template
/// is configured, also creates its directory and its entrypoint, which
/// imports the package and applies it with a show rule.
pub fn populate_project_files(
    project_dir: impl AsRef<Path>,
    pkg: &PackageInfo,
    template: Option<&TemplateInfo>,
) -> Result<()> {
    let project_dir = project_dir.as_ref();
    let mut file = File::create(project_dir.join("README.md"))?; // README.md
    write!(file, "# {}", pkg.name)?;
    if let Some(ref license) = pkg.license
        && let Some(exp) = spdx::license_id(license.as_ref())
    {
        file = File::create(project_dir.join("LICENSE"))?; // LICENSE
        write!(file, "{}", exp.text())?;
    }

    let examples = project_dir.join("examples");
    create_dir_all(&examples)?; // examples
    file = File::create(examples.join("tests.typ"))?; // examples/tests.typ
    write!(
        file,
        "#import \"@local/{}:{}\":\nDo...",
        pkg.name, pkg.version
    )?;

    // Package entrypoint, creating parent directories for nested paths.
    let entrypoint_path = project_dir.join(pkg.entrypoint.as_ref());
    if let Some(parent) = entrypoint_path.parent() {
        create_dir_all(parent)?;
    }
    file = File::create(&entrypoint_path)?; // main.typ
    write!(
        file,
        "// This file is generated by UTPM (https://github.com/typst-community/utpm)"
    )?;

    // Template directory and its entrypoint. The template files should not
    // include the styling code: the entrypoint imports the package and
    // applies it with a show rule.
    if let Some(template) = template {
        let template_dir = project_dir.join(template.path.as_ref());
        create_dir_all(&template_dir)?;
        let template_entrypoint = template_dir.join(template.entrypoint.as_ref());
        if let Some(parent) = template_entrypoint.parent() {
            create_dir_all(parent)?;
        }
        file = File::create(&template_entrypoint)?;
        write!(
            file,
            "// This file is generated by UTPM (https://github.com/typst-community/utpm)\n#import \"@local/{}:{}\": *\n\n// Apply your template with a show rule, e.g.:\n// #show: my-template-function\n",
            pkg.name, pkg.version
        )?;
    }
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
    let manifest = project_dir.join(crate::utils::paths::MANIFEST_FILE);
    utpm_log!(info, "Current typst manifest file: {}", manifest.display());

    // Check if manifest already exists.
    if check_path_file(&manifest) && !cmd.force {
        utpm_log!(
            error,
            "typst.toml already exists. Use --force to overwrite it."
        );
        return Ok(false);
    }

    // Build the package metadata and template configuration.
    let (pkg, template) = create_manifest_parts(cmd)?;

    // Template packages must specify at least one category in `[package]`.
    if template.is_some() && pkg.categories.is_empty() {
        utpm_bail!(MissingCategory);
    }

    // Populate the project with default files if requested.
    if cmd.populate && !get_dry_run() {
        populate_project_files(&project_dir, &pkg, template.as_ref())?;
    }

    // Construct the final manifest.
    let manif = PackageManifest {
        package: pkg,
        tool: ToolInfo::default(),
        template,
        unknown_fields: BTreeMap::new(),
    };

    // Write the manifest to `typst.toml`.
    write_manifest(&manif)?;

    utpm_log!(
        info,
        "File(s) created to {typ}",
        typ = project_dir.display()
    );
    Ok(true)
}
