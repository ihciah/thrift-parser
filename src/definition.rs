use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, tuple};
use nom::IResult;

use crate::basic::{Identifier, ListSeparator, Separator};
use crate::constant::{parse_list_separator, ConstValue, IntConstant};
use crate::field::Field;
use crate::functions::Function;
use crate::types::FieldType;
use crate::Parser;

// Const           ::=  'const' FieldType Identifier '=' ConstValue ListSeparator?
#[derive(Debug, Clone, PartialEq)]
pub struct Const<'a> {
    pub name: Identifier<'a>,
    pub type_: FieldType<'a>,
    pub value: ConstValue<'a>,
}

impl<'a> Parser<'a> for Const<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("const"),
                preceded(Separator::parse, FieldType::parse),
                preceded(Separator::parse, Identifier::parse),
                preceded(opt(Separator::parse), cchar('=')),
                preceded(opt(Separator::parse), ConstValue::parse),
                opt(pair(opt(Separator::parse), ListSeparator::parse)),
            )),
            |(_, type_, name, _, value, _)| Self { name, type_, value },
        )(input)
    }
}

// Typedef         ::=  'typedef' DefinitionType Identifier
// DefinitionType  ::=  BaseType | ContainerType
// BaseType        ::=  'bool' | 'byte' | 'i8' | 'i16' | 'i32' | 'i64' | 'double' | 'string' | 'binary'
// ContainerType   ::=  MapType | SetType | ListType
#[derive(Debug, Clone, PartialEq)]
pub struct Typedef<'a> {
    pub old: FieldType<'a>,
    pub alias: Identifier<'a>,
}

impl<'a> Parser<'a> for Typedef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("typedef"),
                preceded(
                    Separator::parse,
                    alt((FieldType::parse_base_type, FieldType::parse_container_type)),
                ),
                preceded(Separator::parse, Identifier::parse),
            )),
            |(_, old, alias)| Self { old, alias },
        )(input)
    }
}

// Enum            ::=  'enum' Identifier '{' (Identifier ('=' IntConstant)? ListSeparator?)* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct Enum<'a> {
    pub name: Identifier<'a>,
    pub children: Vec<EnumValue<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue<'a> {
    pub name: Identifier<'a>,
    pub value: Option<IntConstant>,
}

impl<'a> Parser<'a> for Enum<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                tag("enum"),
                preceded(Separator::parse, Identifier::parse),
                tuple((opt(Separator::parse), cchar('{'), opt(Separator::parse))),
                separated_list0(parse_list_separator, EnumValue::parse),
                preceded(opt(Separator::parse), cchar('}')),
            )),
            |(_, name, _, children, _)| Self { name, children },
        )(input)
    }
}

impl<'a> Parser<'a> for EnumValue<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                Identifier::parse,
                opt(map(
                    tuple((
                        opt(Separator::parse),
                        cchar('='),
                        opt(Separator::parse),
                        IntConstant::parse,
                    )),
                    |(_, _, _, i)| (i),
                )),
            )),
            |(name, value)| Self { name, value },
        )(input)
    }
}

// Struct          ::=  'struct' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct Struct<'a> {
    pub name: Identifier<'a>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Parser<'a> for Struct<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                pair(tag("struct"), Separator::parse),
                Identifier::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, Field::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(_, name, _, fields, _)| Self { name, fields },
        )(input)
    }
}

// Union          ::=  'union' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct Union<'a> {
    pub name: Identifier<'a>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Parser<'a> for Union<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                pair(tag("union"), Separator::parse),
                Identifier::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, Field::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(_, name, _, fields, _)| Self { name, fields },
        )(input)
    }
}

// Exception       ::=  'exception' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct Exception<'a> {
    pub name: Identifier<'a>,
    pub fields: Vec<Field<'a>>,
}

impl<'a> Parser<'a> for Exception<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                pair(tag("exception"), Separator::parse),
                Identifier::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, Field::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(_, name, _, fields, _)| Self { name, fields },
        )(input)
    }
}

