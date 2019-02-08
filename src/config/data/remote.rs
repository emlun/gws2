#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(PartialEq)]
pub struct Remote {
  pub url: String,
  pub name: String,
}

pub struct MaybeNamedRemote {
  pub url: String,
  pub name: Option<String>,
}
impl MaybeNamedRemote {
  pub fn to_named(self) -> Result<Remote, String> {
    Ok(Remote {
      url: self.url,
      name: try!(self.name.ok_or("Cannot create a named remote from a remote without a name.")),
    })
  }

  pub fn to_named_or(self, default_name: &str) -> Remote {
    Remote {
      url: self.url,
      name: self.name.unwrap_or(default_name.to_string()),
    }
  }
}
