//! Structures for representing 
//! [RDF Terms](https://www.w3.org/TR/rdf11-concepts/#dfn-rdf-term).
//!
//! # Examples
//!
//! ```
//! use rdf::term::*;
//!
//! let iri     = IRI::new("http://example.com/moomin");
//! let node    = BNode::new();
//! let literal = Literal::new("moomin", None, None);
//! ```
extern crate snowflake;

const LANG_STRING_IRI: &'static str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#langString";
const XSD_STRING_IRI:  &'static str = "http://www.w3.org/2001/XMLSchema#string";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Term<'a> {
    IRI(IRI<'a>),
    BNode(BNode),
    Literal(Literal<'a>),
}

/// An [IRI](https://www.w3.org/TR/rdf11-concepts/#dfn-iri) as an RDF Term.
///
/// # Examples
///
/// ```
/// use rdf::term::IRI;
///
/// let iri = IRI::new("http://example.com/moomin");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// Newly initialized nodes are guarenteed to be unique within the process. This means two 
/// separately initialized Blank Nodes will never be equal within a process.
///
/// See [Snowflake](https://stebalien.github.io/snowflake/snowflake/struct.ProcessUniqueId.html) for
/// details about the identifier implementation.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
/// RDF 1.1 types "simple literals" as `xsd:string`. This is used as a default type when neither a
/// datatype nor a language tag is given. Literals initialized with a language tag must be of type
/// `rdf:langString`; this type is forced if a tag is passed but no datatype is given.
///
/// # Examples
///
/// ```
/// use rdf::term::{Literal, IRI};
/// 
/// let literal = Literal::new("moomin", None, None);
/// let date    = Literal::new("2016-05-22", 
///                            Some(IRI::new("http://www.w3.org/2001/XMLSchema#date")), 
///                            None);
///
/// let lang = Literal::new_lang_string("Today", "@en");
/// ```
///
/// # Panics
///
/// When a language tag is passed with a non-`rdf:langString` datatype.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal<'a> {
    lexical:  String,
    datatype: IRI<'a>,
    lang:     Option<&'a str>,
}

impl<'a> Literal<'a> {
    pub fn new(lexical: &str, datatype: Option<IRI<'a>>, lang: Option<&'a str>) -> Self {
        match datatype {
            Some(iri) => {
                if iri == IRI::new(LANG_STRING_IRI) {
                    Literal { lexical: lexical.to_string(), datatype: iri, lang: lang }
                } else {
                    match lang {
                        None => { Literal { lexical: lexical.to_string(), datatype: iri, lang: lang } },
                        _    => { panic!("Initializing a Literal with of non-langString datatype and language tag") }
                    }
                }
            },
            None => {
                match lang {
                    Some(lang) => { Literal::new_lang_string(lexical, lang) },
                    None       => { Literal::new_string(lexical) }
                    
                }
            }
        } 
    }

    pub fn new_string(lexical: &str) -> Self {
        Literal { lexical: lexical.to_string(), 
                  datatype: IRI::new(XSD_STRING_IRI),
                  lang: None }
    }

    pub fn new_lang_string(lexical: &str, lang: &'a str) -> Self {
        Literal { lexical: lexical.to_string(), 
                  datatype: IRI::new(LANG_STRING_IRI), 
                  lang: Some(lang) }
    }
}
