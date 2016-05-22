//! Structures for representing 
//! [RDF Terms](https://www.w3.org/TR/rdf11-concepts/#dfn-rdf-term).
//!
//! # Examples
//!
//! ```
//! use rdf::term;
//!
//! ```
extern crate snowflake;

//const LANG_STRING_IRI: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
const XSD_STRING_IRI:  &'static str = "http://www.w3.org/2001/XMLSchema#string";

/// An [IRI](https://www.w3.org/TR/rdf11-concepts/#dfn-iri) as an RDF Term.
///
/// # Examples
///
/// ```
/// use rdf::term::IRI;
///
/// let iri = IRI::new("http://example.com/moomin");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IRI<'a> {
    value: &'a str,
}

impl<'a> IRI<'a> {
    pub fn new(value: &'a str) -> Self {
        IRI { value: value }
    }
}

use self::snowflake::ProcessUniqueId;
type BNodeId = ProcessUniqueId;

/// A [Blank Node](https://www.w3.org/TR/rdf11-concepts/#dfn-blank-node) as an 
/// RDF Term.
///
/// Newly initialized nodes are guarenteed to be unique within the process. See
/// [Snowflake](https://stebalien.github.io/snowflake/snowflake/struct.ProcessUniqueId.html) for
/// details about the `BNodeId` implementation.
///
/// # Examples
///
/// ```
/// use rdf::term::BNode;
///
/// let node  = BNode::new();
///
/// assert_eq!(node, node);
/// assert_eq!(node, node.clone());
///
/// let other = BNode::new();
///
/// assert!(node != other);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BNode {
    id: BNodeId,
}

impl BNode {
    pub fn new() -> Self {
        BNode { id: BNodeId::new() }
    }
}

/// An [RDF Literal](https://www.w3.org/TR/rdf11-concepts/#dfn-literal).
///
/// Literals are composed of a `lexical` form, a `datatype` `IRI`, and (optionally) a `lang` tag.
///
/// # Examples
///
/// ```
/// use rdf::term::Literal;
/// 
/// let literal = Literal::new("moomin", None, None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal<'a> {
    lexical:  String,
    datatype: IRI<'a>,
    lang:     Option<&'a str>,
}

impl<'a> Literal<'a> {
    pub fn new(lexical: &str, datatype: Option<IRI<'a>>, lang: Option<&'a str>) -> Self {
        match datatype {
            Some(iri) => {
                Literal { lexical: lexical.to_string(), datatype: iri, lang: lang } 
            },
            None => {
                Literal { lexical: lexical.to_string(), datatype: IRI::new(XSD_STRING_IRI), lang: None }
            }
        } 
    }
}
