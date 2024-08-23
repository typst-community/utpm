#[macro_export]
macro_rules! load_manifest {
    () => {
        match Manifest::try_find(get_current_dir()?)? {
            Some(val) => Ok(val),
            None => Err(Error::empty(ErrorKind::Manifest)),
        }?
    };
    ($var:expr) => {
        match Manifest::try_find($var)? {
            Some(val) => Ok(val),
            None => Err(Error::empty(ErrorKind::Manifest)),
        }?
    };
}

#[macro_export]
macro_rules! write_manifest {
    ($var:expr => $path:expr) => {
        let tomlfy: String = toml::to_string_pretty($var)?;
        fs::write($path, tomlfy)?
    };
    ($var:expr) => {
        let tomlfy: String = toml::to_string_pretty($var)?;
        fs::write("./typst.toml", tomlfy)?
    };
}

#[macro_export]
macro_rules! format_package {
    ($namespace:expr) => {{
        (format!(
            "{}/{}",
            if $namespace == "preview" {
                info!("preview found, cache dir use");
                c_packages()?
            } else {
                info!("no preview found, data dir use");
                d_packages()?
            },
            $namespace
        ))
    }};

    ($namespace:expr, $package:expr) => {{
        (format!("{}/{}", format_package!($namespace), $package))
    }};

    ($namespace:ident, $package:ident, $major:ident, $minor:ident, $patch:ident) => {{
        (format!(
            "{}/{}.{}.{}",
            format_package!($namespace, $package),
            $major,
            $minor,
            $patch
        ))
    }};
}
