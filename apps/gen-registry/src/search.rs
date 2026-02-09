//! Full-text search engine using Tantivy

use crate::{
    error::{Error, Result},
    models::SearchIndex,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Schema, STORED, TEXT},
    Index, IndexWriter, ReloadPolicy,
};
use tracing::{debug, info};

/// Search query
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            limit: 20,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub module_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub score: f32,
}

/// Search engine
pub struct SearchEngine {
    index: Index,
    writer: IndexWriter,
    schema: Schema,
}

impl SearchEngine {
    /// Create new search engine
    pub async fn new(data_dir: &str) -> Result<Self> {
        info!("Initializing search engine");

        // Build schema
        let mut schema_builder = Schema::builder();
        let module_id = schema_builder.add_text_field("module_id", TEXT | STORED);
        let name = schema_builder.add_text_field("name", TEXT | STORED);
        let description = schema_builder.add_text_field("description", TEXT | STORED);
        let keywords = schema_builder.add_text_field("keywords", TEXT);
        let schema = schema_builder.build();

        // Create index
        let index_path = Path::new(data_dir).join("search-index");
        std::fs::create_dir_all(&index_path)
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        let index = Index::create_in_dir(&index_path, schema.clone())
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        let writer = index
            .writer(50_000_000)
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        Ok(Self {
            index,
            writer,
            schema,
        })
    }

    /// Index a module
    pub async fn index_module(&self, index: &SearchIndex) -> Result<()> {
        debug!("Indexing module {}", index.module_id);

        let module_id = self.schema.get_field("module_id").unwrap();
        let name = self.schema.get_field("name").unwrap();
        let description = self.schema.get_field("description").unwrap();
        let keywords = self.schema.get_field("keywords").unwrap();

        let keywords_str = index.keywords.iter().cloned().collect::<Vec<_>>().join(" ");

        let mut doc = tantivy::Document::new();
        doc.add_text(module_id, &index.module_id);
        doc.add_text(name, &index.module_id); // Will be fetched from registry
        doc.add_text(description, "");
        doc.add_text(keywords, &keywords_str);

        self.writer
            .add_document(doc)
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        self.writer
            .commit()
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        Ok(())
    }

    /// Search modules
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        debug!("Searching for: {}", query.text);

        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        let searcher = reader.searcher();

        // Parse query
        let name = self.schema.get_field("name").unwrap();
        let description = self.schema.get_field("description").unwrap();
        let keywords = self.schema.get_field("keywords").unwrap();

        let query_parser = QueryParser::for_index(&self.index, vec![name, description, keywords]);
        let query = query_parser
            .parse_query(&query.text)
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        // Execute search
        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(query.limit))
            .map_err(|e| Error::SearchIndexError(e.to_string()))?;

        // Convert results
        let module_id_field = self.schema.get_field("module_id").unwrap();
        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher
                .doc(doc_address)
                .map_err(|e| Error::SearchIndexError(e.to_string()))?;

            if let Some(module_id_value) = retrieved_doc.get_first(module_id_field) {
                if let Some(module_id) = module_id_value.as_str() {
                    results.push(SearchResult {
                        module_id: module_id.to_string(),
                        name: module_id.to_string(), // Will be enriched from registry
                        description: String::new(),
                        version: "latest".to_string(),
                        score,
                    });
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_search_engine() {
        let temp_dir = TempDir::new().unwrap();
        let engine = SearchEngine::new(temp_dir.path().to_str().unwrap())
            .await
            .unwrap();

        let mut index = SearchIndex {
            module_id: "io.univrs.auth".to_string(),
            keywords: ["authentication", "security"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            popularity_score: 100,
            last_indexed: chrono::Utc::now(),
        };

        engine.index_module(&index).await.unwrap();

        let query = SearchQuery::new("authentication");
        let results = engine.search(&query).await.unwrap();
        assert!(!results.is_empty());
    }
}
