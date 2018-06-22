#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate env_logger;
extern crate serde_yaml;
extern crate time;

use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

mod config;
mod mercurial;
mod perforce;

use config::Config;
use config::MappingConfig;
use mercurial::MercurialClient;
use perforce::Change;
use perforce::PerforceClient;

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

fn main() {
    env_logger::init();

    let config = Config::read("config.yaml").expect("config read failed");
    let update_interval = Duration::from_secs(config.update_interval());
    let batch_size = config.batch_size();

    loop {
        info!("Processing batch");
        let now = Instant::now();

        for mapping in config.mappings() {
            process_mapping(&config, mapping, batch_size);
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

fn process_mapping(config: &Config, mapping: &MappingConfig, batch_size: usize) {
    let depot_directory = mapping.depot_directory();
    let bookmark = mapping.bookmark();

    let perforce_config = config.perforce();
    let mut p4_client = PerforceClient::new(
        perforce_config.command(),
        perforce_config.work_dir(),
        perforce_config.client(),
        perforce_config.port(),
        perforce_config.user(),
        perforce_config.password(),
        perforce_config.ignore(),
    );

    let mercurial_config = config.mercurial();
    let hg_client = MercurialClient::new(mercurial_config.command(), mapping.local_directory());

    p4_client.login().expect("failed to login");
    hg_client.update(bookmark).expect("update failed");

    let commit = match hg_client
        .last_commit(bookmark)
        .expect("failed to get last commit")
    {
        Some(commit) => commit + 1,
        None => 1,
    };

    let changes = p4_client
        .changes(depot_directory, commit)
        .expect("failed to read changes");

    if changes.is_empty() {
        info!("No more changes");

        return;
    }

    let mut have_changes = false;

    for id in changes.into_iter().take(batch_size) {
        info!("Processing change {}", id);

        let change = p4_client.change(id).expect("failed to read change");
        let message = format_change(&change);

        p4_client.sync(depot_directory, id).expect("failed to sync");
        p4_client.clean(depot_directory).expect("failed to clean");

        let large_files = hg_client
            .get_large_files(10 * 1024 * 1024)
            .expect("add large files failed");
        let mut has_changes = !large_files.is_empty();

        for large_file in large_files {
            hg_client.add_large(&large_file).expect("add large failed");
        }

        if !has_changes {
            let changed_files = hg_client.status().expect("status failed");

            has_changes = !changed_files.is_empty();
        }

        if has_changes {
            hg_client.addremove(80).expect("addremove failed");
            hg_client
                .commit(&message, change.date(), change.user())
                .expect("commit failed");

            have_changes = true;
        }
    }

    p4_client.logout().expect("failed to logout");

    if have_changes {
        hg_client.push().expect("push failed");
    }
}
