use std::str::FromStr;

use nom::branch::alt;
use nom::character::complete::{char as cchar, digit0, digit1};
use nom::combinator::{map, map_res, opt, recognize};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, separated_pair, tuple};
use nom::IResult;

use crate::basic::{Identifier, ListSeparator, Literal, Separator};
use crate::Parser;

// ConstValue      ::=  IntConstant | DoubleConstant | Literal | Identifier | ConstList | ConstMap
#[derive(Debug, Clone, PartialEq)]
pub enum ConstValue<'a> {
    Identifier(Identifier<'a>),
    Literal(Literal<'a>),
    Double(DoubleConstant),
    Int(IntConstant),
    List(ConstList<'a>),
    Map(ConstMap<'a>),
}

impl<'a> Parser<'a> for ConstValue<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(Identifier::parse, ConstValue::Identifier),
            map(Literal::parse, ConstValue::Literal),
            map(DoubleConstant::parse2, ConstValue::Double),
            map(IntConstant::parse, ConstValue::Int),
            map(ConstList::parse, ConstValue::List),
            map(ConstMap::parse, ConstValue::Map),
        ))(input)
    }
}

// IntConstant     ::=  ('+' | '-')? Digit+
#[derive(derive_newtype::NewType, Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct IntConstant(i64);

impl<'a> Parser<'a> for IntConstant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map_res(
            recognize(tuple((opt(alt((cchar('-'), cchar('+')))), digit1))),
            |d_str| -> Result<Self, std::num::ParseIntError> {
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}

// DoubleConstant  ::=  ('+' | '-')? Digit* ('.' Digit+)? ( ('E' | 'e') IntConstant )?
#[derive(derive_newtype::NewType, Debug, Copy, Clone)]
pub struct DoubleConstant(f64);

impl<'a> Parser<'a> for DoubleConstant {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}
// Double except int: If the double is indeed a int, it will fail!
impl DoubleConstant {
    fn parse2(input: &str) -> IResult<&str, Self> {
        map_res(
            recognize(tuple((
                opt(alt((cchar('-'), cchar('+')))),
                digit0,
                opt(pair(cchar('.'), digit1)),
                opt(pair(alt((cchar('E'), cchar('e'))), IntConstant::parse)),
            ))),
            |d_str| -> Result<Self, std::num::ParseFloatError> {
                if !d_str.contains('.') && !d_str.contains('e') && !d_str.contains('E') {
                    return Err(f64::from_str("").unwrap_err());
                }
                let d = FromStr::from_str(d_str)?;
                Ok(Self(d))
            },
        )(input)
    }
}

impl PartialEq for DoubleConstant {
    fn eq(&self, other: &Self) -> bool {
        float_cmp::approx_eq!(f64, self.0, other.0)
    }
}

// ConstList       ::=  '[' (ConstValue ListSeparator?)* ']'
#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstList<'a>(Vec<ConstValue<'a>>);

impl<'a> Parser<'a> for ConstList<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(
                pair(cchar('['), opt(Separator::parse)),
                separated_list0(parse_list_separator, ConstValue::parse),
                pair(opt(Separator::parse), cchar(']')),
            ),
            Self,
        )(input)
    }
}

// ConstMap        ::=  '{' (ConstValue ':' ConstValue ListSeparator?)* '}'
#[derive(derive_newtype::NewType, PartialEq, Debug, Clone)]
pub struct ConstMap<'a>(Vec<(ConstValue<'a>, ConstValue<'a>)>);

impl<'a> Parser<'a> for ConstMap<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            delimited(
                pair(cchar('{'), opt(Separator::parse)),
                separated_list0(
                    parse_list_separator,
                    separated_pair(
                        ConstValue::parse,
                        delimited(opt(Separator::parse), cchar(':'), opt(Separator::parse)),
                        ConstValue::parse,
                    ),
                ),
                pair(opt(Separator::parse), cchar('}')),
            ),
            Self,
        )(input)
    }
}

// At least one Separator or ListSeparator
pub fn parse_list_separator(input: &str) -> IResult<&str, ()> {
    alt((
        map(
            tuple((
                Separator::parse,
                opt(ListSeparator::parse),
                opt(Separator::parse),
            )),
            |_| (),
        ),
        map(tuple((ListSeparator::parse, opt(Separator::parse))), |_| ()),
    ))(input)
}

#[cfg(test)]
mod test {
    use crate::utils::*;

    use super::*;

    #[test]
    fn test_int_constant() {
        assert_list_eq_with_f(
            vec!["123", "+123", "-123"],
            vec![123, 123, -123],
            IntConstant::parse,
            IntConstant,
        );
        assert_list_err_with_f(
            vec![
                "-+123",
                "+-123",
                "+",
                "-",
                "10000000000000000000000000000000000000000000000",
            ],
            IntConstant::parse,
        );
    }

    #[test]
    fn test_double_constant() {
        assert_list_eq_with_f(
            vec![
                "123.0",
                ".5",
                "-.5",
                "+123.2333333e10",
                "+123.2333333E100",
                "+123.1.THE.FOLLOWING",
                "1.1",
            ],
            vec![
                123.0,
                0.5,
                -0.5,
                123.2333333e10,
                123.2333333E100,
                123.1,
                1.1,
            ],
            DoubleConstant::parse,
            DoubleConstant,
        );
        assert_list_err_with_f(vec!["+-123.THE.FOLLOWING"], DoubleConstant::parse);
    }

    #[test]
    fn test_const_list() {
        assert_list_eq_with_f(
            vec![
                "[ 1,  3 ; 5  6/**/7 , ihciah 1.1]",
                "[6/**/7 ihciah 1.1   A ]",
                "[]",
            ],
            vec![
                vec![
                    ConstValue::Int(IntConstant(1)),
                    ConstValue::Int(IntConstant(3)),
                    ConstValue::Int(IntConstant(5)),
                    ConstValue::Int(IntConstant(6)),
                    ConstValue::Int(IntConstant(7)),
                    ConstValue::Identifier(Identifier::from("ihciah")),
                    ConstValue::Double(DoubleConstant(1.1)),
                ],
                vec![
                    ConstValue::Int(IntConstant(6)),
                    ConstValue::Int(IntConstant(7)),
                    ConstValue::Identifier(Identifier::from("ihciah")),
                    ConstValue::Double(DoubleConstant(1.1)),
                    ConstValue::Identifier(Identifier::from("A")),
                ],
                vec![],
            ],
            ConstList::parse,
            ConstList,
        );
        assert_list_err_with_f(vec!["[1,2,3A]"], ConstList::parse);
    }

    #[test]
    fn test_const_map() {
        assert_list_eq_with_f(
            vec!["{1:2, 3:4}", "{}"],
            vec![
                vec![
                    (
                        ConstValue::Int(IntConstant(1)),
                        ConstValue::Int(IntConstant(2)),
                    ),
                    (
                        ConstValue::Int(IntConstant(3)),
                        ConstValue::Int(IntConstant(4)),
                    ),
                ],
                vec![],
            ],
            ConstMap::parse,
            ConstMap,
        );
        assert_list_err_with_f(vec!["{1:34:5}"], ConstMap::parse);
    }
}
