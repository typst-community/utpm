use std::fs;
use fmt_derive::Display;
use serde::Serialize;
use tracing::instrument;

use crate::{utils::{
    paths::{c_packages, d_packages},
    state::Result,
}, utpm_println};


#[derive(Serialize, Display)]
struct Data {
    list_namespace: Vec<Namespace> 
}

impl Data {
    pub fn new() -> Self {
        Self {
            list_namespace: vec![]
        }
    }

}

#[derive(Serialize, Display)]
struct Namespace {
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



#[derive(Serialize, Display)]
struct Package {
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


use super::ListTreeArgs;

#[instrument(skip(cmd))]
pub fn run(cmd: &ListTreeArgs) -> Result<bool> {
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
            match package_read(&format!("{}/local/{}", typ, e), e.to_string()) {
                Err(_)=> {namespace_read(&format!("{}/{}",typ,e), e.to_string())?;} ,
                Ok(_) => {},
            };
        }
        Ok(true)
    } else {
        let data = read(typ)?;
        utpm_println!(data);
        return Ok(true)
    }
}

fn read(typ: String) -> Result<Data> {
    let dirs = fs::read_dir(&typ)?;
    let mut data = Data::new();

    for dir_res in dirs {
        let dir = dir_res?;
        let nms = namespace_read(&dir.path().to_str().unwrap().into(), dir.file_name().into_string().unwrap())?;
        data.list_namespace.push(nms);
    }
    Ok(data)
}

fn package_read(typ: &String, name: String) -> Result<Package> {
    let mut pkg = Package::new(name);
    for dir_res in fs::read_dir(&typ)? {
        let dir: fs::DirEntry = dir_res?;
        pkg.list_version.push(dir.file_name().to_str().unwrap().into());
    }
    Ok(pkg)
}

fn namespace_read(typ: &String, name: String) -> Result<Namespace> {
    let mut nms = Namespace::new(name);
    for dir_res in fs::read_dir(&typ)? {
        let dir = dir_res?;
        let pkg = package_read(&dir.path().to_str().unwrap().into(), dir.file_name().to_str().unwrap().into())?;
        nms.list_packages.push(pkg);
    }
    Ok(nms)
}
