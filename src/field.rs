use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

use crate::basic::{Identifier, ListSeparator, Separator};
use crate::constant::{ConstValue, IntConstant};
use crate::types::FieldType;
use crate::Parser;

// Field           ::=  FieldID? FieldReq? FieldType Identifier ('=' ConstValue)? ListSeparator?
// FieldID         ::=  IntConstant ':'
// FieldReq        ::=  'required' | 'optional'
// Note: XsdFieldOptions is not supported in out impl and strongly discouraged in official docs.
#[derive(Debug, Clone, PartialEq)]
pub struct Field<'a> {
    pub id: Option<IntConstant>,
    pub required: Option<bool>,
    pub type_: FieldType<'a>,
    pub name: Identifier<'a>,
    pub default: Option<ConstValue<'a>>,
}

impl<'a> Parser<'a> for Field<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    IntConstant::parse,
                    delimited(opt(Separator::parse), cchar(':'), opt(Separator::parse)),
                )),
                opt(terminated(
                    alt((
                        map(tag("required"), |_| true),
                        map(tag("optional"), |_| false),
                    )),
                    Separator::parse,
                )),
                terminated(FieldType::parse, Separator::parse),
                terminated(Identifier::parse, opt(Separator::parse)),
                opt(map(
                    tuple((cchar('='), opt(Separator::parse), ConstValue::parse)),
                    |(_, _, cv)| cv,
                )),
                opt(Separator::parse),
                opt(ListSeparator::parse),
            )),
            |(id, required, type_, name, default, _, _)| Self {
                id,
                required,
                type_,
                name,
                default,
            },
        )(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::basic::Literal;

    #[test]
    fn test_field() {
        let expected = Field {
            id: None,
            required: Some(true),
            type_: FieldType::String,
            name: Identifier::from("name"),
            default: Some(ConstValue::Literal(Literal::from("ihciah")))
        };
        assert_eq!(Field::parse("required  string  name  =  'ihciah'").unwrap().1, expected);
        assert_eq!(Field::parse("required string name='ihciah';").unwrap().1, expected);

        let expected = Field {
            id: Some(IntConstant::from(3)),
            required: Some(true),
            type_: FieldType::String,
            name: Identifier::from("name"),
            default: Some(ConstValue::Literal(Literal::from("ihciah")))
        };
        assert_eq!(Field::parse("3 : required  string  name  =  'ihciah'").unwrap().1, expected);
        assert_eq!(Field::parse("3:required string name='ihciah';").unwrap().1, expected);
    }
}