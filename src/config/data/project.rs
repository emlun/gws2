use std::collections::BTreeMap;

use super::Remote;
use crate::commands::error::Error;
use crate::data::status::BranchMethods;
use crate::util::iter::CollectOrFirstErr;

#[derive(Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Project {
    pub path: String,
    pub main_remote: Remote,
    pub extra_remotes: Vec<Remote>,
}

impl Project {
    pub fn remotes(&self) -> Vec<&Remote> {
        let mut extras: Vec<&Remote> = self.extra_remotes.iter().collect();
        let mut result = vec![&self.main_remote];
        result.append(&mut extras);
        result
    }

    fn local_branches_internal<'repo>(
        &self,
        repo: &'repo git2::Repository,
    ) -> Result<Vec<(git2::Branch<'repo>, Option<git2::Branch<'repo>>)>, Error> {
        Ok(repo
            .branches(Some(git2::BranchType::Local))?
            .flatten()
            .map(|(branch, _)| {
                let upstream = branch.upstream().ok();
                (branch, upstream)
            })
            .collect())
    }

    pub fn current_upstream_heads(
        &self,
        repo: &git2::Repository,
    ) -> Result<BTreeMap<String, git2::Oid>, Error> {
        self.local_branches_internal(repo).and_then(|branches| {
            branches
                .into_iter()
                .flat_map(|(branch, gupstream)| gupstream.map(|gupstream| (branch, gupstream)))
                .map(|(branch, gupstream)| {
                    Ok((
                        branch.branch_name()?.to_string(),
                        gupstream.get().peel_to_commit().unwrap().id(),
                    ))
                })
                .collect_or_first_err()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Project;
    use super::Remote;

    #[test]
    fn remotes_returns_main_remote_and_then_extras() {
        assert_eq!(
            Project {
                path: "foo".to_string(),
                main_remote: Remote {
                    name: "origin".to_string(),
                    url: "git@github.com:foo/boo.git".to_string(),
                },
                extra_remotes: vec![
                    Remote {
                        name: "myone".to_string(),
                        url: "http://coool".to_string(),
                    },
                    Remote {
                        name: "upstream".to_string(),
                        url: "testurl".to_string(),
                    },
                ],
            }
            .remotes(),
            vec![
                Remote {
                    name: "origin".to_string(),
                    url: "git@github.com:foo/boo.git".to_string(),
                },
                Remote {
                    name: "myone".to_string(),
                    url: "http://coool".to_string(),
                },
                Remote {
                    name: "upstream".to_string(),
                    url: "testurl".to_string(),
                },
            ]
            .iter()
            .collect::<Vec<&Remote>>()
        );
    }
}
