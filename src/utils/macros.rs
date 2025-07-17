/// A macro to load the `typst.toml` manifest from the current or a specified directory.
///
/// # Usage
///
/// ```rust,ignore
/// // Load from the current directory
/// let manifest = load_manifest!();
///
/// // Load from a specific path
/// let manifest = load_manifest!("/path/to/project");
/// ```
#[macro_export]
macro_rules! load_manifest {
    () => {
        match Manifest::try_find(get_current_dir()?)? {
            Some(val) => Ok(val),
            None => Err(UtpmError::Manifest),
        }?
    };
    ($var:expr) => {
        match Manifest::try_find($var)? {
            Some(val) => Ok(val),
            None => Err(UtpmError::Manifest),
        }?
    };
}

/// A macro to write a `Manifest` struct to a `typst.toml` file.
///
/// # Usage
///
/// ```rust,ignore
/// // Write to `typst.toml` in the current directory
/// write_manifest!(&manifest);
///
/// // Write to a specific path
/// write_manifest!(&manifest => "/path/to/project/typst.toml");
/// ```
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

/// A macro to format the path to a typst package.
///
/// This macro constructs the path to a package based on its namespace, name, and version.
/// It correctly resolves the base directory for `@preview` and other namespaces.
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

/// A macro to configure SSH credentials for git operations.
///
/// It attempts to use an SSH agent first, then falls back to a private key file.
/// The key path can be specified via the `UTPM_KEYPATH` environment variable.
/// A passphrase can be provided via the `UTPM_PASSPHRASE` environment variable.
#[macro_export]
macro_rules! load_creds {
    ($callbacks:expr, $val:expr) => {{
        $callbacks.credentials(|_, username_from_url, _| {
            let binding: String =
                env::var("UTPM_USERNAME").unwrap_or(username_from_url.unwrap_or("git").to_string());
            let username: &str = binding.as_str();
            match Cred::ssh_key_from_agent(username) {
                Ok(cred) => Ok(cred),
                Err(_) => Ok(match env::var("UTPM_PASSPHRASE") {
                    Ok(s) => {
                        info!(passphrase = true);
                        Cred::ssh_key(username, None, Path::new(&$val), Some(s.as_str()))?
                    }
                    Err(_) => {
                        info!(passphrase = false);
                        Cred::ssh_key(username, None, Path::new(&$val), None)?
                    }
                }),
            }
        });
    }};
}

/// A macro to exit a function early with a `UtpmError`.
///
/// # Usage
///
/// ```rust,ignore
/// // Bail with a simple error variant
/// utpm_bail!(Manifest);
///
/// // Bail with an error variant that has arguments
/// utpm_bail!(AlreadyExist, "mypackage".to_string(), "1.0.0".into(), "Info".to_string());
/// ```
#[macro_export]
macro_rules! utpm_bail {
    ($variant:ident) => {
        return Err($crate::utils::state::UtpmError::$variant)
    };
    ($variant:ident, $($arg:expr),+) => {
        return Err($crate::utils::state::UtpmError::$variant($($arg),+))
    };
}

/// A flexible logging macro that adapts to the configured output format.
///
/// This macro supports different log levels and can handle various data types,
/// serializing them to JSON, YAML, etc., if the corresponding output format is selected.
#[macro_export]
macro_rules! utpm_log {
    ($(@g)? $lvl:ident, $data:expr, $($args:expr => $val:expr),+) => {{
        match $crate::utils::output::get_output_format() {
            #[cfg(feature = "output_json")]
            $crate::OutputFormat::Json => tracing::$lvl!($($args = $val),+, data = &$data),
            #[cfg(feature = "output_yaml")]
            $crate::OutputFormat::Yaml => tracing::$lvl!($($args = $val),+, "{}", serde_yaml::to_string(&$data)?),
            #[cfg(feature = "output_toml")]
            $crate::OutputFormat::Toml => tracing::$lvl!($($args = $val),+, "{}", toml::to_string(&$data)?),
            #[cfg(feature = "output_text")]
            $crate::OutputFormat::Text => tracing::$lvl!($($args = $val),+, "{}", $data),
            #[cfg(feature = "output_hjson")]
            $crate::OutputFormat::Hjson => tracing::$lvl!($($args = $val),+, "{}", serde_hjson::ser::to_string(&$data)?),
        }
    }};
    ($(@g)? $lvl:ident, $data:expr,? $($args:expr => $val:expr),*) => {{
        match $crate::utils::output::get_output_format() {
            #[cfg(feature = "output_json")]
            $crate::OutputFormat::Json => tracing::$lvl!($($args = $val),* data = &$data),
            #[cfg(feature = "output_yaml")]
            $crate::OutputFormat::Yaml => tracing::$lvl!($($args = $val),* "{}", serde_yaml::to_string(&$data)?),
            #[cfg(feature = "output_toml")]
            $crate::OutputFormat::Toml => tracing::$lvl!($($args = $val),* "{}", toml::to_string(&$data)?),
            #[cfg(feature = "output_text")]
            $crate::OutputFormat::Text => tracing::$lvl!($($args = $val),* "{}", $data),
            #[cfg(feature = "output_hjson")]
            $crate::OutputFormat::Hjson => tracing::$lvl!($($args = $val),* "{}", serde_hjson::ser::to_string(&$data)?),
        }
    }};
    ($lvl:ident, $($args:expr => $val:expr),+) => {{
        tracing::$lvl!($($args = $val),+)
    }};
    ($lvl:ident, $fmt:expr, $($args:tt)*) => {
        $crate::utpm_log!(@g $lvl, format!($fmt, $($args)*),?)
    };
    ($lvl:ident, $fmt:expr) => {
        $crate::utpm_log!(@g $lvl, format!($fmt),?)
    };
    (@f $lvl:ident, $data:expr) => {
        if $crate::utils::output::get_output_format() == $crate::utils::output::OutputFormat::Text {
            tracing::$lvl!("{}", format!($data))
        } else {
            $crate::utpm_log!($lvl, $data)
        }
    };
    ($fmt:expr, $($args:tt)+) => {
        $crate::utpm_log!(info, $fmt, $($args)+)
    };
    ($data:expr) => {
        $crate::utpm_log!(info, "{}", $data)
    };
}
