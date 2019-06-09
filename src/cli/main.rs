use clap::ArgMatches;
use clap::Shell;
use directories::ProjectDirs;
use std::path::Path;
use std::path::PathBuf;

use crate::color::palette::Palette;
use crate::commands::common::exit_codes;
use crate::commands::common::Command;
use crate::config::data::user_config::UserConfig;
use crate::config::read::read_config_file;
use crate::config::read::read_workspace_file;

struct RunError {
    exit_code: i32,
    message: String,
}
impl RunError {
    fn from(exit_code: i32, message: String) -> RunError {
        RunError { exit_code, message }
    }
}

pub fn main() -> i32 {
    let cli = super::build_cli();
    let matches = cli.get_matches();

    if &matches.subcommand_name() == &Some("completions") {
        run_completions(matches)
    } else {
        match run_gws(matches) {
            Ok(exit_code) => exit_code,
            Err(err) => {
                eprintln!("{}", &err.message);
                err.exit_code
            }
        }
    }
}

fn run_completions(matches: ArgMatches) -> i32 {
    let mut cli = super::build_cli();
    let shell: Shell = matches
        .subcommand
        .unwrap()
        .matches
        .value_of("shell")
        .expect("shell argument required")
        .parse()
        .expect("Failed to parse shell argument");
    let bin_name: String = cli.get_name().to_string();
    cli.gen_completions_to(bin_name, shell, &mut std::io::stdout());
    exit_codes::OK
}

fn find_workspace(current_dir: &Path) -> Option<(&Path, PathBuf)> {
    let ws_file_path = current_dir.join(".projects.gws");
    if ws_file_path.exists() {
        Some((current_dir, ws_file_path))
    } else {
        current_dir.parent().and_then(find_workspace)
    }
}

fn find_config_file(matches: &ArgMatches) -> Option<PathBuf> {
    if matches.is_present("no-config") {
        None
    } else {
        ProjectDirs::from("se.emlun.gws", "", "gws")
            .map(|project_dir| project_dir.config_dir().join("config.toml"))
            .filter(|f| f.exists())
    }
}

fn run_gws(matches: ArgMatches) -> Result<i32, RunError> {
    let config: Option<UserConfig> = match find_config_file(&matches) {
        Some(config_path) => Some(read_config_file(&config_path).map_err(|e| {
            RunError::from(
                exit_codes::USER_ERROR,
                format!("Failed to parse config file: {:?}\n{:?}", config_path, e),
            )
        })?),
        None => None,
    };

    let palette = config
        .and_then(|conf| conf.palette())
        .unwrap_or_else(Palette::default);

    let subcommand: Command = match &matches.subcommand {
        None => super::status::make_cli_command(&matches),
        Some(sc) => match sc.name.as_ref() {
            "clone" => super::clone::make_cli_command(&sc.matches),
            "fetch" => super::fetch::make_cli_command(&sc.matches),
            "ff" => super::ff::make_cli_command(&sc.matches),
            "status" => super::status::make_cli_command(&sc.matches),
            "update" => super::update::make_cli_command(&sc.matches),
            _ => panic!("Unknown subcommand: {}", sc.name),
        },
    };

    let working_dir: &Path = match matches.args.get("dir") {
        Some(chdir_arg) => Path::new(
            chdir_arg.vals[0]
                .to_str()
                .expect("Did not understand <dir> argument"),
        ),
        None => Path::new("."),
    };

    match find_workspace(working_dir) {
        Some((workspace_dir, ws_file_path)) => match read_workspace_file(&ws_file_path) {
            Ok(ws) => {
                let result = match subcommand {
                    Command::DirectoryCommand(cmd) => cmd.run(workspace_dir, &ws, &palette),
                    Command::RepositoryCommand(cmd) => cmd.run(workspace_dir, &ws, &palette),
                };
                result.map_err(|_| {
                    RunError::from(exit_codes::UNKNOWN_ERROR, "Unknown error".to_string())
                })
            }
            Err(_) => Err(RunError::from(
                exit_codes::USER_ERROR,
                format!("Failed to parse projects file: {:?}", ws_file_path),
            )),
        },
        None => Err(RunError::from(
            exit_codes::USER_ERROR,
            format!("Not in a workspace."),
        )),
    }
}
