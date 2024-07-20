#[macro_export]
macro_rules! manifest {
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
    }
}