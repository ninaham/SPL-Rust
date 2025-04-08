use std::collections::LinkedList;

use nom::{IResult, Parser, branch::alt};

use crate::{
    absyn::{
        absyn::{Expression, Statement, TypeExpression, Variable},
        array_type_expression::ArrayTypeExpression,
        assign_statement::AssignStatement,
        if_statement::IfStatement,
        parameter_definition::ParameterDefinition,
        procedure_definition::ProcedureDefinition,
        type_definition::TypeDefinition,
        variable_definition::VariableDefinition,
        while_statement::WhileStatement,
    },
    parser::token_parser::{
        asgn, colon, comma, eq, ident, intlit, lbrack, lcurl, lparen, of, proc, rbrack, rcurl,
        rparen, semic, r#type, var, r#while,
    },
};

use super::{
    token_parser::{array, r#else, r#if, r#ref},
    tokens::Tokens,
};

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

pub fn procedure_definition(input: &str) -> IResult<&str, ProcedureDefinition> {
    let (rem, _) = proc(input)?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = lparen(rem)?;
    let (rem, pl) = parameter_list(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = lcurl(rem)?;
    let (rem, vl) = variable_list(rem)?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = rcurl(rem)?;
    let pd = ProcedureDefinition {
        name,
        parameters: pl,
        body: stl,
        variales: vl,
    };

    Ok((rem, pd))
}

pub fn parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    alt([non_empty_parameter_list, empty_parameter_list]).parse(input)
}

pub fn empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    alt([parameter, more_than_one_parameter]).parse(input)
}

fn more_than_one_parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    let (rem, mut phead) = parameter(input)?;
    let (rem, _) = comma(rem)?;
    let (rem, mut ptail) = non_empty_parameter_list(rem)?;
    ptail.push_front(phead.pop_back().unwrap());
    Ok((rem, ptail))
}

fn non_ref_parameter(input: &str) -> IResult<&str, ParameterDefinition> {
    let (rem, ident) = ident(input)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = colon(rem)?;
    let (rem, te) = type_expression(rem)?;
    let pd = ParameterDefinition {
        name,
        type_expression: te,
        is_reference: false,
    };

    Ok((rem, pd))
}

pub fn parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    let res = alt([non_ref_parameter, ref_parameter]).parse(input)?;
    Ok((res.0, LinkedList::from([res.1])))
}

fn ref_parameter(input: &str) -> IResult<&str, ParameterDefinition> {
    let (rem, _) = r#ref(input)?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = colon(rem)?;
    let (rem, te) = type_expression(rem)?;
    let pd = ParameterDefinition {
        name,
        type_expression: te,
        is_reference: true,
    };

    Ok((rem, pd))
}

pub fn variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    alt([empty_variable_list, non_empty_variable_list]).parse(input)
}

pub fn empty_variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    Ok((input, LinkedList::from([])))
}

pub fn non_empty_variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    let (rem, vd) = variable_definition(input)?;
    let (rem, mut vl) = variable_list(rem)?;
    vl.push_front(vd);
    Ok((rem, vl))
}

pub fn statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    alt([empty_statement_list, non_empty_statement_list]).parse(input)
}

pub fn empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    Ok((input, LinkedList::from([])))
}

pub fn non_empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    let (rem, st) = statement(input)?;
    let (rem, mut stl) = statement_list(rem)?;
    stl.push_front(st);
    Ok((rem, stl))
}

pub fn statement(input: &str) -> IResult<&str, Statement> {
    alt([
        empty_statement,
        if_statement,
        while_statement,
        compound_statement,
    ])
    .parse(input)
}

pub fn empty_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = semic(input)?;
    Ok((rem, Statement::EmptyStatement))
}

pub fn if_statement(input: &str) -> IResult<&str, Statement> {
    alt([if_statement_with_else, if_statement_without_else]).parse(input)
}

pub fn if_statement_without_else(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = r#if(input)?;
    let (rem, _) = lparen(rem)?;
    let (rem, ex) = expression(rem)?;
    let (rem, st) = statement(rem)?;
    let if_stmt = IfStatement {
        condition: ex,
        then_branch: st,
        else_branch: None,
    };
    Ok((rem, Statement::IfStatement(Box::new(if_stmt))))
}

pub fn if_statement_with_else(input: &str) -> IResult<&str, Statement> {
    let (rem, st) = if_statement_without_else(input)?;
    let (rem, _) = r#else(rem)?;
    let (rem, else_part) = statement(rem)?;
    let mut is = match st {
        Statement::IfStatement(is) => is,
        _ => panic!(),
    };

    is.else_branch = Some(else_part);

    Ok((rem, Statement::IfStatement(is)))
}

pub fn expression(input: &str) -> IResult<&str, Expression> {
    todo!()
}

pub fn while_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = r#while(input)?;
    let (rem, _) = lparen(rem)?;
    let (rem, cond) = expression(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, st) = statement(rem)?;
    let while_stmt = WhileStatement {
        condition: cond,
        body: st,
    };
    Ok((rem, Statement::WhileStatement(Box::new(while_stmt))))
}

pub fn compound_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = lcurl(input)?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = rcurl(rem)?;
    let stl = stl.into_iter().map(Box::new).collect();
    Ok((rem, Statement::CompoundStatement(stl)))
}

pub fn assign_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, var) = variable(input)?;
    let (rem, _) = asgn(rem)?;
    let (rem, exp) = expression(rem)?;
    let (rem, _) = semic(rem)?;

    let asgn_statement = AssignStatement {
        target: var,
        value: exp,
    };
    Ok((rem, Statement::AssignStatement(Box::new(asgn_statement))))
}

pub fn call_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, ident) = ident(input)?;
    let (rem, _) = lparen(rem)?;
    //let (rem, arguments)
    todo!()
}

pub fn variable(input: &str) -> IResult<&str, Variable> {
    todo!()
}
