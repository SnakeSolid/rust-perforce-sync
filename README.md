# Perforce Sync

Perforce depot to Mercurial repository synchronization tool.

The tool performs one-way synchronization from several Perforce depot paths to single Mercurial repository. Every paths
in Mercurial represented by one bookmark.

During single path synchronization tool executes following command sequence:

* login to Perforce - `p4 login -p`;
* update Mercurial repository to corresponding bookmark - `hg update --rev BOOKMARK`;
* get last commit message from Mercurial repository - `hg log --rev BOOKMARK --template {desc|firstline}`;
* list all Perforce changes from last commit - `p4 -F '%change%' changes -e CHANGE PATH...`;
* for every Perforce change do in loop:
  * read full change description - `p4 change -o CHANGE`;
  * synchronize Perforce workspace with change - `p4 sync -q PATH...@CHANGE`;
  * clean Perforce workspace - `p4 clean PATH...`;
  * add all large files (with size > 10Mib) as large files to Mercurial - `hg add --large FILE`;
  * add/remove all changed files to Mercurial repository - `hg addremove --similarity 80`;
  * commit changes to Mercurial repository - `hg commit -m MESSAGE`.
* logout from Perforce - `p4 logout`.
* push new commit to Mercurial server - `hg push`.

## Usage

To start run:

```bash
./perforce-sync [CONFIG]
```

Where CONFIG - path to configuration file.

Tool use `hg push` command to keep repository on a server in consistent state with Perforce. It requires to setup `hgrc`
file to push without login prompt. It can be done using following settings as template:

```ini
[auth]
default.prefix = http://server/hg
default.username = user
default.password = password
default.schemes = http

[paths]
default = http://server/hg/repository/
default:pushurl = http://user@server/hg/repository/
```

## Configuration File

Configuration file should be in YAML format. Configuration file fields:

* `update_interval` - interval between updates;
* `batch_size` - number of changes per single synchronization round;
* `perforce` - Perforce connection settings:
  * `command` - Perforce command line executable;
  * `work_dir` - Perforce working directory;
  * `client` - Perforce workspace name;
  * `port` - Perforce connection port in format: "tcp:server:port";
  * `user` - Perforce user name to login with;
  * `password` - Perforce password;
  * `ignore` - Perforce ignore file, must contain at least all Mercurial directories;
* `mercurial` - mercurial settings:
  * `command` - path to Mercurial executable command.
* `mappings` - list of Perforce path to Mercurial bookmark mappings:
  * `depot_directory` - Perforce depot path (starting with //);
  * `bookmark` - Mercurial bookmark name;
  * `local_directory` - Mercurial repository directory.

## Ignore File

Minimal `p4ignore` file to keep all Mercurial files:

```text
.hg
.hg/**
.hglf/**
.hgignore
```
