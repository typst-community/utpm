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
            $crate::OutputFormat::Toml => tracing::$lvl!($($($args = $val),*,)? "{}", toml::to_string(&$data).unwrap()),
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
            $crate::OutputFormat::Toml => tracing::$lvl!($($args = $val),* "{}", toml::to_string(&$data)?),
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
