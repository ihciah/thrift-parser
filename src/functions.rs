use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, ListSeparator, Separator};
use crate::field::Field;
use crate::types::FieldType;
use crate::Parser;

// Function        ::=  'oneway'? FunctionType Identifier '(' Field* ')' Throws? ListSeparator?
// FunctionType    ::=  FieldType | 'void'
// Throws          ::=  'throws' '(' Field* ')'
#[derive(Debug, Clone, PartialEq)]
pub struct Function<'a> {
    pub oneway: bool,
    // returns None means void
    pub returns: Option<FieldType<'a>>,
    pub name: Identifier<'a>,
    pub parameters: Vec<Field<'a>>,
    pub exceptions: Option<Vec<Field<'a>>>,
}

impl<'a> Parser<'a> for Function<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                map(opt(terminated(tag("oneway"), Separator::parse)), |x| {
                    x.is_some()
                }),
                terminated(
                    alt((
                        map(tag("void"), |_| None),
                        map(FieldType::parse, Some),
                    )),
                    Separator::parse,
                ),
                terminated(Identifier::parse, opt(Separator::parse)),
                terminated(
                    delimited(
                        cchar('('),
                        separated_list0(Separator::parse, Field::parse),
                        cchar(')'),
                    ),
                    opt(Separator::parse),
                ),
                opt(preceded(
                    pair(tag("throws"), Separator::parse),
                    delimited(
                        cchar('('),
                        separated_list0(Separator::parse, Field::parse),
                        cchar(')'),
                    ),
                )),
                opt(pair(opt(Separator::parse), ListSeparator::parse)),
            )),
            |(oneway, returns, name, parameters, exceptions, _)| Self {
                oneway,
                returns,
                name,
                parameters,
                exceptions,
            },
        )(input)
    }
}

#[cfg(test)]
mod test {
    use crate::basic::Literal;
    use crate::constant::{ConstValue, IntConstant};

    use super::*;

    #[test]
    fn test_function() {
        let expected = Function {
            oneway: false,
            returns: Some(FieldType::String),
            name: Identifier::from("GetUser"),
            parameters: vec![Field {
                id: None,
                required: Some(true),
                type_: FieldType::String,
                name: Identifier::from("name"),
                default: Some(ConstValue::Literal(Literal::from("ihciah"))),
            }],
            exceptions: None,
        };
        assert_eq!(
            Function::parse("string GetUser(required string name='ihciah')")
                .unwrap()
                .1,
            expected
        );

        let expected = Function {
            oneway: true,
            returns: None,
            name: Identifier::from("DeleteUser"),
            parameters: vec![Field {
                id: Some(IntConstant::from(10086)),
                required: Some(false),
                type_: FieldType::I32,
                name: Identifier::from("age"),
                default: None,
            }],
            exceptions: None,
        };
        assert_eq!(
            Function::parse("oneway void DeleteUser(10086:optional i32 age)")
                .unwrap()
                .1,
            expected
        );
    }
}
