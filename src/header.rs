use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::map;
use nom::sequence::{pair, preceded, tuple};
use nom::IResult;

use crate::basic::{Identifier, Literal, Separator};
use crate::Parser;

// Include         ::=  'include' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct Include<'a>(Literal<'a>);

impl<'a> Parser<'a> for Include<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(pair(tag("include"), Separator::parse), Literal::parse),
            Self,
        )(input)
    }
}

// CppInclude      ::=  'cpp_include' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppInclude<'a>(Literal<'a>);

impl<'a> Parser<'a> for CppInclude<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(pair(tag("cpp_include"), Separator::parse), Literal::parse),
            Self,
        )(input)
    }
}

// Namespace       ::=  ( 'namespace' ( NamespaceScope Identifier ) )
#[derive(Eq, PartialEq, Debug, Clone)]
pub struct Namespace<'a> {
    pub scope: NamespaceScope<'a>,
    pub name: Identifier<'a>,
}

// NamespaceScope  ::=  '*' | 'c_glib' | 'rust' | 'cpp' | 'delphi' | 'haxe' | 'go' | 'java' |
// 'js' | 'lua' | 'netstd' | 'perl' | 'php' | 'py' | 'py.twisted' | 'rb' | 'st' | 'xsd'
// We add rust into it.
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct NamespaceScope<'a>(&'a str);

impl<'a> Parser<'a> for Namespace<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("namespace"),
                preceded(Separator::parse, NamespaceScope::parse),
                preceded(Separator::parse, Identifier::parse),
            )),
            |(_, scope, name)| Self { scope, name },
        )(input)
    }
}

impl<'a> Parser<'a> for NamespaceScope<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            alt((
                tag("*"),
                tag("c_glib"),
                tag("rust"),
                tag("cpp"),
                tag("delphi"),
                tag("haxe"),
                tag("go"),
                tag("java"),
                tag("js"),
                tag("lua"),
                tag("netstd"),
                tag("perl"),
                tag("php"),
                tag("py"),
                tag("py.twisted"),
                tag("rb"),
                tag("st"),
                tag("xsd"),
            )),
            Self,
        )(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_include() {
        assert_eq!(
            Include::parse("include 'another.thrift'").unwrap().1,
            Include::from(Literal::from("another.thrift"))
        )
    }

    #[test]
    fn test_namespace() {
        assert_eq!(
            Namespace::parse("namespace * MyNamespace").unwrap().1,
            Namespace {
                scope: NamespaceScope::from("*"),
                name: Identifier::from("MyNamespace")
            }
        )
    }
}
