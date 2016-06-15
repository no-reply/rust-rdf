//! [NTriples](https://www.w3.org/TR/n-triples/)
//! 
//! TODO: parse entire documents, not just individual triples;
//!       build statements in ntriple combinator;
//!       clean up and improve term matching;
//!       
//! # Examples
//!
//! ```
//!
//! ```

use std::str;
use term::{BNode, IRI, Literal, Term};
use nom::{is_space, is_alphanumeric, line_ending};

fn to_iri(iri_bytes: &[u8]) -> Term {
    match str::from_utf8(iri_bytes) {
        Ok(utf)  => Term::IRI(IRI::new(utf)),
        Err(err) => panic!("Invalid UTF-8 byte sequence: {}", err),
    }
}

/// TODO: use same node for same node ids in the document
fn to_bnode(node_bytes: &[u8]) -> Term {
    Term::BNode(BNode::new())
}

/// TODO: handle lang tags and datatypes
fn to_literal(literal_bytes: &[u8]) -> Term {
    match str::from_utf8(literal_bytes) {
        Ok(utf)  => Term::Literal(Literal::new(utf, None, None)),
        Err(err) => panic!("Invalid UTF-8 byte sequence: {}", err),
    }
}

named!(iri_chars, is_not!(">"));
named!(iriref, delimited!(char!('<'), iri_chars, char!('>')));
named!(iri<Term>, map!(iriref, to_iri));

named!(bnode_label_chars, take_while!(is_alphanumeric));
named!(bnode_label, preceded!(tag!("_:"), bnode_label_chars));
named!(bnode<Term>, map!(bnode_label, to_bnode));

named!(literal_char, is_not!("\""));
named!(literal_quote, delimited!(char!('"'), literal_char, char!('"')));
named!(literal<Term>, map!(literal_quote, to_literal));

named!(subject<Term>, alt_complete!(iri | bnode));
named!(object<Term>, alt_complete!(subject | literal));

/// TODO: bulid Statement from triple
named!(ntriple<&[u8], &str>, 
       chain!(
           s: subject ~
           take_while!(is_space)? ~
           p: iri ~
           take_while!(is_space)? ~
           o: object ~
           take_while!(is_space)? ~
           char!('.'),

           || { "found!" }
           )
       );
       
#[cfg(test)]
mod tests {
    use super::{bnode_label, iriref, literal_quote, ntriple};
    use nom::IResult::Done;

    #[test]
    fn bnode_labels() {
        assert_eq!(bnode_label(&b"_:abc"[..]), Done(&b""[..], &b"abc"[..]));
        assert!(bnode_label(&b"abc"[..]).is_err())
    }

    #[test]
    fn irirefs() {
        assert_eq!(iriref(&b"<ab>"[..]), Done(&b""[..], &b"ab"[..]));
    }

    #[test]
    fn literal_quotes() {
        assert_eq!(literal_quote(&b"\"moomin\""[..]), Done(&b""[..], &b"moomin"[..]))
    }

    #[test]
    fn ntriples() {
        assert_eq!(ntriple(&b"<ab> <cd> <ef> ."[..]),           Done(&b""[..], "found!"));
        assert_eq!(ntriple(&b"<ab>\t<cd>\t  <ef> ."[..]),       Done(&b""[..], "found!"));
        assert_eq!(ntriple(&b"_:node1 <iri> \"moomin\" ."[..]), Done(&b""[..], "found!"));

        // Errors
        assert!(ntriple(&b"_:node1 :node2 \"moomin\" ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> <iri> ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> ."[..]).is_err());
        assert!(ntriple(&b"<iri> <iri> <iri> <iri> ."[..]).is_err());
    }
}
