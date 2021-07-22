use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::IResult;

use crate::basic::Separator;
use crate::definition::{Const, Enum, Exception, Service, Struct, Typedef, Union};
use crate::header::{CppInclude, Include, Namespace};
use crate::Parser;

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Document<'a> {
    pub includes: Vec<Include<'a>>,
    pub cpp_includes: Vec<CppInclude<'a>>,
    pub namespaces: Vec<Namespace<'a>>,
    pub typedefs: Vec<Typedef<'a>>,
    pub consts: Vec<Const<'a>>,
    pub enums: Vec<Enum<'a>>,
    pub structs: Vec<Struct<'a>>,
    pub unions: Vec<Union<'a>>,
    pub exceptions: Vec<Exception<'a>>,
    pub services: Vec<Service<'a>>,
}

impl<'a> Parser<'a> for Document<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let mut target = Self::default();
        let includes = &mut target.includes;
        let cpp_includes = &mut target.cpp_includes;
        let namespaces = &mut target.namespaces;
        let typedefs = &mut target.typedefs;
        let consts = &mut target.consts;
        let enums = &mut target.enums;
        let structs = &mut target.structs;
        let unions = &mut target.unions;
        let exceptions = &mut target.exceptions;
        let services = &mut target.services;

        let (remains, _) = many0(delimited(
            opt(Separator::parse),
            alt((
                map(Include::parse, |i| includes.push(i)),
                map(CppInclude::parse, |i| cpp_includes.push(i)),
                map(Namespace::parse, |i| namespaces.push(i)),
                map(Typedef::parse, |i| typedefs.push(i)),
                map(Const::parse, |i| consts.push(i)),
                map(Enum::parse, |i| enums.push(i)),
                map(Struct::parse, |i| structs.push(i)),
                map(Union::parse, |i| unions.push(i)),
                map(Exception::parse, |i| exceptions.push(i)),
                map(Service::parse, |i| services.push(i)),
            )),
            opt(Separator::parse),
        ))(input)?;
        Ok((remains, target))
    }
}

#[cfg(test)]
mod tests {
    use crate::basic::Literal;

    use super::*;

    #[test]
    fn test_document() {
        let expected = Document {
            includes: vec![Include::from(Literal::from("another.thrift"))],
            ..Default::default()
        };
        assert_eq!(
            Document::parse("include 'another.thrift'").unwrap().1,
            expected
        );
    }
}
