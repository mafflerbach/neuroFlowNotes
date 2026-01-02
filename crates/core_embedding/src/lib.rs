//! Core embedding crate for semantic search.
//!
//! This crate provides an LM Studio client for generating text embeddings
//! using the OpenAI-compatible API, along with background processing and
//! hybrid search combining FTS5 with vector similarity.

mod client;
mod hybrid;
mod queue;
mod types;

pub use client::EmbeddingClient;
pub use hybrid::hybrid_search;
pub use queue::{EmbeddingManager, EmbeddingQueue};
pub use types::*;
