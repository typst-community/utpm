use std::fs;
use fmt_derive::{Debug, Display};
use ptree::TreeItem;
use serde::Serialize;
use tracing::instrument;
use std::borrow::Cow;

use crate::{commands::tree::run as R, utils::{
    output::{get_output_format, OutputFormat}, paths::{c_packages, d_packages}, state::Result
}, utpm_println};


#[derive(Serialize, Display, Debug, Clone)]
#[display("{}", list_namespace.iter().map(|ns| ns.to_string()).collect::<Vec<_>>().join(""))]
pub struct Data {
    path: String,
    list_namespace: Vec<Namespace> 
}



impl Data {
    pub fn new(path: String) -> Self {
        Self {
            list_namespace: vec![],
            path
        }
    }
}

impl TreeItem for Data {
    type Child = Namespace;

    fn write_self<W: std::io::Write>(&self, w: &mut W, _style: &ptree::Style) -> std::io::Result<()> {
        write!(w, "{}", self.path)
    }

    fn children(&self) -> Cow<[Namespace]> {
        Cow::Borrowed(&self.list_namespace)
    }
}




#[derive(Serialize, Display, Debug, Clone)]
#[display("\n* {}: \n{}", name, list_packages.iter().map(|ns| ns.to_string()).collect::<Vec<_>>().join("\n"))]
pub struct Namespace {
    name: String,
    list_packages: Vec<Package>
}

impl Namespace {
    pub fn new(name: String) -> Self {
        Self {
            list_packages: vec![],
            name
        }
    }
}

impl TreeItem for Namespace {
    type Child = Package;

    fn write_self<W: std::io::Write>(&self, w: &mut W, _style: &ptree::Style) -> std::io::Result<()> {
        write!(w, "{}", self.name)
    }

    fn children(&self) -> Cow<[Package]> {
        Cow::Borrowed(&self.list_packages)
    }
}



#[derive(Serialize, Display, Debug, Clone)]
#[display("* * {name}: {}", list_version.join(", "))]
pub struct Package {
    name: String,
    list_version: Vec<String>
}

impl Package {
    pub fn new(name: String) -> Self {
        Self {
            list_version: vec![],
            name
        }
    }
}

impl TreeItem for Package {
    type Child = Package; // Peut être n'importe quoi, car sans enfants

    fn write_self<W: std::io::Write>(&self, w: &mut W, _style: &ptree::Style) -> std::io::Result<()> {
        write!(w, "{}: {}", self.name, self.list_version.join(", "))
    }

    fn children(&self) -> Cow<[Self::Child]> {
        Cow::Borrowed(&[]) // Zéro enfant
    }
}


use super::ListTreeArgs;

#[instrument(skip(cmd))]
pub fn run(cmd: &ListTreeArgs) -> Result<bool> {
    // For right now, I'll use this hack because the command tree is deprecated but I don't want to change the
    // code atleast for a while. This works
    if cmd.tree && get_output_format() == OutputFormat::Text {
        return R(cmd)
    }
    let typ: String = d_packages()?;
    if cmd.all {
        let preview: String = c_packages()?;
        let data1 = read(typ)?;
        let data2 = read(preview)?;
        utpm_println!(data1);
        utpm_println!(data2);
        return Ok(true)
    }

    if let Some(list) = &cmd.include {
        let preview: String = c_packages()?;
        for e in list {
            if e == "preview" {
                let data = read(preview)?;
                utpm_println!(data);
                return Ok(true);
            }
            let pkg = package_read(&format!("{}/local/{}", typ, e), e.to_string());

            match pkg {
                Err(_)=> {utpm_println!(namespace_read(&format!("{}/{}",typ,e), e.to_string())?);},
                Ok(data) => {utpm_println!(data)},
            };
        }
        Ok(true)
    } else {
        let data = read(typ)?;
        utpm_println!(data);
        return Ok(true)
    }
}


pub fn read(typ: String) -> Result<Data> {
    let dirs = fs::read_dir(&typ)?;
    let mut data = Data::new(typ);

    for dir_res in dirs {
        let dir = dir_res?;
        let nms = namespace_read(&dir.path().to_str().unwrap().into(), dir.file_name().into_string().unwrap())?;
        data.list_namespace.push(nms);
    }
    Ok(data)
}

pub fn package_read(typ: &String, name: String) -> Result<Package> {
    let mut pkg = Package::new(name);
    for dir_res in fs::read_dir(&typ)? {
        let dir: fs::DirEntry = dir_res?;
        pkg.list_version.push(dir.file_name().to_str().unwrap().into());
    }
    Ok(pkg)
}

pub fn namespace_read(typ: &String, name: String) -> Result<Namespace> {
    let mut nms = Namespace::new(name);
    for dir_res in fs::read_dir(&typ)? {
        let dir = dir_res?;
        let pkg = package_read(&dir.path().to_str().unwrap().into(), dir.file_name().to_str().unwrap().into())?;
        nms.list_packages.push(pkg);
    }
    Ok(nms)
}
