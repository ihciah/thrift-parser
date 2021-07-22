use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, Literal, Separator};
use crate::Parser;

// FieldType       ::=  Identifier | BaseType | ContainerType
// BaseType        ::=  'bool' | 'byte' | 'i8' | 'i16' | 'i32' | 'i64' | 'double' | 'string' | 'binary'
// ContainerType   ::=  MapType | SetType | ListType
// MapType         ::=  'map' CppType? '<' FieldType ',' FieldType '>'
// SetType         ::=  'set' CppType? '<' FieldType '>'
// ListType        ::=  'list' '<' FieldType '>' CppType?
// CppType         ::=  'cpp_type' Literal
// Note: CppType is not fully supported in out impl.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType<'a> {
    Identifier(Identifier<'a>),
    Bool,
    Byte,
    I8,
    I16,
    I32,
    I64,
    Double,
    String,
    Binary,
    Map(Box<FieldType<'a>>, Box<FieldType<'a>>),
    Set(Box<FieldType<'a>>),
    List(Box<FieldType<'a>>),
}

impl<'a> FieldType<'a> {
    pub fn parse_base_type(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(tag("bool"), |_| Self::Bool),
            map(tag("byte"), |_| Self::Byte),
            map(tag("i8"), |_| Self::I8),
            map(tag("i16"), |_| Self::I16),
            map(tag("i32"), |_| Self::I32),
            map(tag("i64"), |_| Self::I64),
            map(tag("double"), |_| Self::Double),
            map(tag("string"), |_| Self::String),
            map(tag("binary"), |_| Self::Binary),
        ))(input)
    }

    pub fn parse_container_type(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(
                preceded(
                    tuple((
                        tag("map"),
                        opt(Separator::parse),
                        opt(terminated(CppType::parse, opt(Separator::parse))),
                    )),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        separated_pair(
                            FieldType::parse,
                            tuple((opt(Separator::parse), cchar(','), opt(Separator::parse))),
                            FieldType::parse,
                        ),
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                ),
                |(k, v)| Self::Map(Box::new(k), Box::new(v)),
            ),
            map(
                preceded(
                    tuple((
                        tag("set"),
                        opt(Separator::parse),
                        opt(terminated(CppType::parse, opt(Separator::parse))),
                    )),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        FieldType::parse,
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                ),
                |v| Self::Set(Box::new(v)),
            ),
            map(
                delimited(
                    pair(tag("list"), opt(Separator::parse)),
                    delimited(
                        pair(cchar('<'), opt(Separator::parse)),
                        FieldType::parse,
                        pair(opt(Separator::parse), cchar('>')),
                    ),
                    opt(pair(opt(Separator::parse), CppType::parse)),
                ),
                |v| Self::List(Box::new(v)),
            ),
            map(Identifier::parse, Self::Identifier),
        ))(input)
    }

    pub fn parse_identifier_type(input: &'a str) -> IResult<&'a str, Self> {
        map(Identifier::parse, Self::Identifier)(input)
    }
}

impl<'a> Parser<'a> for FieldType<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Self::parse_base_type,
            Self::parse_container_type,
            Self::parse_identifier_type,
        ))(input)
    }
}

// CppType         ::=  'cpp_type' Literal
#[derive(derive_newtype::NewType, Eq, PartialEq, Debug, Clone)]
pub struct CppType<'a>(Literal<'a>);

impl<'a> Parser<'a> for CppType<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            preceded(tag("cpp_type"), preceded(Separator::parse, Literal::parse)),
            Self,
        )(input)
    }
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    use super::*;

    #[test]
    fn test_cpp_type() {
        assert_list_eq_with_f(
            vec!["cpp_type \"MINI-LUST\"", "cpp_type 'ihciah'"],
            vec![Literal::from("MINI-LUST"), Literal::from("ihciah")],
            CppType::parse,
            CppType,
        );
    }

    #[test]
    fn test_field_type() {
        assert_list_eq_with_f(
            vec!["bool", "i16"],
            vec![FieldType::Bool, FieldType::I16],
            FieldType::parse,
            |x| x,
        );
        assert_eq!(
            FieldType::parse("map <bool, bool>").unwrap().1,
            FieldType::Map(Box::new(FieldType::Bool), Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("map<bool,bool>").unwrap().1,
            FieldType::Map(Box::new(FieldType::Bool), Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("set <bool>").unwrap().1,
            FieldType::Set(Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("set<bool>").unwrap().1,
            FieldType::Set(Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("list <bool>").unwrap().1,
            FieldType::List(Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("list<bool>").unwrap().1,
            FieldType::List(Box::new(FieldType::Bool))
        );
        assert_eq!(
            FieldType::parse("ihc_iah").unwrap().1,
            FieldType::Identifier(Identifier::from("ihc_iah"))
        );
    }
}
