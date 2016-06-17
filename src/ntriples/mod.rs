//! [NTriples](https://www.w3.org/TR/n-triples/)
//!
//! TODO: - clean up and improve base charset matching (e.g. `iri_chars`).
//!       - handle document as bnode scope.
//!       - provide nice interfaces and error reporting.
//!
//! # Examples
//!
//! ```
//! use rdf::ntriples;
//! ```

use std::str;
use nom::{alpha, alphanumeric, eof, eol, space};
use nom::IResult::Done;

use graph::Graph;
use term::{BNode, IRI, Literal, Term};
use statement::Statement;

fn to_iri(iri_bytes: &[u8]) -> Term {
    Term::IRI(
        IRI::new(
            str::from_utf8(iri_bytes).unwrap()))
}

/// TODO: use same node for same node ids in the document
fn to_bnode(_: &[u8]) -> Term {
    Term::BNode(BNode::new())
}

/// PN_CHARS_BASE    ::= [A-Z] | [a-z] | [#x00C0-#x00D6] | [#x00D8-#x00F6] | [#x00F8-#x02FF] |
///                      [#x0370-#x037D] | [#x037F-#x1FFF] | [#x200C-#x200D] | [#x2070-#x218F] |
///                      [#x2C00-#x2FEF] | [#x3001-#xD7FF] | [#xF900-#xFDCF] | [#xFDF0-#xFFFD] |
///                      [#x10000-#xEFFFF]
/// PN_CHARS_U       ::= PN_CHARS_BASE | '_' | ':'
/// PN_CHARS         ::= PN_CHARS_U | '-' | [0-9] | #x00B7 | [#x0300-#x036F] | [#x203F-#x2040]
///
/// TODO: match correct charset
named!(bnode_label_chars, recognize!(alphanumeric));

/// BLANK_NODE_LABEL ::= '_:' (PN_CHARS_U | [0-9]) ((PN_CHARS | '.')* PN_CHARS)?
named!(bnode_label, preceded!(tag!("_:"), bnode_label_chars));
named!(bnode<Term>, map!(bnode_label, to_bnode));

/// TODO: match correct charset
named!(iri_chars, is_not!(">"));

/// IRIREF ::= '<' ([^#x00-#x20<>"{}|^`\] | UCHAR)* '>'"])
named!(iriref, delimited!(char!('<'), iri_chars, char!('>')));
named!(iri<Term>, map!(iriref, to_iri));

/// UCHAR                ::= '\u' HEX HEX HEX HEX | '\U' HEX HEX HEX HEX HEX HEX HEX HEX
/// ECHAR                ::= '\' [tbnrf"'\]"]
/// HEX                  ::= [0-9] | [A-F] | [a-f]
///
/// TODO: match correct charset; handle escaped '"' chars.
named!(literal_char, is_not!("\""));

/// STRING_LITERAL_QUOTE ::= '"' ([^#x22#x5C#xA#xD] | ECHAR | UCHAR)* '"'
named!(literal_quote, delimited!(char!('"'), recognize!(many0!(literal_char)), char!('"')));

/// LANGTAG ::= '@' [a-zA-Z]+ ('-' [a-zA-Z0-9]+)*
named!(bcp, recognize!(chain!(
    alpha ~
        many0!(complete!(preceded!(char!('-'), alphanumeric))),
    || { false }
    )));

named!(language, preceded!(char!('@'), bcp));

named!(datatype, preceded!(tag!("^^"), iriref));

/// literal ::= STRING_LITERAL_QUOTE ('^^' IRIREF | LANGTAG)?
/// TODO: this still matches cases like `"\"moomin\"^^<datatype>@fi"`. `Literal` will panic when
///       initializing a "literal" like thisunless `rdf:langString` is given as the datatype,
///       but it should be fixed anyway.
named!(literal<Term>, chain!(
    quote: literal_quote ~
        dtype: complete!(map!(datatype, |d| { IRI::new(str::from_utf8(d).unwrap()) }))? ~
        lang:  complete!(map!(language, |l| { str::from_utf8(l).unwrap() }))?,
    || {
        match str::from_utf8(quote) {
            Ok(utf)  => Term::Literal(Literal::new(utf, dtype, lang)),
            Err(err) => panic!("Invalid UTF-8 byte sequence: {}", err),
        }
    })
       );

