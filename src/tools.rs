use std::fs::File;
use std::io::Read;
use std::path::Path;

use glob::{glob_with, MatchOptions};
use log::{debug, info, warn};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::tokenizer::*;
use tantivy::ReloadPolicy;
use tantivy::{doc, Index};

use crate::config::IndexConfig;

struct MyIndex {
    index: Index,
    filename: Field,
    contents: Field,
}

impl MyIndex {
    fn open(path: String) -> tantivy::Result<Self> {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("filename", STRING | STORED);

        // Use a custom stemmer based on en_stem. We can modify this later to switch languages from
        // the config if desired.
        let stemmer = SimpleTokenizer
            .filter(RemoveLongFilter::limit(40))
            .filter(LowerCaser)
            .filter(Stemmer::new(Language::English));
        let stemmer_name = format!("custom_stemmer_{}", "en");

        let text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(&stemmer_name)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        schema_builder.add_text_field("contents", text_options);
        let schema = schema_builder.build();

        let index_path = MmapDirectory::open(path)?;
        let index = Index::open_or_create(index_path, schema.clone())?;
        index.tokenizers().register(&stemmer_name, stemmer);

        Ok(Self {
            index,
            filename: schema.get_field("filename").unwrap(),
            contents: schema.get_field("contents").unwrap(),
        })
    }
}

pub fn reindex(index_config: IndexConfig) -> tantivy::Result<()> {
    let my_index = MyIndex::open(index_config.index_path.clone())?;
    let mut index_writer = my_index.index.writer(50_000_000)?;

    // reset the index
    index_writer.delete_all_documents()?;

    let glob_options = MatchOptions {
        case_sensitive: index_config.case_sensitive.unwrap_or(true),
        require_literal_separator: index_config.require_literal_separator.unwrap_or(false),
        require_literal_leading_dot: index_config.require_literal_leading_dot.unwrap_or(false),
    };

    // now add all the matching files to the index
    for expr in index_config.files {
        info!(":: Adding pattern to index: {}", expr);
        for entry in glob_with(&expr, glob_options).expect("failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let path = Path::new(&path);
                    // skip directories - we only want to read files
                    if path.is_file() {
                        debug!("Adding file: {}", path.display());
                        let mut file = File::open(path)?;
                        let doc = doc!(
                            my_index.filename => path.to_str().unwrap(),
                            my_index.contents => {
                                let mut contents = String::new();
                                file.read_to_string(&mut contents)?;
                                contents
                            },
                        );
                        index_writer.add_document(doc);
                    }
                }
                Err(e) => {
                    warn!("Warning: {:?}", e);
                }
            }
        }
    }

    // and finally commit to disk
    index_writer.commit()?;

    Ok(())
}

pub fn search(
    index_config: IndexConfig,
    query: &str,
    limit: usize,
) -> tantivy::Result<Vec<Result>> {
    let my_index = MyIndex::open(index_config.index_path.clone())?;

    let reader = my_index
        .index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&my_index.index, vec![my_index.contents]);
    let query = query_parser.parse_query(&query)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let fname = retrieved_doc
            .get_first(my_index.filename)
            .unwrap()
            .text()
            .unwrap();
        results.push(Result {
            score,
            fname: fname.to_owned(),
        });
    }
    Ok(results)
}

#[derive(Debug, Clone)]
pub struct Result {
    pub score: f32,
    pub fname: String,
}
