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
        $crate::utils::try_find(&$crate::utils::paths::get_current_dir()?)?
    };
    ($var:expr) => {
        $crate::utils::try_find($var)?
    };
}

/// A macro to write a `Manifest` struct to a `typst.toml` file.
///
/// # Usage
///
/// ```rust,ignore
/// let manifest: Manifest = load_manifest!().unwrap();
/// // Write to `typst.toml` in the current directory
/// write_manifest!(&manifest);
///
/// // Write to a specific path
/// write_manifest!(&manifest => "/path/to/projectMANIFEST_PATH");
/// ```
#[macro_export]
macro_rules! write_manifest {
    ($var:expr => $path:expr) => {
        let tomlfy: String = toml::to_string_pretty($var)?;
        if !$crate::utils::dryrun::get_dry_run() {
            std::fs::write($path, tomlfy)?
        }
    };
    ($var:expr) => {
        let tomlfy: String = toml::to_string_pretty($var)?;
        if !$crate::utils::dryrun::get_dry_run() {
            std::fs::write(".MANIFEST_PATH", tomlfy)?
        }
    };
}

/// A macro to format the path to a typst package.
///
/// This macro constructs the path to a package based on its namespace, name, and version.
/// It correctly resolves the base directory for `@preview` and other namespaces.
/// 
/// # Example
/// 
/// ```rust,ignore
/// // This is a example from the unlink command.
/// // We are checking every possible output but at the end
/// // it will be formated to a path
/// ...
///  if let Some(cap) = re_all.captures(packages.as_str()) {
///     let (_, [namespace, package, major, minor, patch]) = cap.extract();
///     path = format_package!(namespace, package, major, minor, patch);
/// } else if let Some(cap) = re_name.captures(packages.as_str()) {
///     let (_, [namespace, package]) = cap.extract();
///     path = format_package!(namespace, package);
/// } else if let Some(cap) = re_namespace.captures(packages.as_str()) {
///     let (_, [namespace]) = cap.extract();
///     path = format_package!(namespace);
/// } else {
///     utpm_bail!(PackageNotValid);
/// }
/// ...
/// ```
#[macro_export]
macro_rules! format_package {
    ($namespace:expr) => {{
        (format!(
            "{}/{}",
            if $namespace == "preview" {
                $crate::utpm_log!("preview found, cache dir use");
                $crate::utils::paths::c_packages()?
            } else {
                $crate::utpm_log!("no preview found, data dir use");
                $crate::utils::paths::d_packages()?
            },
            $namespace
        ))
    }};

    ($namespace:expr, $package:expr) => {{ (format!("{}/{}", $crate::format_package!($namespace), $package)) }};

    ($namespace:ident, $package:ident, $major:ident, $minor:ident, $patch:ident) => {{
        (format!(
            "{}/{}.{}.{}",
            $crate::format_package!($namespace, $package),
            $major,
            $minor,
            $patch
        ))
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
/// 
/// Works like (more or less) the `info!`, `trace!`, etc... macros
/// 
/// # Examples
/// Print a simple text: 
/// ```rust,ignore
/// utpm_log!(info, "Hello! I'm secretly alive...");
/// utpm_log!("Don't listen to him! He is not!");
/// utpm_log!(error, "It's urgent.");
/// ``` 
/// Print values directly accessible from a JSON processor:
/// ```rust,ignore
/// let path = "/your/heart/";
/// utpm_log!(trace, "Accessing your heart...", "path" => path);
/// ``` 
/// 
/// TODO: Finish examples
#[macro_export]
macro_rules! utpm_log {
    ($(@g)? $lvl:ident, $data:expr $(, $($args:expr => $val:expr),*)?) => {{
        match $crate::utils::output::get_output_format() {
            #[cfg(feature = "output_json")]
            $crate::OutputFormat::Json => tracing::$lvl!($($($args = $val),*,)? "{}", serde_json::to_string(&$data).unwrap()),
            #[cfg(feature = "output_yaml")]
            $crate::OutputFormat::Yaml => tracing::$lvl!($($($args = $val),*,)? "{}", serde_yaml::to_string(&$data).unwrap()),
            #[cfg(feature = "output_toml")]
            $crate::OutputFormat::Toml => tracing::$lvl!($($($args = $val),*,)? "{}", toml::to_string(&$data).unwrap()),
            #[cfg(feature = "output_text")]
            $crate::OutputFormat::Text => tracing::$lvl!($($($args = $val),*,)? "{}", $data),
            #[cfg(feature = "output_hjson")]
            $crate::OutputFormat::Hjson => tracing::$lvl!($($($args = $val),*,)? "{}", serde_hjson::to_string(&$data).unwrap()),
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
