#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Remote {
    pub url: String,
    pub name: String,
}

pub struct MaybeNamedRemote {
    pub url: String,
    pub name: Option<String>,
}
impl MaybeNamedRemote {
    pub fn into_named(self) -> Result<Remote, String> {
        Ok(Remote {
            url: self.url,
            name: self
                .name
                .ok_or("Cannot create a named remote from a remote without a name.")?,
        })
    }

    pub fn into_named_or(self, default_name: &str) -> Remote {
        Remote {
            url: self.url,
            name: self.name.unwrap_or_else(|| default_name.to_string()),
        }
    }
}
