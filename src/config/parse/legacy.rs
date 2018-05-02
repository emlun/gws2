use ::std::str::FromStr;

use config::data::MaybeNamedRemote;
use config::data::Project;
use config::data::Remote;
use config::data::Workspace;
use config::error::ConfigError;

pub fn parse(input: &str) -> Result<Workspace, ConfigError> {
    input.parse()
}

impl FromStr for Workspace {
    type Err = ConfigError;

    fn from_str(config: &str) -> Result<Self, Self::Err> {
        let mut projects: Vec<Project> = Vec::new();

        let nonempty_lines = config
            .trim()
            .lines()
            .flat_map(|s| s.split('#').next())
            .map(&str::trim)
            .filter(|s| !s.is_empty());

        for line in nonempty_lines {
            projects.push(try!(line.parse()));
        }

        Ok(Workspace {
            projects,
        })
    }
}

impl FromStr for Project {
    type Err = ConfigError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut segments =
            try!(line
                .trim()
                .split('#')
                .next()
                .ok_or(ConfigError::InternalError("Failed to remove line comment".to_string()))
            )
            .split('|')
            .map(&str::trim);

        let path: String = try!(
            segments.next()
                .ok_or(ConfigError::SyntaxError("Expected project path, found empty line.".to_string()))
        ).to_string();

        let  remote_parses: Vec<Result<MaybeNamedRemote, ConfigError>> = segments.map(MaybeNamedRemote::from_str).collect();
        let mut maybe_remotes: Vec<MaybeNamedRemote> = Vec::new();

        for result in remote_parses {
            maybe_remotes.push(try!(result));
        }

        let mut maybe_remotes_iter = maybe_remotes.into_iter();

        let first_remote: Remote = try!(
            maybe_remotes_iter.next()
                .ok_or(ConfigError::InvalidConfig("At least one remote is required".to_string()))
        )
            .to_named_or("origin")
        ;

        let second_remote: Option<Remote> = maybe_remotes_iter.next()
            .map(|r| r.to_named_or("upstream"));

        let mut all_remotes: Vec<Remote> = Vec::new();
        all_remotes.push(first_remote);
        second_remote.into_iter().for_each(|r| all_remotes.push(r));

        for r in maybe_remotes_iter {
            all_remotes.push(
                try!(
                    r.to_named()
                    .map_err(|_|
                        ConfigError::SyntaxError("Remotes past the 2nd must be given an explicit name.".to_string()))
            ));
        }

        Ok(Project {
            path,
            remotes: all_remotes,
        })
    }
}

impl FromStr for MaybeNamedRemote {
    type Err = ConfigError;

    fn from_str(segment: &str) -> Result<Self, Self::Err> {
        let mut parts = segment.split(|c| c == ' ' || c == '\t')
            .map(&str::trim)
            .filter(|s| !s.is_empty())
        ;

        let url: String = try!(
            parts.next()
                .ok_or(ConfigError::SyntaxError("All remotes must specify a URL.".to_string()))
        ).to_string();
        let name: Option<String> = parts.next().map(String::from);

        Ok(MaybeNamedRemote {
            url,
            name,
        })
    }
}


#[cfg(test)]
mod tests {
    use ::std::str::FromStr;
    use super::Project;
    use super::Remote;
    use super::Workspace;

    #[test]
    fn project_must_have_path() {
        assert!(Project::from_str(" ").is_err());
    }

    #[test]
    fn project_must_have_one_remote() {
        assert!(Project::from_str("foo").is_err());
    }

    #[test]
    fn remote_must_have_url() {
        assert!(Project::from_str("foo |").is_err());
    }

    #[test]
    fn minimal_line() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("origin"),
                        }
                    ],
                }
            )
        );
    }

    #[test]
    fn one_named_remote() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git github"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("github"),
                        }
                    ],
                }
            )
        );
    }

    #[test]
    fn line_comment_is_ignored() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git # | https:/github.com/foo/foo.git ignored"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("origin"),
                        }
                    ],
                }
            )
        );
    }

    #[test]
    fn two_unnamed_remotes() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git | git@github.com:bar/foo.git"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("origin"),
                        },
                        Remote {
                            url: String::from("git@github.com:bar/foo.git"),
                            name: String::from("upstream"),
                        },
                    ],
                }
            )
        );
    }

    #[test]
    fn second_remote_must_have_url() {
        assert!(Project::from_str("foo | git@github.com:foo/foo.git |").is_err());
    }

    #[test]
    fn two_named_remotes() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git github-foo | git@github.com:bar/foo.git github-bar"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("github-foo"),
                        },
                        Remote {
                            url: String::from("git@github.com:bar/foo.git"),
                            name: String::from("github-bar"),
                        },
                    ],
                }
            )
        );
    }

    #[test]
    fn two_named_remotes_one_unnamed() {
        assert_eq!(
            Project::from_str("foo | git@github.com:foo/foo.git github-foo | git@github.com:bar/foo.git github-bar | git@github.com:boo/foo.git github-boo"),
            Ok(
                Project {
                    path: String::from("foo"),
                    remotes: vec![
                        Remote {
                            url: String::from("git@github.com:foo/foo.git"),
                            name: String::from("github-foo"),
                        },
                        Remote {
                            url: String::from("git@github.com:bar/foo.git"),
                            name: String::from("github-bar"),
                        },
                        Remote {
                            url: String::from("git@github.com:boo/foo.git"),
                            name: String::from("github-boo"),
                        },
                    ],
                }
            )
        );
    }

    #[test]
    fn third_remote_must_have_url() {
        let line = "foo | git@github.com:foo/foo.git | git@github.com:bar/foo.git |";
        assert!(
            Project::from_str(line).is_err(),
            format!("This line should result in an error: {}", line)
        );
    }

    #[test]
    fn third_remote_must_have_name() {
        let line = "foo | git@github.com:foo/foo.git | git@github.com:bar/foo.git | git@github.com:boo/foo.git";
        assert!(
            Project::from_str(line).is_err(),
            format!("This line should result in an error: {}", line)
        );
    }

    #[test]
    fn example_config_is_parsed_correctly() {
        let config = "
            foo/bar | https://github.com/foo/bar.git
            boo | git@github.com:foo/boo.git | http://coool myone | testurl upstream

            # Just a comment

            moo | git@github.com:foo/moo.git # | http://coool myone | testurl upstream
        ";

        let workspace: Result<Workspace, _> = config.parse();

        assert_eq!(
            workspace,
            Ok(Workspace {
                projects: vec![
                    Project {
                        path: "foo/bar".to_string(),
                        remotes: vec![
                            Remote {
                                name: "origin".to_string(),
                                url: "https://github.com/foo/bar.git".to_string(),
                            },
                        ],
                    },
                    Project {
                        path: "boo".to_string(),
                        remotes: vec![
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
                        ],
                    },
                    Project {
                        path: "moo".to_string(),
                        remotes: vec![
                            Remote {
                                name: "origin".to_string(),
                                url: "git@github.com:foo/moo.git".to_string(),
                            },
                        ],
                    },
                ],
            })
        )
    }

}
