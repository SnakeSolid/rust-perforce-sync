use time::strptime;
use time::Tm;

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Result as IoResult;
use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use super::PerforceError;
use super::PerforceResult;

#[derive(Debug)]
pub struct PerforceClient {
    command: String,
    work_dir: String,
    client: String,
    port: String,
    user: String,
    password: String,
    ignore: String,
    token: Option<String>,
}

impl PerforceClient {
    pub fn new(
        command: &str,
        work_dir: &str,
        client: &str,
        port: &str,
        user: &str,
        password: &str,
        ignore: &str,
    ) -> PerforceClient {
        PerforceClient {
            command: command.into(),
            work_dir: work_dir.into(),
            client: client.into(),
            port: port.into(),
            user: user.into(),
            password: password.into(),
            ignore: ignore.into(),
            token: None,
        }
    }

    pub fn login(&mut self) -> PerforceResult<()> {
        info!("Perforce login.");
        let mut child = Command::new(&self.command)
            .current_dir(&self.work_dir)
            .env_clear()
            .env("P4PORT", &self.port)
            .env("P4USER", &self.user)
            .arg("login")
            .arg("-p")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(PerforceError::execution_error)?;

        if let Some(ref mut stdout) = child.stdout {
            debug!("Waiting for password prompt.");
            expect_string(stdout, "Enter password: ").map_err(PerforceError::communication_error)?;
        }

        if let Some(ref mut stdin) = child.stdin {
            debug!("Sending password.");
            send_string(stdin, &self.password).map_err(PerforceError::communication_error)?;
        }

        if let Some(ref mut stdout) = child.stdout {
            debug!("Reading auth token.");
            let mut buffer = String::new();

            stdout
                .read_to_string(&mut buffer)
                .map_err(PerforceError::communication_error)?;

            let token = buffer.trim();

            if token.len() == 32 {
                self.token = Some(token.into());

                debug!("Login success.");
            } else {
                return Err(PerforceError::LoginFailed);
            }
        }

        let result = child.wait().map_err(PerforceError::io_error)?;

        if result.success() {
            Ok(())
        } else {
            Err(PerforceError::exit_error(result.code()))
        }
    }

    pub fn logout(&mut self) -> PerforceResult<()> {
        info!("Perforce logout.");
        if let Some(ref token) = self.token {
            let mut child = Command::new(&self.command)
                .current_dir(&self.work_dir)
                .env_clear()
                .env("P4PORT", &self.port)
                .env("P4PASSWD", token)
                .env("P4USER", &self.user)
                .arg("logout")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(PerforceError::execution_error)?;

            debug!("Execution logout.");
            let status = child.wait().map_err(PerforceError::io_error)?;

            if status.success() {
                debug!("Logout success.");
                Ok(())
            } else {
                warn!("Logout failed.");
                Err(PerforceError::exit_error(status.code()))
            }
        } else {
            Ok(())
        }
    }

    pub fn sync(&mut self, directory: &str, commit: u32) -> PerforceResult<()> {
        info!("Perforce sync.");
        if let Some(ref token) = self.token {
            let mut child = Command::new(&self.command)
                .current_dir(&self.work_dir)
                .env_clear()
                .env("P4CLIENT", &self.client)
                .env("P4PORT", &self.port)
                .env("P4PASSWD", token)
                .env("P4USER", &self.user)
                .arg("sync")
                .arg("-q")
                .arg(format!("{}...@{}", directory, commit))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(PerforceError::execution_error)?;

            debug!("Execution sync {} with @{}.", directory, commit);
            let status = child.wait().map_err(PerforceError::io_error)?;

            if status.success() {
                debug!("Sync complete.");
                Ok(())
            } else {
                warn!("Sync failed.");
                Err(PerforceError::exit_error(status.code()))
            }
        } else {
            Err(PerforceError::NotLoggedIn)
        }
    }

    pub fn clean(&mut self, directory: &str) -> PerforceResult<()> {
        info!("Perforce clean.");
        if let Some(ref token) = self.token {
            let mut child = Command::new(&self.command)
                .current_dir(&self.work_dir)
                .env_clear()
                .env("P4CLIENT", &self.client)
                .env("P4PORT", &self.port)
                .env("P4PASSWD", token)
                .env("P4USER", &self.user)
                .env("P4IGNORE", &self.ignore)
                .arg("clean")
                .arg(format!("{}...", directory))
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(PerforceError::execution_error)?;

            debug!("Execution clean of {}.", directory);
            let status = child.wait().map_err(PerforceError::io_error)?;

            if status.success() {
                debug!("Clean complete.");
                Ok(())
            } else {
                warn!("Clean failed.");
                Err(PerforceError::exit_error(status.code()))
            }
        } else {
            Err(PerforceError::NotLoggedIn)
        }
    }

