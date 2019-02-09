#[derive(Debug)]
#[derive(Eq)]
#[derive(Ord)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub struct Branch {
  pub name: Option<String>,
  pub upstream_name: Option<Option<String>>,
}
