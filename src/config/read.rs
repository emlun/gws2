use super::data::user_config::UserConfig;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

use super::data::Workspace;
use super::error::ConfigError;
use super::parse::legacy;

pub fn read_workspace_file<P: AsRef<Path>>(file_path: P) -> Result<Workspace, ConfigError> {
    let mut contents: String = String::new();

    let mut file = File::open(file_path).map_err(ConfigError::OpenFile)?;

    file.read_to_string(&mut contents)
        .map_err(ConfigError::OpenFile)
        .and_then(|_| legacy::parse(&contents))
}

pub fn read_config_toml(content: &str) -> Result<UserConfig, ConfigError> {
    Ok(toml::from_str(&content).map_err(|e| {
        ConfigError::SyntaxError(match e.line_col() {
            Some((line, col)) => format!(
                "TOML syntax error at line {}, column {}: {:?}",
                line, col, e
            ),
            None => format!("TOML syntax error at (unknown position): {:?}", e),
        })
    })?)
}

pub fn read_config_file<P: AsRef<Path>>(file_path: P) -> Result<UserConfig, ConfigError> {
    let mut contents: String = String::new();
    let mut file = File::open(file_path).map_err(ConfigError::OpenFile)?;
    file.read_to_string(&mut contents)
        .map_err(ConfigError::OpenFile)?;
    read_config_toml(&contents)
}

#[cfg(test)]
mod tests {
    use ansi_term::Colour;
    use std::path::Path;

    use super::super::data::Project;
    use super::super::data::Remote;
    use super::super::data::Workspace;
    use crate::color::palette::Palette;

    use super::read_config_toml;
    use super::read_workspace_file;
    use super::ConfigError;

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

    #[test]
    fn palette_is_parsed_correctly() -> Result<(), ConfigError> {
        let config_content = r#"
        [palette]
            branch = 13
            clean = 9
            cloning = 14
            dirty = 9
            error = 9
            missing = [255, 0x64, 0x2b]
            repo = "green"
            repo_exists = 10
        "#;

        let expected = Palette {
            branch: Colour::Fixed(13).normal(),
            clean: Colour::Fixed(9).normal(),
            cloning: Colour::Fixed(14).normal(),
            dirty: Colour::Fixed(9).normal(),
            error: Colour::Fixed(9).normal(),
            missing: Colour::RGB(0xff, 0x64, 0x2b).normal(),
            repo: Colour::Green.normal(),
            repo_exists: Colour::Fixed(10).normal(),
        };

        let config = read_config_toml(&config_content)?;
        assert_eq!(config.palette(), Some(expected));

        Ok(())
    }

}
