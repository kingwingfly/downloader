use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(module, visibility(pub(crate)), context(suffix(Error)))]
pub enum ConfigError {
    #[snafu(context(false))]
    NotAccessible { source: keyring::error::Error },
    #[snafu(context(false))]
    SourceError { source: config::ConfigError },
    #[snafu(context(false))]
    EncrypterError { source: rsa::errors::Error },
    #[snafu(context(false))]
    WrongEncrypterSerde { source: serde_json::Error },
    #[snafu(context(suffix(false)))]
    ConfigDirUnknown,
}

pub type ConfigResult<T> = Result<T, ConfigError>;
