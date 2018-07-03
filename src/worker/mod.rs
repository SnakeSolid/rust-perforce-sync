mod error;

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use config::Config;
use config::MappingConfig;
use mercurial::MercurialClient;
use perforce::Change;
use perforce::PerforceClient;

pub use self::error::WorkerError;
pub use self::error::WorkerResult;

pub struct Worker<'a> {
    config: &'a Config,
}

impl<'a> Worker<'a> {
    pub fn new(config: &'a Config) -> Worker<'a> {
        Worker { config }
    }

    pub fn start(&self) {
        let update_interval = Duration::from_secs(self.config.update_interval());
        let batch_size = self.config.batch_size();

        loop {
            info!("Processing batch, batch_size = {}", batch_size);
            let now = Instant::now();

            for mapping in self.config.mappings() {
                if let Err(err) = self.process_mapping(mapping, batch_size) {
                    error!("{}", err);
                }
            }

            let elapsed = now.elapsed();

            info!("Batch time = {}", elapsed.as_secs());

            if elapsed < update_interval {
                let delta = update_interval - elapsed;

                info!("Sleeping for {}", delta.as_secs());
                sleep(delta);
            }
        }
    }

    fn process_mapping(&self, mapping: &MappingConfig, batch_size: usize) -> WorkerResult<()> {
        info!(
            "Processing mapping, depot_directory = {}",
            mapping.depot_directory()
        );
        let depot_directory = mapping.depot_directory();
        let bookmark = mapping.bookmark();

        let perforce_config = self.config.perforce();
        let mut p4_client = PerforceClient::new(
            perforce_config.command(),
            perforce_config.work_dir(),
            perforce_config.client(),
            perforce_config.port(),
            perforce_config.user(),
            perforce_config.password(),
            perforce_config.ignore(),
        );

        let mercurial_config = self.config.mercurial();
        let hg_client = MercurialClient::new(mercurial_config.command(), mapping.local_directory());

        p4_client.login().map_err(WorkerError::perforce_error)?;
        hg_client
            .update(bookmark)
            .map_err(WorkerError::mercurial_error)?;

        let commit = match hg_client
            .last_commit(bookmark)
            .map_err(WorkerError::mercurial_error)?
        {
            Some(commit) => commit + 1,
            None => 1,
        };

        let changes = p4_client
            .changes(depot_directory, commit)
            .map_err(WorkerError::perforce_error)?;

        if changes.is_empty() {
            info!("No more changes");

            return Ok(());
        }

        let mut have_changes = false;

        for id in changes.into_iter().take(batch_size) {
            info!("Processing change {}", id);

            let change = p4_client.change(id).map_err(WorkerError::perforce_error)?;
            let message = format_change(&change);

            p4_client
                .sync(depot_directory, id)
                .map_err(WorkerError::perforce_error)?;
            p4_client
                .clean(depot_directory)
                .map_err(WorkerError::perforce_error)?;

            let large_files = hg_client
                .get_large_files(10 * 1024 * 1024)
                .map_err(WorkerError::mercurial_error)?;
            let mut has_changes = !large_files.is_empty();

            for large_file in large_files {
                hg_client
                    .add_large(&large_file)
                    .map_err(WorkerError::mercurial_error)?;
            }

            if !has_changes {
                let changed_files = hg_client.status().map_err(WorkerError::mercurial_error)?;

                has_changes = !changed_files.is_empty();
            }

            if has_changes {
                hg_client
                    .addremove(80)
                    .map_err(WorkerError::mercurial_error)?;
                hg_client
                    .commit(&message, change.date(), change.user())
                    .map_err(WorkerError::mercurial_error)?;

                have_changes = true;
            }
        }

        p4_client.logout().map_err(WorkerError::perforce_error)?;

        if have_changes {
            hg_client.push().map_err(WorkerError::mercurial_error)?;
        }

        Ok(())
    }
}

fn format_change(change: &Change) -> String {
    let mut message = format!("change #{}\n", change.change());

    for ch in change.description().chars() {
        if ch.is_ascii() {
            message.push(ch);
        } else {
            let escaped = format!("{}", ch.escape_unicode());

            message.push_str(&escaped);
        }
    }

    message
}
