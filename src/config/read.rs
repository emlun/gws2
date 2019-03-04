use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::data::Workspace;
use super::error::ConfigError;
use super::parse::legacy;

pub fn read_workspace_file<P: AsRef<Path>>(file_path: P) -> Result<Workspace, ConfigError> {
    let mut contents: String = String::new();

    let mut file = File::open(file_path).map_err(|e| ConfigError::OpenFile(e))?;

    file.read_to_string(&mut contents)
        .map_err(ConfigError::OpenFile)
        .and_then(|_| legacy::parse(&contents))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::super::data::Project;
    use super::super::data::Remote;
    use super::super::data::Workspace;

    use super::read_workspace_file;

    #[test]
    fn good_file_is_parsed_correctly() {
        println!("{:?}", Path::new("foo.txt"));
        assert_eq!(
            read_workspace_file(Path::new("tests").join("example_projects.gws")),
            Ok(Workspace::from(vec![
                Project {
                    path: "foo/bar".to_string(),
                    main_remote: Remote {
                        name: "origin".to_string(),
                        url: "https://github.com/foo/bar.git".to_string(),
                    },
                    extra_remotes: vec![],
                },
                Project {
                    path: "boo".to_string(),
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
                },
                Project {
                    path: "moo".to_string(),
                    main_remote: Remote {
                        name: "origin".to_string(),
                        url: "git@github.com:foo/moo.git".to_string(),
                    },
                    extra_remotes: vec![],
                },
            ],))
        );
    }

}
