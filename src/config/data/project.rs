use super::Remote;


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Project {
    pub path: String,
    pub remotes: Vec<Remote>,
}
