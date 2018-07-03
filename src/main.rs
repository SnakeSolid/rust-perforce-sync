#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate env_logger;
extern crate serde_yaml;
extern crate time;

use std::env;

mod config;
mod mercurial;
mod perforce;
mod worker;

use config::Config;
use worker::Worker;

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() == 0 {
        error!("No configuration given. Use configuration file as single parameter.");

        return;
    } else if args.len() > 1 {
        error!("Too many configurations given. Use configuration file as single parameter.");

        return;
    }

    let config = Config::read(&args[0]).expect("config read failed");
    let worker = Worker::new(&config);

    worker.start();
}
