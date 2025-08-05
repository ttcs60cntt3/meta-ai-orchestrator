//! RAG (Retrieval-Augmented Generation) engine trait

use async_trait::async_trait;
use meta_ai_common::{
    types::{Document, Embedding, SearchResult},
    error::Result,
};

/// RAG engine trait for document indexing and retrieval
#[async_trait]
pub trait RagEngine: Send + Sync {
    /// Index a document
    async fn index_document(&self, document: Document) -> Result<()>;
    
    /// Index multiple documents
    async fn index_documents(&self, documents: Vec<Document>) -> Result<IndexResult>;
    
    /// Search for relevant documents
    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>>;
    
    /// Search with embedding vector
    async fn search_by_embedding(&self, embedding: &Embedding, top_k: usize) -> Result<Vec<SearchResult>>;
    
    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Embedding>;
    
    /// Delete a document
    async fn delete_document(&self, document_id: &str) -> Result<()>;
    
    /// Update a document
    async fn update_document(&self, document: Document) -> Result<()>;
    
    /// Get collection statistics
    async fn get_stats(&self) -> Result<CollectionStats>;
    
    /// Clear all documents
    async fn clear_collection(&self) -> Result<()>;
}

/// Result of bulk indexing operation
#[derive(Debug)]
pub struct IndexResult {
    pub total_documents: usize,
    pub successful: usize,
    pub failed: usize,
    pub duration_ms: u64,
    pub errors: Vec<IndexError>,
}

/// Indexing error
#[derive(Debug)]
pub struct IndexError {
    pub document_id: String,
    pub error: String,
}

/// Collection statistics
#[derive(Debug, Clone)]
pub struct CollectionStats {
    pub total_documents: usize,
    pub total_embeddings: usize,
    pub embedding_dimension: usize,
    pub index_size_bytes: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Embedding model trait
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Embedding>;
    
    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Embedding>>;
    
    /// Get embedding dimension
    fn dimension(&self) -> usize;
    
    /// Get model name
    fn model_name(&self) -> &str;
}

/// Document chunking strategy
#[derive(Debug, Clone)]
pub enum ChunkingStrategy {
    /// Fixed size chunks with overlap
    FixedSize { size: usize, overlap: usize },
    /// Sentence-based chunking
    Sentence { max_sentences: usize },
    /// Paragraph-based chunking
    Paragraph { max_paragraphs: usize },
    /// Semantic chunking based on topic changes
    Semantic { similarity_threshold: f32 },
}

/// Document processor for chunking and preprocessing
#[async_trait]
pub trait DocumentProcessor: Send + Sync {
    /// Process a document into chunks
    async fn process_document(&self, content: &str) -> Result<Vec<DocumentChunk>>;
    
    /// Get chunking strategy
    fn chunking_strategy(&self) -> &ChunkingStrategy;
    
    /// Preprocess text (cleaning, normalization)
    fn preprocess_text(&self, text: &str) -> String;
}

/// Document chunk
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub content: String,
    pub start_offset: usize,
    pub end_offset: usize,
    pub metadata: meta_ai_common::types::Metadata,
}

/// Vector store trait for embedding storage
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Store embeddings
    async fn store_embeddings(&self, embeddings: Vec<(String, Embedding)>) -> Result<()>;
    
    /// Search similar embeddings
    async fn search_similar(
        &self,
        query_embedding: &Embedding,
        top_k: usize,
        score_threshold: Option<f32>,
    ) -> Result<Vec<(String, f32)>>;
    
    /// Delete embeddings by IDs
    async fn delete_embeddings(&self, ids: &[String]) -> Result<()>;
    
    /// Get store info
    async fn info(&self) -> Result<VectorStoreInfo>;
}

/// Vector store information
#[derive(Debug, Clone)]
pub struct VectorStoreInfo {
    pub backend: String,
    pub total_vectors: usize,
    pub dimension: usize,
    pub index_type: String,
    pub distance_metric: DistanceMetric,
}

/// Distance metrics for similarity search
#[derive(Debug, Clone, Copy)]
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
    Manhattan,
}

/// In-memory embedding cache
pub struct EmbeddingCache {
    cache: dashmap::DashMap<String, Embedding>,
    max_size: usize,
}

impl EmbeddingCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: dashmap::DashMap::new(),
            max_size,
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Embedding> {
        self.cache.get(key).map(|e| e.clone())
    }
    
    pub fn insert(&self, key: String, embedding: Embedding) {
        if self.cache.len() >= self.max_size {
            // Simple eviction: remove first item
            if let Some(first_key) = self.cache.iter().next().map(|e| e.key().clone()) {
                self.cache.remove(&first_key);
            }
        }
        self.cache.insert(key, embedding);
    }
    
    pub fn clear(&self) {
        self.cache.clear();
    }
}