// Service         ::=  'service' Identifier ( 'extends' Identifier )? '{' Function* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct Service<'a> {
    pub name: Identifier<'a>,
    pub extension: Option<Identifier<'a>>,
    pub functions: Vec<Function<'a>>,
}

impl<'a> Parser<'a> for Service<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                delimited(
                    pair(tag("service"), Separator::parse),
                    Identifier::parse,
                    opt(Separator::parse),
                ),
                opt(map(
                    tuple((
                        tag("extends"),
                        Separator::parse,
                        Identifier::parse,
                        opt(Separator::parse),
                    )),
                    |(_, _, ext, _)| ext,
                )),
                delimited(
                    pair(cchar('{'), opt(Separator::parse)),
                    separated_list0(Separator::parse, Function::parse),
                    pair(opt(Separator::parse), cchar('}')),
                ),
            )),
            |(name, extension, functions)| Self {
                name,
                extension,
                functions,
            },
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::basic::Literal;

    #[test]
    fn test_const() {
        assert_eq!(
            Const::parse("const bool is_rust_easy = 'yes!';").unwrap().1,
            Const {
                name: Identifier::from("is_rust_easy"),
                type_: FieldType::Bool,
                value: ConstValue::Literal(Literal::from("yes!"))
            }
        );
    }

    #[test]
    fn test_typedef() {
        assert_eq!(
            Typedef::parse("typedef i32 MyI32").unwrap().1,
            Typedef {
                old: FieldType::I32,
                alias: Identifier::from("MyI32")
            }
        );
    }

    #[test]
    fn test_enum() {
        let expected = Enum {
            name: Identifier::from("PL"),
            children: vec![
                EnumValue {
                    name: Identifier::from("Rust"),
                    value: None,
                },
                EnumValue {
                    name: Identifier::from("Go"),
                    value: Some(IntConstant::from(2)),
                },
                EnumValue {
                    name: Identifier::from("Cpp"),
                    value: Some(IntConstant::from(3)),
                },
            ],
        };
        assert_eq!(
            Enum::parse("enum PL { Rust Go=2 , Cpp = 3 }").unwrap().1,
            expected
        );
        assert_eq!(Enum::parse("enum PL{Rust Go=2,Cpp=3}").unwrap().1, expected);
    }

    #[test]
    fn test_struct() {
        let expected = Struct {
            name: Identifier::from("user"),
            fields: vec![
                Field {
                    id: Some(IntConstant::from(1)),
                    required: Some(false),
                    type_: FieldType::String,
                    name: Identifier::from("name"),
                    default: None,
                },
                Field {
                    id: Some(IntConstant::from(2)),
                    required: None,
                    type_: FieldType::I32,
                    name: Identifier::from("age"),
                    default: Some(ConstValue::Int(IntConstant::from(18))),
                },
            ],
        };
        assert_eq!(
            Struct::parse("struct user{1:optional string name; 2:i32 age=18}")
                .unwrap()
                .1,
            expected
        );
        assert_eq!(
            Struct::parse("struct user { 1 : optional string name ; 2 : i32 age = 18 }")
                .unwrap()
                .1,
            expected
        );
    }

    #[test]
    fn test_service() {
        let function = Function {
            oneway: false,
            returns: Some(FieldType::String),
            name: Identifier::from("GetUser"),
            parameters: vec![Field {
                id: None,
                required: Some(true),
                type_: FieldType::String,
                name: Identifier::from("name"),
                default: None,
            }],
            exceptions: None,
        };
        let expected = Service {
            name: Identifier::from("DemoService"),
            extension: Some(Identifier::from("BaseService")),
            functions: vec![function.clone(), function],
        };
        assert_eq!(
            Service::parse(
                "service DemoService extends BaseService { \
         string GetUser(required string name),
         string GetUser(required string name) }"
            )
            .unwrap()
            .1,
            expected
        );
    }
}
