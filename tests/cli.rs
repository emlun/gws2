mod util;

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use std::process::Command;
use util::in_example_workspace;

#[test]
fn status_finds_workspace_in_ancestor_dir() -> Result<(), util::Error> {
    in_example_workspace(|workspace_dir, _| {
        let working_dir = workspace_dir.join("new_commit").join("remote");
        let mut cmd = Command::cargo_bin("gws")?;
        cmd.arg("-C").arg(working_dir.to_str().unwrap());
        cmd.arg("--no-config");
        cmd.assert().success();
        Ok(())
    })
}

#[test]
fn status_does_not_find_workspace_in_unrelated_dir() -> Result<(), util::Error> {
    let working_dir = tempfile::tempdir()?;
    let mut cmd = Command::cargo_bin("gws")?;
    cmd.arg("-C").arg(working_dir.path().to_str().unwrap());
    cmd.arg("--no-config");
    cmd.assert().failure();
    Ok(())
}
