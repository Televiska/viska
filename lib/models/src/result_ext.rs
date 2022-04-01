pub trait ResultExt {
    fn log_error(&self, reason: impl AsRef<str>);
}

impl<T, E: std::fmt::Display> ResultExt for Result<T, E> {
    fn log_error(&self, reason: impl AsRef<str>) {
        if let Err(e) = self {
            common::log::error!("{}: {}", reason.as_ref(), e);
        }
    }
}