/// subject ::= IRIREF | BLANK_NODE_LABEL
named!(subject<Term>, alt_complete!(iri | bnode));

/// object ::= IRIREF | BLANK_NODE_LABEL | literal
named!(object<Term>, alt_complete!(subject | literal));

// TODO: fix eol handling for comments!
//named!(comment, preceded!(char!('#'), take_till!(is_not_s!(eol))));
named!(comment, preceded!(char!('#'), take_until!("\n")));

/// triple ::= subject predicate object '.'
named!(ntriple<&[u8], Statement>,
       chain!(
           space? ~
               s: subject ~
               space? ~
               p: iri ~ // predicate
               space? ~
               o: object ~
               space? ~
               char!('.') ~
               space? ~
               complete!(comment)?,

           || { Statement::new(s, p, o) }
           )
       );

/// (EOL triple)
named!(eol_ntriple<&[u8], Statement>, chain!(eol ~ nt: ntriple, || { nt }));

/// ntriplesDoc ::= triple? (EOL triple)* EOL?
/// NOTE: handling comments, which are unaccounted for in the grammar
named!(ntriples<&[u8], Graph>,
        chain!(
            mut graph: map!(take!(0), |_| { Graph::new() }) ~
                // eat all comments at the start of the document
                many0!(complete!(chain!(space? ~ comment ~ eol?, || { false }))) ~
                // triple?
                map!(complete!(ntriple), |nt| { graph.insert(nt) })? ~
                // ((EOL triple) | (EOL comment))*
                many0!(alt_complete!(
                    map!(complete!(eol_ntriple), |nt| { graph.insert(nt) }) |
                    chain!(complete!(eol) ~ space? ~ comment, || { false }))
                       ) ~
                // EOL?
                complete!(eol)?,
            || { graph }
            )
        );

pub fn parse(input: &[u8]) -> Graph {
    match ntriples(input) {
        Done(_, out) => out,
        _            => panic!("failed to parse"),
    }
}

#[cfg(test)]
mod tests {
    use super::{bnode_label, bnode, comment, datatype, iri, iriref, language,
                literal, literal_quote, ntriple, ntriples};
    use super::super::term::{BNode, IRI, Literal, Term};
    use super::super::graph::Graph;
    use nom::IResult::Done;
    use nom::GetInput;

    #[test]
    fn bnode_label_test() {
        assert_eq!(bnode_label(&b"_:abc"[..]), Done(&b""[..], &b"abc"[..]));
        assert!(bnode_label(&b"abc"[..]).is_err())
    }

    // #[test]
    // fn bnodes() {
    //     // need to add bnode scopes
    // }

