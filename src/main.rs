use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::{self, StructOpt};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::ReloadPolicy;
use tantivy::{doc, Index, IndexWriter};
use walkdir::WalkDir;

mod tools;

use crate::tools::{get_schema, search, reindex};

#[derive(StructOpt, Debug)]
#[structopt(name = "local-search")]
struct Opt {
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
    Index {
    },
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

fn main() -> tantivy::Result<()> {
    let opt = Opt::from_args();
    dbg!(&opt);

    let schema = get_schema();

    match opt.cmd {
        Command::Search { limit, query } => {
            let index_path = MmapDirectory::open(opt.index_name.unwrap())?;
            let index = Index::open_or_create(index_path, schema.clone())?;
            search(index, query, limit)?;
        }
        Command::Index {  } => {
            let index_path = MmapDirectory::open(opt.index_name.unwrap())?;
            let index = Index::open_or_create(index_path, schema.clone())?;
            reindex(index)?;
        }
    }

    Ok(())
}

// TODO: instate this kind of logic to pick and load a config file
// pub fn choose_config_file(
//     file_override: &Option<String>,
// ) -> Result<Option<String>, Box<dyn Error>> {
//     match file_override {
//         Some(s) => {
//             // file override, use if exists, else err
//             if s == "NONE" {
//                 Ok(None)
//             } else if Path::new(s).exists() {
//                 Ok(Some(s.to_owned()))
//             } else {
//                 Err(format!("config file not found: {:?}", s).into())
//             }
//         }
//         None => {
//             // no file override; find a file in the default locations
//             let config_dir = match env::var("XDG_CONFIG_HOME") {
//                 Ok(val) => val,
//                 Err(_) => format!("{}/.config", env::var("HOME")?),
//             };

//             let config_file = format!("{}/pc/config.toml", config_dir);

//             if Path::new(&config_file).exists() {
//                 Ok(Some(config_file))
//             } else {
//                 Ok(None)
//             }
//         }
//     }
// }

// pub fn read_config(path: &str) -> Result<Config, Box<dyn Error>> {
//     let data = read_file(path)?;
//     let config = toml::from_str(&data)?;
//     Ok(config)
// }
