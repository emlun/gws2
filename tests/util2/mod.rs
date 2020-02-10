/// Workaround to silence false-positive warnings about dead code for tests that
/// don't use these functions
use std::fs::create_dir_all;

use std::fs::File;
use std::path::Path;

use gws::config::data::Workspace;
use gws::config::read::read_workspace_file;

use super::util::write_projects_file;
use super::util::Error;

pub fn in_workspace_with_projects_file<T, F>(projects_contents: &str, test: F) -> Result<T, Error>
where
    F: Fn(&Path, Workspace) -> Result<T, Error>,
{
    let tmpdir = tempfile::tempdir()?;
    let workspace_dir = tmpdir.path().join("workspace");

    create_dir_all(&workspace_dir)?;
    write_projects_file(
        workspace_dir.join(".projects.gws").as_path(),
        projects_contents,
    )?;

    let workspace = read_workspace_file(workspace_dir.join(".projects.gws")).unwrap();
    Ok(test(&workspace_dir, workspace)?)
}

pub fn with_bundled_ssh_key_in_agent<T, F>(
    test: F,
) -> Result<Box<dyn Fn(&Path, Workspace) -> Result<T, Error>>, Error>
where
    F: Fn(&Path, Workspace) -> Result<T, Error>,
    F: 'static,
{
    if ssh_agent_looks_like_gpg() {
        Err(Error::from(
            "It looks like your ssh-agent is the gpg-agent. This agent asks you to set a passphrase for the test key, which isn't very helpful. To run the SSH tests, please use the standard ssh-agent instead. You can do this by starting a subshell and running 'eval $(ssh-agent)' in it.",
        ))
    } else {
        let keyfile_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/id_rsa");

        Ok(Box::new(move |path: &Path, workspace: Workspace| {
            let mut ssh_add_proc = std::process::Command::new("ssh-add")
                .arg("-") // Write keyfile via stdin to avoid file permission issues
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            std::io::copy(
                &mut File::open(&keyfile_path)?,
                &mut ssh_add_proc.stdin.as_mut().unwrap(),
            )?;
            assert!(
                ssh_add_proc.wait()?.success(),
                "Failed to add test key to ssh-agent. Is it running and is SSH_AUTH_SOCK set?"
            );

            let result = test(path, workspace);

            std::process::Command::new("ssh-add")
                .arg("-d")
                .arg(keyfile_path.to_str().unwrap())
                .status()?;
            result
        }))
    }
}

fn ssh_agent_looks_like_gpg() -> bool {
    env!("SSH_AUTH_SOCK").contains("gpg-agent")
}