    pub fn changes(&mut self, directory: &str, commit: u32) -> PerforceResult<Vec<u32>> {
        info!("Perforce changes.");
        if let Some(ref token) = self.token {
            let mut child = Command::new(&self.command)
                .current_dir(&self.work_dir)
                .env_clear()
                .env("P4CLIENT", &self.client)
                .env("P4PORT", &self.port)
                .env("P4PASSWD", token)
                .env("P4USER", &self.user)
                .arg("-F")
                .arg("%change%")
                .arg("changes")
                .arg("-e")
                .arg(format!("{}", commit))
                .arg(format!("{}...", directory))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(PerforceError::execution_error)?;

            let mut result = Vec::new();

            debug!("Reading changes of {}.", directory);
            if let Some(ref mut stdout) = child.stdout {
                debug!("Reading all changes.");
                let mut reader = BufReader::new(stdout);

                for line in reader.lines() {
                    debug!("Reading change {:?}.", line);
                    let line = line.map_err(PerforceError::communication_error)?;

                    if line.starts_with("Change ") {
                        debug!("Parsing change {}.", line);
                        let change = line[7..].parse()?;

                        debug!("Adding change {}.", change);
                        result.push(change);
                    }
                }
            }

            result.sort_unstable();

            let status = child.wait().map_err(PerforceError::io_error)?;

            if status.success() {
                debug!("Changes complete.");
                Ok(result)
            } else {
                warn!("Changes failed.");
                Err(PerforceError::exit_error(status.code()))
            }
        } else {
            Err(PerforceError::NotLoggedIn)
        }
    }

    pub fn change(&mut self, commit: u32) -> PerforceResult<Change> {
        info!("Perforce change.");
        if let Some(ref token) = self.token {
            let mut child = Command::new(&self.command)
                .current_dir(&self.work_dir)
                .env_clear()
                .env("P4CLIENT", &self.client)
                .env("P4PORT", &self.port)
                .env("P4PASSWD", token)
                .env("P4USER", &self.user)
                .arg("change")
                .arg("-o")
                .arg(format!("{}", commit))
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .map_err(PerforceError::execution_error)?;

            let mut change = None;
            let mut date = None;
            let mut user: Option<String> = None;
            let mut is_description = false;
            let mut description = String::new();

            debug!("Reading change of {}.", commit);
            if let Some(ref mut stdout) = child.stdout {
                debug!("Reading change content.");
                let mut buffer = Vec::with_capacity(1024);

                stdout
                    .read_to_end(&mut buffer)
                    .map_err(PerforceError::communication_error)?;

                let buffer = String::from_utf8_lossy(&buffer);

                for line in buffer.lines() {
                    debug!("Reading change {:?}.", line);
                    if line.starts_with("Change:") {
                        change = Some(line[8..].parse()?);
                    } else if line.starts_with("Date:") {
                        date = Some(strptime(&line[6..], "%Y/%m/%d %H:%M:%S")
                            .map_err(PerforceError::date_parse_error)?);
                    } else if line.starts_with("User:") {
                        user = Some(line[6..].into());
                    } else if line.starts_with("Description:") {
                        is_description = true;
                    } else if line.starts_with("\t") && is_description {
                        description.push_str(&line[1..]);
                        description.push_str("\n");
                    }
                }
            }

            let status = child.wait().map_err(PerforceError::io_error)?;

            if status.success() {
                if let (Some(change), Some(date), Some(user)) = (change, date, user) {
                    debug!("Reading change complete.");
                    Ok(Change::new(change, date, &user, &description))
                } else {
                    warn!("Reading change failed.");
                    Err(PerforceError::incorrect_change(commit))
                }
            } else {
                warn!("Changes failed.");
                Err(PerforceError::exit_error(status.code()))
            }
        } else {
            Err(PerforceError::NotLoggedIn)
        }
    }
}

#[derive(Debug)]
pub struct Change {
    change: u32,
    date: Tm,
    user: String,
    description: String,
}

impl Change {
    fn new(change: u32, date: Tm, user: &str, description: &str) -> Change {
        Change {
            change,
            date,
            user: user.into(),
            description: description.into(),
        }
    }

    pub fn change(&self) -> u32 {
        self.change
    }

    pub fn date(&self) -> &Tm {
        &self.date
    }

    pub fn user(&self) -> &str {
        &self.user
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

fn expect_string(read: &mut Read, s: &str) -> IoResult<bool> {
    let mut buffer: Vec<_> = (0..s.len()).map(|_| 0).collect();

    read.read_exact(&mut buffer)?;

    Ok(buffer == s.as_bytes())
}

fn send_string(write: &mut Write, s: &str) -> IoResult<()> {
    write.write(s.as_bytes())?;
    write.write(b"\n")?;

    Ok(())
}
