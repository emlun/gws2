#[derive(Debug)]
#[derive(PartialEq)]
pub enum Error {
  Git2Error(git2::Error),
  RepositoryMissing,
}

impl From<git2::Error> for Error {
  fn from(e: git2::Error) -> Error {
    Error::Git2Error(e)
  }
}
