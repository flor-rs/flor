use log::error;

pub trait LogError {
    fn log_error(self, err_log: impl AsRef<str>);
}

impl<T, E: std::fmt::Debug> LogError for Result<T, E> {
    fn log_error(self, err_log: impl AsRef<str>) {
        if let Err(err) = self {
            error!("{}: {:?}", err_log.as_ref(), err);
        }
    }
}
