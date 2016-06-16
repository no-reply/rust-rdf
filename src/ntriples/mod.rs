//! [NTriples](https://www.w3.org/TR/n-triples/)
//!
//! TODO: parse entire documents, not just individual triples;
//!       build statements in ntriple combinator;
//!       clean up and improve term matching (e.g. `iri_chars`).
//!
//! # Examples
//!
//! ```
//! use rdf::ntriples;
//! ```

use std::str;
use nom::{eol, is_alphanumeric, space};
use nom::IResult::Done;

use graph::Graph;
use term::{BNode, IRI, Literal, Term};
use statement::Statement;

fn to_iri(iri_bytes: &[u8]) -> Term {
    match str::from_utf8(iri_bytes) {
        Ok(utf)  => Term::IRI(IRI::new(utf)),
        Err(err) => panic!("Invalid UTF-8 byte sequence: {}", err),
    }
}

/// TODO: use same node for same node ids in the document
fn to_bnode(_: &[u8]) -> Term {
    Term::BNode(BNode::new())
}

/// TODO: handle lang tags and datatypes
fn to_literal(literal_bytes: &[u8]) -> Term {
    match str::from_utf8(literal_bytes) {
        Ok(utf)  => Term::Literal(Literal::new(utf, None, None)),
        Err(err) => panic!("Invalid UTF-8 byte sequence: {}", err),
    }
}

named!(bnode_label_chars, take_while!(is_alphanumeric));
named!(bnode_label, preceded!(tag!("_:"), bnode_label_chars));
named!(bnode<Term>, map!(bnode_label, to_bnode));

named!(iri_chars, is_not!(">"));
named!(iriref, delimited!(char!('<'), iri_chars, char!('>')));
named!(iri<Term>, map!(iriref, to_iri));

named!(literal_char, is_not!("\""));
named!(literal_quote, delimited!(char!('"'), literal_char, char!('"')));
named!(literal<Term>, map!(literal_quote, to_literal));

named!(subject<Term>, alt_complete!(iri | bnode));
named!(object<Term>, alt_complete!(subject | literal));

// TODO: fix eol handling for comments!
//named!(comment, preceded!(char!('#'), is_not_s!(eol)));
named!(comment, preceded!(char!('#'), take_until!("\n")));
named!(line_comment, terminated!(comment, eol));


named!(ntriple<&[u8], Statement>,
       chain!(
           space? ~
               s: subject ~
               space? ~
               p: iri ~
               space? ~
               o: object ~
               space? ~
               char!('.'), // todo: handle comments

           || { Statement::new(s, p, o) }
           )
       );

named!(eol_ntriple<&[u8], Statement>,
       chain!(
           eol ~
               nt: ntriple,
           || { nt }
           )
       );

// named!(next_ntriple<&[u8], Statement>, preceded!(many0!(line_comment), ntriple));

// named!(first_ntriple<&[u8], Graph>,
//        chain!(
//            nt: ntriple,
//            || { let mut graph = Graph::new();
//                 graph.insert(nt);
//                 graph }
//            )
//        );

 named!(parse_ntriples<&[u8], Graph>,
        chain!(
            mut graph: map!(ntriple, |nt| { 
                let mut g = Graph::new();
                g.insert(nt);
                g 
            }) ~
                map!(ntriple, |nt| { graph.insert(nt) })? ~
                char!('\n')?,
                
                //many0!(pair!(eol, map!(ntriple, |nt| { graph.insert(nt) } ))) ~
            || { graph }
            )
        );

pub fn parse(input: &[u8]) -> Graph {
    match parse_ntriples(input) {
        Done(_, out) => out,
        _            => panic!("failed to parse"),
    }
}

#[cfg(test)]
mod tests {
    use super::{bnode_label, bnode, comment, iri, iriref, line_comment, literal, 
                literal_quote, parse_ntriples, ntriple};
    use super::super::term::{BNode, IRI, Literal, Term};
    use super::super::graph::Graph;
    use nom::IResult::Done;

    #[test]
    fn bnode_label_test() {
        assert_eq!(bnode_label(&b"_:abc"[..]), Done(&b""[..], &b"abc"[..]));
        assert!(bnode_label(&b"abc"[..]).is_err())
    }

    // #[test]
    // fn bnodes() {
    //     // need to patch bnodes for provided IDs for this to work
    //     assert!(false)
    // }

    #[test]
    fn comment_test() {
        assert_eq!(comment(&b"#_:abc\n<next>"[..]), Done(&b"\n<next>"[..], &b"_:abc"[..]));
    }

    #[test]
    fn line_comment_test() {
        assert_eq!(line_comment(&b"#_:abc\n<next>"[..]), 
                   Done(&b"<next>"[..], &b"_:abc"[..]));
    }

    #[test]
    fn iriref_test() {
        assert_eq!(iriref(&b"<ab>"[..]), Done(&b""[..], &b"ab"[..]));
    }

    #[test]
    fn iri_test() {
        let uri = Term::IRI(IRI::new("moomin"));
        assert_eq!(iri(&b"<moomin>"[..]), Done(&b""[..], uri));
        assert!(iri(&b"moomin>"[..]).is_err())
    }

    #[test]
    fn literal_quote_test() {
        assert_eq!(literal_quote(&b"\"moomin\""[..]), Done(&b""[..], &b"moomin"[..]))
    }

    #[test]
    fn literal_test() {
        let string_literal = Term::Literal(Literal::new("moomin", None, None));
        assert_eq!(literal(&b"\"moomin\""[..]), Done(&b""[..], string_literal));
    }

    #[test]
    fn ntriple_test() {
        // todo: test actual output
        assert!(ntriple(&b"<ab> <cd> <ef> ."[..]).is_done());
        assert!(ntriple(&b"<ab><cd><ef>."[..]).is_done());
        assert!(ntriple(&b"   <ab>\t<cd> <ef> ."[..]).is_done());
        assert!(ntriple(&b"_:node1\t<cd> \"moomin valley\t\n...\" ."[..]).is_done());
        assert!(ntriple(&b"<ab> <cd> _:node1 ."[..]).is_done());
    }

    #[test]
    fn ntriple_errors_test() {
        // Errors
        assert!(ntriple(&b"_:node1 _:node2 \"moomin\" ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> <iri> ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> ."[..]).is_err());
        assert!(ntriple(&b"<iri> <iri> <iri> <iri> ."[..]).is_err());
        assert!(ntriple(&b"_:node1 <iri> \"mo\"omin\" ."[..]).is_err());
    }

    #[test]
    fn parse_ntriples_test() {
        //assert_eq!(parse_ntriples(&b"<ab> <cd> <ef> .\n"[..]), Done(&b""[..], Graph::new()));
        assert_eq!(parse_ntriples(&b"<ab> <cd> <ef> .\n<s> <p> <o> .\n<s> <p> <o2> .\n"[..]), Done(&b""[..], Graph::new()));

        // assert_eq!(parse_ntriples(&b"<ab> <cd> <ef> .\n<s> <p> <o> .\n<s> <p> _:o .\n"[..]), Done(&b""[..], Graph::new()));
    }
}
