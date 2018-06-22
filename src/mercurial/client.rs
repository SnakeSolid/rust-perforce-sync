use time::strftime;
use time::Tm;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use super::MercurialError;
use super::MercurialResult;

#[derive(Debug)]
pub struct MercurialClient {
    command: String,
    work_dir: String,
}

impl MercurialClient {
    pub fn new(command: &str, work_dir: &str) -> MercurialClient {
        MercurialClient {
            command: command.into(),
            work_dir: work_dir.into(),
        }
    }

    pub fn update(&self, revision: &str) -> MercurialResult<()> {
        info!("Mercurial update, revision = {}.", revision);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("update")
            .arg("--rev")
            .arg(revision)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("update success.");
            Ok(())
        } else {
            warn!("update failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn last_commit(&self, revision: &str) -> MercurialResult<Option<u32>> {
        info!("Mercurial last commit, revision = {}.", revision);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("log")
            .arg("--rev")
            .arg(revision)
            .arg("--template")
            .arg("{desc|firstline}")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let mut result = None;

        if let Some(ref mut stdout) = child.stdout {
            debug!("Waiting for log output.");
            let mut buffer = String::new();

            stdout
                .read_to_string(&mut buffer)
                .map_err(MercurialError::communication_error)?;

            if buffer.starts_with("change #") {
                result = Some(buffer[8..]
                    .parse()
                    .map_err(MercurialError::change_parse_error)?);
            }
        }

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("last commit success.");
            Ok(result)
        } else {
            warn!("last commit failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn addremove(&self, similarity: u8) -> MercurialResult<()> {
        info!("Mercurial addremove, similarity = {}.", similarity);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("addremove")
            .arg("--similarity")
            .arg(format!("{}", similarity))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            Ok(())
        } else {
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn add_large(&self, path: &str) -> MercurialResult<()> {
        info!("Mercurial add large, path = {}.", path);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("add")
            .arg("--large")
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("Add large success.");
            Ok(())
        } else {
            warn!("Add large failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn get_large_files(&self, min_size: u64) -> MercurialResult<Vec<String>> {
        info!("Mercurial add large files, min_size = {}.", min_size);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("status")
            .arg("--no-status")
            .arg("--unknown")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let mut all_files = Vec::new();

        debug!("Reading changed files.");
        if let Some(ref mut stdout) = child.stdout {
            debug!("Reading all files.");
            let mut reader = BufReader::new(stdout);

            for line in reader.lines() {
                debug!("Reading file {:?}.", line);
                let line = line.map_err(MercurialError::communication_error)?;

                debug!("Adding file {}.", line);
                all_files.push(line);
            }
        }

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            let mut large_files = Vec::new();

            for path in all_files {
                let mut file_path = PathBuf::new();
                file_path.push(&self.work_dir);
                file_path.push(&path);

                let metadata = file_path
                    .metadata()
                    .map_err(MercurialError::read_metadata_error)?;
                let file_size = metadata.len();

                if file_size >= min_size {
                    debug!("Found large file, size = {}.", file_size);
                    large_files.push(path);
                }
            }

            debug!("Add large files success.");
            Ok(large_files)
        } else {
            warn!("Add large files failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn status(&self) -> MercurialResult<Vec<String>> {
        info!("Mercurial status.");
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("status")
            .arg("--no-status")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let mut result = Vec::new();

        debug!("Reading changed files.");
        if let Some(ref mut stdout) = child.stdout {
            debug!("Reading all files.");
            let mut reader = BufReader::new(stdout);

            for line in reader.lines() {
                debug!("Reading file {:?}.", line);
                let line = line.map_err(MercurialError::communication_error)?;

                debug!("Adding file {}.", line);
                result.push(line);
            }
        }

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("Status success.");
            Ok(result)
        } else {
            warn!("Status failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn commit(&self, message: &str, date: &Tm, user: &str) -> MercurialResult<()> {
        info!("Mercurial commit, user = {}.", user);
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("commit")
            .arg("--message")
            .arg(message)
            .arg("--date")
            .arg(strftime("%Y-%m-%d %H:%M:%S", date).map_err(MercurialError::date_format_error)?)
            .arg("--user")
            .arg(user)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("Commit success.");
            Ok(())
        } else {
            warn!("Commit failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }

    pub fn push(&self) -> MercurialResult<()> {
        info!("Mercurial push.");
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .arg("push")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(MercurialError::execution_error)?;

        let status = child.wait().map_err(MercurialError::io_error)?;

        if status.success() {
            debug!("Push success.");
            Ok(())
        } else {
            warn!("Push failed.");
            Err(MercurialError::exit_error(status.code()))
        }
    }
}
