#[derive(Debug)]
pub enum ConfigError {
    InvalidConfig(String),
    InternalError(String),
    OpenFile(::std::io::Error),
    SyntaxError(String),
}

use self::ConfigError::InvalidConfig;
use self::ConfigError::InternalError;
use self::ConfigError::OpenFile;
use self::ConfigError::SyntaxError;

impl PartialEq for ConfigError {

    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (&InvalidConfig(ref a), &InvalidConfig(ref b)) => a == b,
            (&InternalError(ref a), &InternalError(ref b)) => a == b,
            (&OpenFile(_), &OpenFile(_)) => false,
            (&SyntaxError(ref a), &SyntaxError(ref b)) => a == b,
            (_, _) => false,
        }
    }
}
