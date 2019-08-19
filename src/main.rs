use failure::Fail;
use std::cmp::Ordering::Equal;
use std::error::Error;
use structopt::{self, StructOpt};

mod config;
mod tools;

use crate::config::{Config, IndexConfig};
use crate::tools::{reindex, search};

#[derive(StructOpt, Debug)]
#[structopt(name = "searchr")]
struct Opt {
    /// Be verbose (log to stderr). -vv for debug level, -vvv for trace
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u32,
    /// Use a custom config file
    #[structopt(short = "c", long = "config")]
    config_path: Option<String>,
    /// Choose which index to operate on
    #[structopt(short = "i", long = "index")]
    index_name: Option<String>,
    /// Ignore the --index option and work on all registered indices
    #[structopt(short = "a", long = "all")]
    all: bool,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Re-index a registered index
    #[structopt(name = "index")]
    Index {},
    /// Run a search query on an index
    #[structopt(name = "search")]
    Search {
        /// Limit the number of displayed results
        #[structopt(short = "l", long = "limit", default_value = "10")]
        limit: usize,
        /// The search query
        query: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let config = Config::load(opt.config_path)?;

    let level = match opt.verbose {
        0 => simplelog::LevelFilter::Warn,
        1 => simplelog::LevelFilter::Info,
        2 => simplelog::LevelFilter::Debug,
        _ => simplelog::LevelFilter::Trace,
    };
    simplelog::TermLogger::init(
        level,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stderr,
    )
    .unwrap_or_else(|_| eprintln!(":: Could not init termlogger. Ignoring."));

    if config.indexes.is_empty() {
        return Err("no indexes defined in the config file".into());
    }

    let indexes: Vec<IndexConfig> = if opt.all {
        config.indexes.values().map(|v| v.to_owned()).collect()
    } else {
        let chosen_index = opt.index_name.or(config.main.default_index);
        let index = if config.indexes.len() == 1 && chosen_index.is_none() {
            config
                .indexes
                .values()
                .map(|v| v.to_owned())
                .nth(0)
                .expect("len asserted to be 1")
        } else {
            match chosen_index {
                None => {
                    return Err(
                        "more than 1 index defined, but no index chosen and no default".into(),
                    );
                }
                Some(key) => match config.indexes.get(&key) {
                    Some(index) => index.to_owned(),
                    None => {
                        return Err(format!("index name {} not found", key).into());
                    }
                },
            }
        };
        vec![index]
    };

    match opt.cmd {
        Command::Search { limit, query } => {
            let mut results = Vec::new();
            for index_config in indexes {
                results.append(&mut search(index_config, &query, limit).map_err(|e| e.compat())?);
            }
            // merge the results and print the top `limit` entries
            results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Equal));
            for result in results.into_iter().take(limit) {
                println!("{}", result.fname);
            }
        }
        Command::Index {} => {
            for index_config in indexes {
                reindex(index_config).map_err(|e| e.compat())?;
            }
        }
    }

    Ok(())
}
