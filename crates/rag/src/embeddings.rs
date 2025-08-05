//! Embedding model implementation

use async_trait::async_trait;
use meta_ai_common::{error::{Error, Result}, types::Embedding};
use meta_ai_core::rag::EmbeddingModel;

pub struct BGEEmbeddingModel;

impl BGEEmbeddingModel {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl EmbeddingModel for BGEEmbeddingModel {
    async fn embed(&self, _text: &str) -> Result<Embedding> {
        Err(Error::Rag("Not implemented".to_string()))
    }
    
    async fn embed_batch(&self, _texts: Vec<&str>) -> Result<Vec<Embedding>> {
        Err(Error::Rag("Not implemented".to_string()))
    }
    
    fn dimension(&self) -> usize {
        768
    }
    
    fn model_name(&self) -> &str {
        "BAAI/bge-base-en-v1.5"
    }
}