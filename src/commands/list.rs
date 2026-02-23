use fmt_derive::{Debug, Display};
use ptree::{TreeItem, print_tree};
use serde::Serialize;
use std::borrow::Cow;
use std::fs;
use std::path::Path;
use tracing::instrument;

use crate::{
    utils::{
        output::{OutputFormat, get_output_format},
        paths::{package_cache_path, package_path},
        state::Result,
    },
    utpm_log,
};

/// Represents a collection of namespaces at a specific path.
#[derive(Serialize, Display, Debug, Clone)]
#[display("{}", list_namespace.iter().map(|ns| ns.to_string()).collect::<Vec<_>>().join(""))]
pub struct Data {
    path: String,
    list_namespace: Vec<Namespace>,
}

impl Data {
    /// Creates a new `Data` instance.
    pub fn new(path: String) -> Self {
        Self {
            list_namespace: vec![],
            path,
        }
    }
}

impl TreeItem for Data {
    type Child = Namespace;

    fn write_self<W: std::io::Write>(
        &self,
        w: &mut W,
        _style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(w, "{}", self.path)
    }

    fn children(&self) -> Cow<'_, [Namespace]> {
        Cow::Borrowed(&self.list_namespace)
    }
}

/// Represents a package namespace containing multiple packages.
#[derive(Serialize, Display, Debug, Clone)]
#[display("\n* {}: \n{}", name, list_packages.iter().map(|ns| ns.to_string()).collect::<Vec<_>>().join("\n"))]
pub struct Namespace {
    name: String,
    list_packages: Vec<Package>,
}

impl Namespace {
    /// Creates a new `Namespace` instance.
    pub fn new(name: String) -> Self {
        Self {
            list_packages: vec![],
            name,
        }
    }
}

impl TreeItem for Namespace {
    type Child = Package;

    fn write_self<W: std::io::Write>(
        &self,
        w: &mut W,
        _style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(w, "{}", self.name)
    }

    fn children(&self) -> Cow<'_, [Package]> {
        Cow::Borrowed(&self.list_packages)
    }
}

/// Represents a package with its available versions.
#[derive(Serialize, Display, Debug, Clone)]
#[display("* * {name}: {}", list_version.join(", "))]
pub struct Package {
    name: String,
    list_version: Vec<String>,
}

impl Package {
    /// Creates a new `Package` instance.
    pub fn new(name: String) -> Self {
        Self {
            list_version: vec![],
            name,
        }
    }
}

impl TreeItem for Package {
    type Child = Package; // Can be anything, since it has no children

    fn write_self<W: std::io::Write>(
        &self,
        w: &mut W,
        _style: &ptree::Style,
    ) -> std::io::Result<()> {
        write!(w, "{}: {}", self.name, self.list_version.join(", "))
    }

    fn children(&self) -> Cow<'_, [Self::Child]> {
        Cow::Borrowed(&[]) // No children
    }
}

use super::ListTreeArgs;

/// Lists packages in local storage.
///
/// Can display as a simple list or as a tree structure, depending on the
/// command-line arguments and output format.
#[instrument(skip(cmd))]
pub async fn run(cmd: &ListTreeArgs) -> Result<bool> {
    // If tree view is requested with text output, use the tree-specific function.
    if cmd.tree && get_output_format() == OutputFormat::Text {
        return run_tree(cmd);
    }
    let typ = package_path()?;
    // If `--all` is specified, list packages from both data and cache directories.
    if cmd.all {
        let preview = package_cache_path()?;
        let data1 = read(&typ)?;
        let data2 = read(&preview)?;
        utpm_log!(data1);
        utpm_log!(data2);
        return Ok(true);
    }

    // If specific packages/namespaces are included, list only those.
    if let Some(list) = &cmd.include {
        let preview = package_cache_path()?;
        for e in list {
            if e == "preview" {
                let data = read(&preview)?;
                utpm_log!(data);
                return Ok(true);
            }
            let pkg = package_read(typ.join("local").join(e), e.to_string());
            match pkg {
                Err(_) => {
                    utpm_log!(namespace_read(typ.join(e), e.to_string())?);
                },
                Ok(data) => {
                    utpm_log!(data)
                },
            };
        }
        Ok(true)
    } else {
        // By default, list packages from the data directory.
        let data = read(&typ)?;
        utpm_log!(data);
        return Ok(true);
    }
}

/// Reads all namespaces and packages from a given directory path.
pub fn read(typ: impl AsRef<Path>) -> Result<Data> {
    let typ_path = typ.as_ref();
    let dirs = fs::read_dir(typ_path)?;
    let mut data = Data::new(typ_path.display().to_string());

    for dir_res in dirs {
        let dir = dir_res?;
        let nms = namespace_read(dir.path(), dir.file_name().to_string_lossy().to_string())?;
        data.list_namespace.push(nms);
    }
    Ok(data)
}

/// Reads all versions of a specific package.
pub fn package_read(typ: impl AsRef<Path>, name: String) -> Result<Package> {
    let mut pkg = Package::new(name);
    for dir_res in fs::read_dir(typ)? {
        let dir: fs::DirEntry = dir_res?;
        pkg.list_version
            .push(dir.file_name().to_string_lossy().to_string());
    }
    pkg.list_version.sort();
    Ok(pkg)
}

/// Reads all packages within a specific namespace.
pub fn namespace_read(typ: impl AsRef<Path>, name: String) -> Result<Namespace> {
    let mut nms = Namespace::new(name);
    for dir_res in fs::read_dir(typ)? {
        let dir = dir_res?;
        let pkg = package_read(dir.path(), dir.file_name().to_string_lossy().to_string())?;
        nms.list_packages.push(pkg);
    }
    Ok(nms)
}

/// Displays the packages in a tree format.
#[instrument(skip(cmd))]
pub fn run_tree(cmd: &ListTreeArgs) -> Result<bool> {
    utpm_log!(trace, "executing list command with tree format");
    let packages = package_path()?;
    if cmd.all {
        let cache = package_cache_path()?;
        let packages_data = read(&packages)?;
        let cache_data = read(&cache)?;
        print_tree(&packages_data)?;
        print_tree(&cache_data)?;
    } else if let Some(list) = &cmd.include {
        let preview = package_cache_path()?;
        for e in list {
            if e == "preview" {
                let data = read(&preview)?;
                print_tree(&data)?;
                return Ok(true);
            }
            let pkg = package_read(packages.join("local").join(e), e.to_string());
            match pkg {
                Err(_) => print_tree(&namespace_read(packages.join(e), e.to_string())?),
                Ok(data) => print_tree(&data),
            }?;
        }
    } else {
        let data = read(&packages)?;
        print_tree(&data)?;
    }
    Ok(true)
}
