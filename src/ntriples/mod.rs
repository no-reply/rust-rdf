//! [NTriples](https://www.w3.org/TR/n-triples/)
//!
//! # Examples
//!
//! ```
//! 
//! 
//! 
//! 
//! ```

use nom::{line_ending, is_space, is_alphanumeric};

named!(iri, is_not!(">"));
named!(iriref, delimited!(char!('<'), iri, char!('>')));

named!(bnode, preceded!(tag!("_:"), take_while!(is_alphanumeric)));

named!(literal_char, is_not!("\""));
named!(literal, delimited!(char!('"'), literal_char, char!('"')));

named!(subject, alt_complete!(iriref | bnode));
named!(object, alt_complete!(subject | literal));

named!(ntriple<&[u8], &str>, 
       chain!(
           s: subject ~
           take_while!(is_space)? ~
           p: iriref ~
           take_while!(is_space)? ~
           o: object ~
           take_while!(is_space)? ~
           char!('.'),

           || { "found!" }
           )
       );
       
#[cfg(test)]
mod tests {
    use super::{bnode, iriref, literal, ntriple};
    use nom::IResult::{Done, Incomplete};

    #[test]
    fn bnodes() {
        assert_eq!(bnode(&b"_:abc"[..]), Done(&b""[..], &b"abc"[..]))
    }

    #[test]
    fn irirefs() {
        assert_eq!(iriref(&b"<ab>"[..]), Done(&b""[..], &b"ab"[..]));
    }

    #[test]
    fn triples() {
        assert_eq!(ntriple(&b"<ab> <cd> <ef> ."[..]), Done(&b""[..], "found!"))
    }

    #[test]
    fn tabs_in_triples() {
        assert_eq!(ntriple(&b"<ab>\t<cd>\t  <ef> ."[..]), Done(&b""[..], "found!"))
    }

    #[test]
    fn literals() {
        assert_eq!(literal(&b"\"moomin\""[..]), Done(&b""[..], &b"moomin"[..]))
    }

    #[test]
    fn ntriples() {
        assert_eq!(ntriple(&b"_:node1 <iri> \"moomin\" ."[..]), Done(&b""[..], "found!"))
    }
}
