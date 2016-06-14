//! An [RDF Graph]().
//!
//! # Examples
//!
//! ```
//! use rdf::graph::*;
//! 
//! let graph = Graph::new();
//! ```

use statement::Statement;
use std::collections::HashSet;

pub type Graph<'a> = HashSet<Statement<'a>>;
