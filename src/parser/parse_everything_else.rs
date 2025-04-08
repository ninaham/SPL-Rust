use nom::{IResult, Parser, branch::alt};

use crate::{
    absyn::{
        absyn::TypeExpression, array_type_expression::ArrayTypeExpression,
        type_definition::TypeDefinition, variable_definition::VariableDefinition,
    },
    parser::token_parser::{colon, eq, ident, intlit, lbrack, of, rbrack, semic, r#type, var},
};

use super::{token_parser::array, tokens::Tokens};

pub fn variable_definition(input: &str) -> IResult<&str, VariableDefinition> {
    let (rem, _) = var(input)?;
    let (rem, m1) = ident(rem)?;
    let name = match m1 {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = colon(rem)?;
    let (rem, te) = type_expression(rem)?;
    let (rem, _) = semic(rem)?;
    Ok((
        rem,
        VariableDefinition {
            name,
            type_expression: te,
        },
    ))
}

pub fn array_type_expression(input: &str) -> IResult<&str, TypeExpression> {
    let (rem, _) = array(input)?;
    let (rem, _) = lbrack(rem)?;
    let (rem, intlit) = intlit(rem)?;
    let intlit = match intlit {
        Tokens::Intlit(i) => i,
        _ => panic!(),
    };
    let (rem, _) = rbrack(rem)?;
    let (rem, _) = of(rem)?;
    let (rem, te) = type_expression(rem)?;
    let ate = ArrayTypeExpression {
        array_size: intlit as usize,
        base_type: te,
    };
    Ok((rem, TypeExpression::ArrayTypeExpression(Box::new(ate))))
}

pub fn named_type_expression(input: &str) -> IResult<&str, TypeExpression> {
    let (rem, name) = ident(input)?;
    let name = match name {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };

    Ok((rem, TypeExpression::NamedTypeExpression(name)))
}

pub fn type_expression(input: &str) -> IResult<&str, TypeExpression> {
    alt([named_type_expression, array_type_expression]).parse(input)
}

pub fn type_definition(input: &str) -> IResult<&str, TypeDefinition> {
    let (rem, _) = r#type(input)?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = eq(rem)?;
    let (rem, te) = type_expression(rem)?;
    let (rem, _) = semic(rem)?;
    let type_def = TypeDefinition {
        name,
        type_expression: te,
    };

    Ok((rem, type_def))
}
