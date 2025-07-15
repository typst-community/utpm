use ptree::print_tree;
use tracing::instrument;

use crate::{commands::{list::{namespace_read, package_read, read, run as R}}, utils::{
    output::{get_output_format, OutputFormat}, paths::{c_packages, d_packages}, state::Result
}};

use super::ListTreeArgs;

#[instrument(skip(cmd))]
pub fn run(cmd: &ListTreeArgs) -> Result<bool> {
    if get_output_format() != OutputFormat::Text {
        return R(cmd);
    }
    let typ: String = d_packages()?;
    if cmd.all {
        let preview: String = c_packages()?;
        let data1 = read(typ)?;
        let data2 = read(preview)?;
        print_tree(&data1)?;
        print_tree(&data2)?;
        return Ok(true)
    }

    if let Some(list) = &cmd.include {
        let preview: String = c_packages()?;
        for e in list {
            if e == "preview" {
                let data = read(preview)?;
                print_tree(&data)?;
                return Ok(true);
            }
            let pkg = package_read(&format!("{}/local/{}", typ, e), e.to_string());
            match pkg {
                Err(_)=> {print_tree(&namespace_read(&format!("{}/{}",typ,e), e.to_string())?)},
                Ok(data) => {print_tree(&data)},
            }?;
        }
        Ok(true)
    } else {
        let data = read(typ)?;
        print_tree(&data)?;
        return Ok(true)
    }
}
