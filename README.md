# gws2

[![Build Status](https://github.com/emlun/gws2/workflows/build/badge.svg)](https://github.com/emlun/gws2/actions?query=workflow:build)
[![Clippy Status](https://github.com/emlun/gws2/workflows/clippy/badge.svg)](https://github.com/emlun/gws2/actions?query=workflow:clippy)

Rewrite of [gws][gws] in Rust

`gws` is a colorful helper to manage workspaces composed of Git repositories.

## SHOW ME PICTURES

Here are some screen captures of the original [`gws`][gws]:

![gws](http://streakycobra.github.io/gws/images/001.png)

![gws](http://streakycobra.github.io/gws/images/002.png)

![gws](http://streakycobra.github.io/gws/images/003.png)

![gws](http://streakycobra.github.io/gws/images/004.png)

![gws](http://streakycobra.github.io/gws/images/005.png)

## Differences from original gws

gws2 hasn't quite yet reached feature parity with [gws][gws].

- [ ] `check` command [#2][iss-check]
- [x] `clone` command
- [x] `fetch` command
- [x] `ff` command
- [ ] `init` command [#1][iss-init]
- [x] `status` command
- [x] `update` command
- [x] Customizable color scheme [#4][iss-theme]
- [ ] `.ignore.gws` file [#3][iss-ignore]
- [x] Run from subdirectory [#5][iss-subdir]

But on the other hand, gws2 already has some new features:

- Branches' sync status is compared to the branch's configured upstream branch,
  not implicitly against `origin/<name>`.
- The option `-C` allows to change the working directory, like the `-C` option
  of `git`
- `--help` is expanded and structured per subcommand.
- Being a compiled binary, performance is greatly improved.

## Installation

### Arch Linux

Install the [gws2-git](https://aur.archlinux.org/packages/gws2-git) package from
the AUR.

### Other

Build and install the `gws` binary using `cargo`:

```shell
git clone --recurse-submodules https://github.com/emlun/gws2.git
cd gws2
cargo install --path .
gws --version
```

Optionally, also generate a completions file for your shell. For example:

```shell
gws completions bash > ~/.config/bash/completions/gws.bash_completion
```

See `gws completions --help` for a list of supported shells.

### Quick Start

Pick one of above options to create a `.projects.gws` file. Then:

<details>
  <summary>
    Manual
  </summary>
  Create a file named `.projects.gws` in a desired workspace folder (e.g.
  `~/dev`) and fill it with project definitions (see [Syntaxes](#projectsgws)
  below):

  ```txt
  # Work related
  work/tools/q | https://github.com/harelba/q.git

  # Other
  contrib/gws  | https://github.com/StreakyCobra/gws.git
  contrib/peru | https://github.com/buildinspace/peru
  ```

</details>

<details>
  <summary>
    Automated
  </summary>
  > [!Warning]
  > (NOT YET IMPLEMENTED: [#1][iss-init]) Let `gws` detect existing repositories and create the `.projects.gws` for you:

  ```shell
  cd path/to/your/workspace
  gws init
  ```
  
</details>

- Clone all missing repositories with `gws update`, or some specific ones with
  `gws clone`.

- Do some hacking.

- Show the status of the workspace with `gws`. It reveals which repositories
  are clean, which ones have uncommited changes, and even which ones are not
  up-to-date with `origin`.

### But better

Let's say you made a `~/dev/` workspace folder and you created your
`.projects.gws` list in it. Then your workspace became really easy to replicate!
Just make this `~/dev` folder a Git repository, add two files and commit them:
`.projects.gws` and the following `.gitignore`:

```gitignore
# Ignore everything, so all repositories in our case
*

# But not these files
!.projects.gws
!.gitignore
```

Now, when you need to get your workspace on another computer, just clone
the `dev` repository, for instance again to the `~/dev` folder. Go into it and
do a `gws update`. Everything is cloned and ready to be hacked!

You want to add a new project into your workspace? Add it to the
`.projects.gws` list, do a `gws update` to get it. Then commit and push the
`.projects.gws` file, so when you arrive at work for instance, you just need to
`git pull` on the `~/dev` folder and then `gws update` to get the same
workspace structure that you had at home.

## Why?

Written by [Fabien Dubosson](https://github.com/StreakyCobra), the original
[gws][gws] author:

> [!Note]
> If you are, like me, a Linux programmer/hacker/coder who uses Git a lot, you
> certainly have a directory in your home folder named `dev`, `workspace`, `code`
> or something else that contains all the projects you are working on. For
> instance my current organisation is:
>
> ```shell
> $ tree 
> dev
> ├── archlinux
> │   ├── aur
> │   └── habs
> ├── perso
> │   ├── gws
> │   ├── imaxplore
> │   └── teafree
> └── config
> ```
>
> where `aur`, `habs`, `gws`, `imaxplore`, `teafree`, `config` are Git
> repositories.
>
> Since I use at least three different computers - one laptop, one at home and
> one at work - I like to have the same folder structure on all of them. Of
> course remembering which project was added recently on other computers and in
> which folder is tedious.
>
> So I started to think about using Git submodules to register all projects on
> which I am working and syncing them with Git between the two computers. But
> clearly Git submodules are not usable because they are work with specific
> commits and not by following branches.
>
> No worry. The problem is pretty trivial, so I decided to start write a little
> bash (YOLO) script that reads a simple list of repositories, and clones them if
> they don't exist. And then, commit by commit, the script as grown to finally
> become a helper to sync, monitor and check workspaces.
>
> I thought it can be useful to other people, so I made a little cleanup, wrote
> some small documentation, and there it is. I hope you will enjoy it!

## Features

This tool offers some features, including:

- It uses a list of projects, named `.projects.gws`, containing many projects
  described by their names, their repository URLs, and optionaly an upstream
  URL (mapped as a Git remote named `upstream`), like:

```txt
work/theSoftware | git@github.com:You/theSoftware.git
perso/gws        | git@github.com:You/gws.git         | git@github.com:StreakyCobra/gws.git
```

- (NOT YET IMPLEMENTED: [#3][iss-ignore]) It can use an ignore list, named `.ignore.gws`, containing regular
  expressions which discard some specific projects, for instance to disable on
  your home computer the work-related projects.

  ```txt
  ^work/
  ```

- (NOT YET IMPLEMENTED: [#1][iss-init]) It can detect already existing repositories and create the projects list
  from that.

  ```shell
  gws init
  ```

- It can clone missing repositories from the projects list (but not
  delete ones removed from the list, you have to do that manually for safety.
  Note that there is the `check` command to identify unlisted repositories).

  ```shell
  gws update
  ```

  `update` accepts the `--only-changes` option. If present, repos that have at
  least one remote and are unaffected by the update will not be printed.

- It can also clone a specified selection of missing repositories from the
  projects list, if you don't need all of them right now.

  ```shell
  gws clone work/theSoftware
  ```

- It can monitor all listed repositories in one command, showing uncommitted
  changes, untracked changes and branches not synced with origin.

  ```shell
  gws status
  ```

  or simply

  ```shell
  gws
  ```

  `gws status` and `gws` accept the `--only-changes` option. If present, missing
  repos as well as repos that have at least one remote and only clean branches
  will not be shown.

- It can fetch the modifications from upstream for all repositories. It is
  useful to make sure you have the latest modifications, for instance before
  getting on a train with no internet connection:

  ```shell
  gws fetch
  ```

  `fetch` accepts the `--only-changes` option, which has the same effect as for
  `status`.

- It can also (for the same reasons) pull the modifications from upstream for
  all repositories (but fast-forward only). Same as `gws fetch`, but also does
  fast-forward merges.

  ```shell
  gws ff    # Mnemonic: ff=fast-forward
  ```

  `ff` accepts the `--only-changes` option, which has the same effect as for
  `status`.

- (NOT YET IMPLEMENTED: [#2][iss-check]) It can check the workspace for all repositories (known, unknown, ignored,
  missing). Note: This command can be quite slow in large repositories (e.g.
  home folder), because it needs to search the entire space for unknown
  repositories. Mainly used from time to time to check workspace consistency:

  ```shell
  gws check
  ```

## Syntaxes

### .projects.gws

One project per line. Must be of the form

```txt
<any/folder/path> | <remote_url1> <remote_name1> [ | <remote_url2> <remote_name2> [ |  ... ]]
```

where

- the `<remote_name1>` can be skipped and `origin` will be used instead.
- the `<remote_name2>` can be skipped and `upstream` will be used instead.
- there must be at least one `<remote_name>` mapping to `origin`.
- there can also be blank lines, comments or inline comments. Comments start
  with `#` and continue to the end of the line.
- the *folder path* can be any valid linux folder path not containing `|`, `#`
  or spaces.
- the *remote names* can be any string not containing `|`, `#` or spaces.
- the *remote URLs* are passed to Git as-is, so they can be anything accepted
  by Git but must not contain `|`, `#` or spaces. For instance if you have SSH
  aliases in your config they are accepted.

### .ignore.gws

(NOT YET IMPLEMENTED: [#3][iss-ignore])

One regular expression per line. The regular expression will be matched against
each project's *folder path*. Some examples:

- Ignore the folder `work` and all its subfolders:

  ```txt
  ^work/
  ```

- Ignore all repositories ending with `-work`:

  ```txt
  -work$
  ```

- Ignore all repos containing an `a` inside:

  ```txt
  a
  ```

This function is really useful for locally ignoring some projects that are not
needed or not accessible.

## Config file

A config file can be placed at `${USER_CONFIG}/gws/config.toml`, for example
`~/.config/gws/config.toml` in Linux. See the [directories-rs docs][config-dir]
for examples for other operating systems.

See `config.example.toml` for exhaustive documentation of the available
settings.

## Other thoughts

- Except for cloning repositories, this program does not have as a goal to
  interact with your repositories. So no `pull all`, `push all`, `delete all
  unused`, features will be implemented (except fast-forward). This will imply
  too much checking to prevent data loss. Instead, just look at the status of
  the repositories and perform any needed actions manually on regular basis.

- You can use the commands from any subfolder of the workspace (as `git` does
  for instance).

- The file `.projects.gws` can easily be verisoned to sync the list of
  projects between different computers.

- The file `.ignore.gws` allows for keeping the same `.projects.gws` list on
  all computers, but to locally disable some projects (for instance
  work-related projects at home because they are unneeded or even not
  accessible from there).

- (NOT YET IMPLEMENTED: [#2][iss-check]) `gws check` can be quite slow (for instance if the
  workspace is the home folder) because it searches all existing Git projects
  recursively.

## Running the tests

Some of the integration tests attempt to clone a repository from GitHub via SSH.
If you already have an SSH key set up for GitHub in your keyring, you're all
good - but if you don't, there's an SSH key bundled in `tests/id_rsa` and
registered as a deploy key for the public git repository at
<https://github.com/emlun/gws2> . The tests will automatically try to add it to
your `ssh-agent` for the tests that need it, and remove it from the agent again
after the test.

However, if it looks like your SSH agent is `gpg-agent`, then the tests will not
add the key and will fail instead. This is because `gpg-agent` asks you to set a
passphrase to protect the imported test key, and also makes it more cumbersome
to clean up the key after the tests. Therefore, to run the SSH tests, please run
the standard `ssh-agent` instead. You can do this by, for example, spawning a
subshell and running `eval $(ssh-agent)` in it:

```shell
bash
eval $(ssh-agent)
cargo test
cargo test
killall ssh-agent
exit
```

[config-dir]: https://docs.rs/directories/latest/directories/struct.ProjectDirs.html#method.config_dir
[gws]: https://github.com/StreakyCobra/gws
[iss-check]: https://github.com/emlun/gws2/issues/2
[iss-ignore]: https://github.com/emlun/gws2/issues/3
[iss-init]: https://github.com/emlun/gws2/issues/1
[iss-subdir]: https://github.com/emlun/gws2/issues/5
[iss-theme]: https://github.com/emlun/gws2/issues/4