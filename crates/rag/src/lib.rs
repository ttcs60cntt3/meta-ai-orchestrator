#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

//! RAG (Retrieval-Augmented Generation) implementation

pub mod engine;
pub mod embeddings;
pub mod vector_store;

pub use engine::QdrantRagEngine;
pub use embeddings::BGEEmbeddingModel;
pub use vector_store::QdrantVectorStore;