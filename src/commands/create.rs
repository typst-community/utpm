use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, create_dir_all, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use inquire::{required, validator::Validation, Select, Text};
use owo_colors::OwoColorize;
use semver::Version;
use toml::Table;
use typst_project::manifest::{
    author::{Author, Website},
    categories::Category,
    disciplines::Discipline,
    ident::Ident,
    license::License,
    package::Package,
    tool::Tool,
    Manifest,
};

use crate::utils::{
    paths::{check_path_file, get_current_dir},
    specs::Extra,
    state::Result,
};

use super::CreateInitArgs;

pub fn run(cmd: &mut CreateInitArgs) -> Result<bool> {
    let curr = get_current_dir()?;
    let typ = curr.clone() + "/typst.toml";

    let mut extra = Extra::default();
    extra.namespace = cmd.namespace.to_owned();

    let mut authors: HashSet<Author> = HashSet::new();
    // temp
    if let Some(auts) = &cmd.authors {
        for e in auts {
            authors.insert(Author::from_str(&e)?);
        }
    }

    let mut keywords: HashSet<String> = HashSet::new();
    // temp
    if let Some(auts) = &cmd.keywords {
        for e in auts {
            keywords.insert(e.clone());
        }
    }

    let mut exclude: HashSet<PathBuf> = HashSet::new();
    // temp
    if let Some(auts) = &cmd.exclude {
        for e in auts {
            exclude.insert(e.into());
        }
    }

    let mut categories: HashSet<Category> = HashSet::new();
    // temp
    if let Some(auts) = &cmd.categories {
        for e in auts {
            categories.insert(*e);
        }
    }

    let mut disciplines: HashSet<Discipline> = HashSet::new();
    // temp
    if let Some(auts) = &cmd.disciplines {
        for e in auts {
            disciplines.insert(*e);
        }
    }

    let mut pkg = Package {
        name: Ident::from_str(cmd.name.to_owned().unwrap_or("temp".into()).as_str())?,
        version: cmd.version.to_owned(),
        entrypoint: cmd.entrypoint.to_owned().into(),
        authors,
        license: License::from_str(cmd.license.to_owned().unwrap_or("MIT".into()).as_str())?,
        description: cmd.description.to_owned().unwrap_or("".into()),
        repository: if cmd.repository.is_none() {
            None
        } else {
            Some(Website::from_str(
                cmd.repository.to_owned().unwrap_or("".into()).as_str(),
            )?)
        },
        homepage: if cmd.homepage.is_none() {
            None
        } else {
            Some(Website::from_str(
                cmd.homepage.to_owned().unwrap_or("".into()).as_str(),
            )?)
        },
        keywords,
        compiler: cmd.compiler.to_owned(),
        exclude,
        categories,
        disciplines,
    };

    //let mut tmpl: Template = Template::new(cmd.template, entrypoint, thumbnail)

    if check_path_file(&typ) && !cmd.force {
        return Ok(false);
    }

    if cmd.force {
        println!(
            "{} {}",
            "WARNING:".bold().yellow(),
            "--force is a dangerous flag, use it cautiously".bold()
        );
    }

    if !cmd.cli {
        let choice = vec!["yes", "no"];
        let public = Select::new("Do you want to make your package public? Questions are on authors, license, description", choice.clone()).prompt()?;
        let more = Select::new("Do you want more questions to customise your package? Questions are on repository url, homepage url, keywords, compiler version, excluded files, categories and disciplines", choice.clone()).prompt()?;
        let extra_opts = Select::new(
            "Do you want to specify informations of utpm? Questions are on the namespace",
            choice.clone(),
        )
        .prompt()?;
        let template = Select::new("Do you want to create a template?", choice.clone()).prompt()?;
        let popu = Select::new(
            "Do you want to populate your package? Files like index.typ will be created",
            choice.clone(),
        )
        .prompt()?;

        if popu == "yes" {
            cmd.populate = true;
        }

        pkg.name = Ident::from_str(
            Text::new("Name: ")
                .with_validator(required!("This field is required"))
                .with_help_message("e.g. my_example")
                .prompt()?
                .as_str(),
        )?;

        pkg.version = Version::parse(
            Text::new("Version: ")
                .with_validator(required!("This field is required"))
                .with_validator(&|obj: &str| {
                    return match Version::parse(&obj) {
                        Ok(_) => Ok(Validation::Valid),
                        Err(_) => Ok(Validation::Invalid(
                            "A correct version must be types (check semVer)".into(),
                        )),
                    };
                })
                .with_help_message("e.g. 1.0.0 (SemVer)")
                .with_default("1.0.0")
                .prompt()?
                .as_str(),
        )?;

        pkg.entrypoint = PathBuf::from(
            Text::new("Entrypoint: ")
                .with_validator(required!("This field is required"))
                .with_help_message("e.g. main.typ")
                .with_default("main.typ")
                .prompt()?,
        );

        if public == "yes" {
            pkg.authors = Text::new("Authors: ")
                .with_help_message("e.g. Thumus,Somebody,Somebody Else")
                .prompt()?
                .split(",")
                .map(|f| Author::from_str(f.to_string().as_str()).unwrap())
                .collect::<HashSet<Author>>();

            pkg.license = License::from_str(
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
                    .as_str(),
            )?;

            pkg.description = Text::new("description: ")
                .with_help_message("e.g. A package")
                .prompt()?;
        }
        if more == "yes" {
            pkg.repository = Some(Website::from_str(
                Text::new("URL of the repository: ")
                    .with_help_message("e.g. https://github.com/Thumuss/utpm")
                    .prompt()?
                    .as_str(),
            )?);
            pkg.homepage = Some(Website::from_str(
                Text::new("Homepage: ")
                    .with_help_message("e.g. anything")
                    .prompt()?
                    .as_str(),
            )?);
            pkg.keywords = Text::new("Keywords: ")
                .with_help_message("e.g. Typst,keyword")
                .prompt()?
                .split(",")
                .map(|f| f.to_string())
                .collect::<HashSet<String>>();

            pkg.compiler = Some(Version::parse(
                Text::new("Compiler version required: ")
                    .with_help_message("e.g. 0.7.0")
                    .with_validator(&|obj: &str| {
                        return match Version::parse(&obj) {
                            Ok(_) => Ok(Validation::Valid),
                            Err(_) => Ok(Validation::Invalid(
                                "A correct version must be types (check semVer)".into(),
                            )),
                        };
                    })
                    .prompt()?
                    .as_str(),
            )?);
            pkg.exclude = Text::new("Exclude: ")
                .with_help_message("e.g. backup/mypassword.txt,.env")
                .prompt()?
                .split(",")
                .filter(|f| f.len() > 0)
                .map(|f| PathBuf::from_str(f).unwrap())
                .collect::<HashSet<PathBuf>>();
        }

        if extra_opts == "yes" {
            extra.namespace = Some(
                Text::new("Namespace: ")
                    .with_help_message("e.g. backup/mypassword.txt,.env")
                    .with_default("local")
                    .prompt()?
                    .to_string(),
            )
        }

        if template == "yes" {
            //todo
        }
    }

    if cmd.populate {
        let mut file = File::create(curr.clone() + "/README.md")?; // README.md
        file.write_all(("# ".to_string() + &pkg.name.clone()).as_bytes())?;
        if let Some(exp) = spdx::license_id(pkg.license.to_string().as_str()) {
            file = File::create(curr.clone() + "/LICENSE")?; // LICENSE
            file.write_all(exp.text().as_bytes())?;
        }

        create_dir_all(curr.clone() + "/examples")?; // examples
        let examples = curr.clone() + "/examples";
        file = File::create(examples + "/tests.typ")?; // examples/texts.typ
        let fm = format!(
            "#import \"@{}/{}:{}\": *\nDo...",
            extra.namespace.clone().unwrap_or("preview".to_string()),
            pkg.name.clone(),
            pkg.version.clone().to_string()
        );
        file.write_all(fm.as_bytes())?;
        file = File::create(pkg.entrypoint.clone())?; // main.typ
        file.write_all(b"// This file is generated by UTPM (https://github.com/Thumuss/utpm)")?;
    }
    let mut keys: BTreeMap<String, Table> = BTreeMap::new();
    keys.insert("utpm".into(), Table::try_from(extra.clone())?);

    let manif = Manifest {
        package: pkg,
        tool: if extra.namespace.is_none()
            && (extra.dependencies.is_none() || extra.dependencies.unwrap().len() == 0)
        {
            None
        } else {
            Some(Tool { keys })
        },
        template: None,
    };

    let tomlfy: String = toml::to_string_pretty(&manif)?;
    fs::write("./typst.toml", tomlfy)?;
    println!("{}", format!("File created to {typ}").bold().to_string());
    Ok(true)
}