    #[test]
    fn comment_test() {
        assert_eq!(comment(&b"#_:abc\n<next>"[..]), Done(&b"\n<next>"[..], &b"_:abc"[..]));
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
    fn datatype_test() {
        assert_eq!(datatype(&b"^^<a>"[..]), Done(&b""[..], &b"a"[..]));
        assert_eq!(datatype(&b"^^<abc>"[..]), Done(&b""[..], &b"abc"[..]));

        assert!(datatype(&b"<abc>"[..]).is_err());
        assert!(datatype(&b"^^abc"[..]).is_err());
    }

    #[test]
    fn lang_test() {
        assert_eq!(language(&b"@de"[..]), Done(&b""[..], &b"de"[..]));
        assert_eq!(language(&b"@DE"[..]), Done(&b""[..], &b"DE"[..]));
        assert_eq!(language(&b"@de-AT"[..]), Done(&b""[..], &b"de-AT"[..]));
        assert_eq!(language(&b"@sr-Latn-RS"[..]), Done(&b""[..], &b"sr-Latn-RS"[..]));
        assert_eq!(language(&b"@es-419"[..]), Done(&b""[..], &b"es-419"[..]));

        assert_eq!(language(&b"@de-"[..]), Done(&b"-"[..], &b"de"[..]));
        assert_eq!(language(&b"@sr-Latn-"[..]), Done(&b"-"[..], &b"sr-Latn"[..]));

        assert!(language(&b"@12"[..]).is_err());
        assert!(language(&b"de"[..]).is_err());
    }

    #[test]
    fn literal_quote_test() {
        assert_eq!(literal_quote(&b"\"moomin\""[..]), Done(&b""[..], &b"moomin"[..]))
    }

    #[test]
    fn literal_test() {
        let empty_literal = Term::Literal(Literal::new("", None, None));
        assert_eq!(literal(&b"\"\""[..]), Done(&b""[..], empty_literal));

        let string_literal = Term::Literal(Literal::new("moomin", None, None));
        assert_eq!(literal(&b"\"moomin\""[..]), Done(&b""[..], string_literal));

        let datatype_literal = Term::Literal(Literal::new("moomin", Some(IRI::new("moomin")), None));
        assert_eq!(literal(&b"\"moomin\"^^<moomin>"[..]), Done(&b""[..], datatype_literal));

        let lang_literal = Term::Literal(Literal::new("moomin", None, Some("de")));
        assert_eq!(literal(&b"\"moomin\"@de"[..]), Done(&b""[..], lang_literal));
    }

    #[test]
    fn ntriple_test() {
        // TODO: test actual output; this will be easier when Graph is more mature, bgp matching
        //       is possible, etc...
        assert!(ntriple(&b"<ab> <cd> <ef> ."[..]).is_done());
        assert!(ntriple(&b"<ab><cd><ef>."[..]).is_done());
        assert!(ntriple(&b"   <ab>\t<cd> <ef> ."[..]).is_done());
        assert!(ntriple(&b"_:node1\t<cd> \"moomin valley\t\n...\" ."[..]).is_done());
        assert!(ntriple(&b"<ab> <cd> _:node1 ."[..]).is_done());
    }

    #[test]
    fn ntriple_errors_test() {
        // Errors
        assert!(ntriple(&b"<> <p> \"moomin\" ."[..]).is_err());
        assert!(ntriple(&b"_:node1 _:node2 \"moomin\" ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> <iri> ."[..]).is_err());
        assert!(ntriple(&b"\"moomin\" <iri> ."[..]).is_err());
        assert!(ntriple(&b"<iri> <iri> <iri> <iri> ."[..]).is_err());
        assert!(ntriple(&b"_:node1 <iri> \"mo\"omin\" ."[..]).is_err());
    }

    #[test]
    fn ntriples_test() {
        assert!(ntriples(&b"<ab> <cd> <ef> ."[..]).is_done());
        assert!(ntriples(&b"<ab> <cd> <ef> .\n\n"[..]).is_done());
        assert!(ntriples(&b"<ab> <cd> <ef> .\n<s> <p> <o> .   \n<s> <p> <o2> .      #comment\t\n<s><p><o3>."[..]).is_done());
        assert!(ntriples(&b"   #abc\n#  def\n  # abc\n<ab> <cd> <ef> .\n<s> <p> <o> .   \n#comment\n<s> <p> <o2> .      #comment\t\n<s><p><o3>."[..]).is_done());
        assert!(ntriples(&b"<ab> <cd> <ef> .\n<s> <p> <o> .   \n<s> <p> <o2> .    \n"[..]).is_done());

        assert_eq!(ntriples(&b"<ab> <cd> <ef> .\n<s> <p> <o> .   \n<s> <p> <o2> .    \n<abc>"[..]).remaining_input(), Some(&b"<abc>"[..]));
    }

    #[test]
    fn ntriples_empty_test() {
        assert_eq!(ntriples(&b""[..]), Done(&b""[..], Graph::new()));

        // fixme! I require 2 `\n` chars, but leave one behind!
        //assert_eq!(ntriples(&b"### comment\n\n"[..]), Done(&b""[..], Graph::new()));
    }
}
