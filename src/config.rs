use std::fs::File;
use std::io::Result as IoResult;
use std::path::Path;

use serde_yaml;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    update_interval: u64,
    batch_size: usize,
    perforce: PerforceConfig,
    mercurial: MercurialConfig,
    mappings: Vec<MappingConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerforceConfig {
    command: String,
    work_dir: String,
    client: String,
    port: String,
    user: String,
    password: String,
    ignore: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MercurialConfig {
    command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MappingConfig {
    depot_directory: String,
    bookmark: String,
    local_directory: String,
}

impl Config {
    pub fn read<P>(path: P) -> IoResult<Config>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;

        Ok(serde_yaml::from_reader(file).expect("deserialization error"))
    }

    #[inline]
    pub fn update_interval(&self) -> u64 {
        self.update_interval
    }

    #[inline]
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    #[inline]
    pub fn perforce(&self) -> &PerforceConfig {
        &self.perforce
    }

    #[inline]
    pub fn mercurial(&self) -> &MercurialConfig {
        &self.mercurial
    }

    #[inline]
    pub fn mappings(&self) -> &[MappingConfig] {
        self.mappings.as_ref()
    }
}

impl PerforceConfig {
    pub fn command(&self) -> &String {
        &self.command
    }

    pub fn work_dir(&self) -> &String {
        &self.work_dir
    }

    pub fn client(&self) -> &String {
        &self.client
    }

    pub fn port(&self) -> &String {
        &self.port
    }

    pub fn user(&self) -> &String {
        &self.user
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn ignore(&self) -> &String {
        &self.ignore
    }
}

impl MercurialConfig {
    pub fn command(&self) -> &String {
        &self.command
    }
}

impl MappingConfig {
    pub fn depot_directory(&self) -> &String {
        &self.depot_directory
    }

    pub fn bookmark(&self) -> &String {
        &self.bookmark
    }

    pub fn local_directory(&self) -> &String {
        &self.local_directory
    }
}
