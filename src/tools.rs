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


pub fn get_schema() -> Schema {
    let mut schema_builder = Schema::builder();
    // TODO: more fields? tag, etc.? could be useful if we expand to other file types as well?
    schema_builder.add_text_field("filename", STRING | STORED);
    // TODO: make sure stemming tokenizer is running on this field
    schema_builder.add_text_field("contents", TEXT);
    let schema = schema_builder.build();
    schema
}

pub fn reindex(index: Index) -> tantivy::Result<()> {
    let mut index_writer = index.writer(50_000_000)?;
    index_writer.delete_all_documents()?;

    let schema = get_schema();
    let filename = schema.get_field("filename").unwrap();
    let contents = schema.get_field("contents").unwrap();

    // TODO: no hard coding
    for entry in WalkDir::new("/home/swalladge/wiki/")
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            // TODO: make this customizable
            !e.file_name()
                .to_str()
                .map(|s| s.starts_with("."))
                .unwrap_or(false)
        })
        .filter_map(|e| e.ok())
    {
        if entry
            .file_name()
            .to_owned()
            .into_string()
            .unwrap()
            .ends_with(".md") // TODO: no hardcode
        {
            let title = entry.path().file_name().unwrap();
            let mut file = File::open(entry.path())?;
            let doc = doc!(
                filename => entry.path().to_str().unwrap(),
                contents => {
                    let mut contents = String::new();
                    file.read_to_string(&mut contents)?;
                    contents
                },
            );
            index_writer.add_document(doc);
        }
    }

    index_writer.commit()?;

    Ok(())
}

pub fn search(index: Index, query: String, limit: usize) -> tantivy::Result<()> {
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
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        let fname = retrieved_doc.get_first(filename).unwrap().text().unwrap();
        println!("{}", fname);
    }
    Ok(())
}
