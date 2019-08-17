use glob::glob;
use std::fs::File;
use std::io::Read;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::ReloadPolicy;
use tantivy::{doc, Index};

use crate::config::IndexConfig;

pub fn get_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    // TODO: more fields? tag, etc.? could be useful if we expand to other file types as well?
    schema_builder.add_text_field("filename", STRING | STORED);
    // TODO: make sure stemming tokenizer is running on this field
    schema_builder.add_text_field("contents", TEXT);
    schema_builder.build()
}

pub fn get_index(path: String) -> tantivy::Result<Index> {
    let schema = get_schema();
    let index_path = MmapDirectory::open(path)?;
    let index = Index::open_or_create(index_path, schema.clone())?;
    Ok(index)
}

pub fn reindex(index_config: IndexConfig) -> tantivy::Result<()> {
    let index = get_index(index_config.index_path.clone())?;
    let mut index_writer = index.writer(50_000_000)?;

    // reset the index
    index_writer.delete_all_documents()?;

    let schema = get_schema();
    let filename = schema.get_field("filename").unwrap();
    let contents = schema.get_field("contents").unwrap();

    // now add all the matching files to the index
    for expr in index_config.files {
        for entry in glob(&expr).expect("failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    let mut file = File::open(&path)?;
                    let doc = doc!(
                        filename => path.to_str().unwrap(),
                        contents => {
                            let mut contents = String::new();
                            file.read_to_string(&mut contents)?;
                            contents
                        },
                    );
                    index_writer.add_document(doc);
                }
                Err(e) => {
                    println!("Warning: {:?}", e);
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
    let index = get_index(index_config.index_path.clone())?;
    let schema = index.schema();
    // TODO: put this in a struct so can do fieldsStruct = FieldsStruct::from(schema)
    let contents = schema.get_field("contents").unwrap();
    let filename = schema.get_field("filename").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![contents]);
    let query = query_parser.parse_query(&query)?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
    let mut results = Vec::new();
    for (score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let fname = retrieved_doc.get_first(filename).unwrap().text().unwrap();
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
