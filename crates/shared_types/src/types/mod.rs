//! Type modules - organized by domain.

pub mod backlink;
pub mod embed;
pub mod event;
pub mod folder;
pub mod import;
pub mod note;
pub mod property;
pub mod query;
pub mod query_embed;
pub mod schedule;
pub mod search;
pub mod tag;
pub mod todo;
pub mod vault;

// Re-export all types for convenience
pub use backlink::*;
pub use embed::*;
pub use event::*;
pub use folder::*;
pub use import::*;
pub use note::*;
pub use property::*;
pub use query::*;
pub use query_embed::*;
pub use schedule::*;
pub use search::*;
pub use tag::*;
pub use todo::*;
pub use vault::*;
