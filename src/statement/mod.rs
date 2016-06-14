//! An [RDF Statement](https://www.w3.org/TR/rdf11-concepts/#dfn-rdf-statement).
//!
//! # Examples
//!
//! ```
//! use rdf::statement::*;
//! ```

use term::Term;

/// A Statement
///
/// # Examples
///
/// ```
/// use rdf::term::*;
/// use rdf::statement::Statement;
///
/// let node = Term::BNode(BNode::new());
/// let iri  = Term::IRI(IRI::new("http://example.com/relation"));
/// let lit  = Term::Literal(Literal::new("moomin", None, None));
///
/// let stmt = Statement::new(node, iri, lit);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Statement<'a> {
    subject:   Term<'a>,
    predicate: Term<'a>,
    object:    Term<'a>,
}

impl<'a> Statement<'a> {
    pub fn new(s: Term<'a>, p: Term<'a>, o: Term<'a>) {
        Statement { subject: s, predicate: p, object: o };
    }
}
