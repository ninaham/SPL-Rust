use std::collections::LinkedList;

use nom::{branch::alt, character::complete::{alpha1, char, space0}, combinator::map, multi::fold_many0, sequence::{delimited, preceded}, IResult, Parser};

use crate::{
    absyn::{
        absyn::{Definition, Expression, Node, Statement, TypeExpression, Variable},
        array_access::ArrayAccess,
        array_type_expression::ArrayTypeExpression,
        assign_statement::AssignStatement,
        binary_expression::{BinaryExpression, Operator},
        call_statement::CallStatement,
        if_statement::IfStatement,
        parameter_definition::ParameterDefinition,
        procedure_definition::ProcedureDefinition,
        type_definition::TypeDefinition,
        unary_expression::UnaryExpression,
        variable_definition::VariableDefinition,
        while_statement::WhileStatement,
    },
    parser::token_parser::{
        asgn, colon, comma, eq, ident, intlit, lbrack, lcurl, lparen, of, proc, rbrack, rcurl,
        rparen, semic, r#type, var, r#while, whitespace1,
    },
};

use super::{
    token_parser::{
        array, r#else, ge, gt, r#if, le, lt, minus, ne, plus, r#ref, slash, star, whitespace0,
    },
    tokens::Tokens,
};

pub fn parse(input: &str) -> Node {
    let (rem, n) = match program(input) {
        Ok(x) => x,
        Result::Err(e) => panic!("Parser Error: {:?}", e),
    };
    if !rem.is_empty() {
        panic!("input not empty, remaining: {}", rem)
    }

    n
}

fn program(input: &str) -> IResult<&str, Node> {
    println!("Parsing program");
    let (rem, gd) = global_definition_list(input)?;
    let gd = gd.into_iter().map(Box::new).collect();
    Ok((rem, Node::Program(gd)))
}

fn global_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    println!("Parsing global definition list");
    alt([non_empty_definition_list, empty_definition_list]).parse(input)
}

fn empty_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    println!("Parsing empty definition list");
    Ok((input, LinkedList::from([])))
}

fn non_empty_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    println!("Parsing non-empty definition list");
    let (rem, _) = whitespace0(input)?;
    let (rem, dhead) = global_definition(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, mut dtail) = global_definition_list(rem)?;
    dtail.push_front(dhead);
    Ok((rem, dtail))
}

fn global_definition(input: &str) -> IResult<&str, Definition> {
    println!("Parsing global definition");
    alt([procedure_definition, type_definition]).parse(input)
}

fn variable_definition(input: &str) -> IResult<&str, VariableDefinition> {
    let (rem, _) = var(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, m1) = ident(rem)?;
    let name = match m1 {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = colon(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, te) = type_expression(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = semic(rem)?;
    Ok((
        rem,
        VariableDefinition {
            name,
            type_expression: te,
        },
    ))
}

fn array_type_expression(input: &str) -> IResult<&str, TypeExpression> {
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

fn named_type_expression(input: &str) -> IResult<&str, TypeExpression> {
    println!("Parsing named type expression");
    let (rem, name) = ident(input)?;
    let name = match name {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };

    Ok((rem, TypeExpression::NamedTypeExpression(name)))
}

fn type_expression(input: &str) -> IResult<&str, TypeExpression> {
    println!("Parsing type expression");
    alt([named_type_expression, array_type_expression]).parse(input)
}

fn type_definition(input: &str) -> IResult<&str, Definition> {
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

    Ok((rem, Definition::TypeDefinition(Box::new(type_def))))
}

fn procedure_definition(input: &str) -> IResult<&str, Definition> {
    println!("Parsing procedure definition");
    let (rem, _) = proc(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = lparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, pl) = parameter_list(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = lcurl(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, vl) = variable_list(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rcurl(rem)?;
    let pd = ProcedureDefinition {
        name,
        parameters: pl,
        body: stl,
        variales: vl,
    };

    Ok((rem, Definition::ProcedureDefinition(Box::new(pd))))
}

fn parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    println!("Parsing parameter list");
    alt([non_empty_parameter_list, empty_parameter_list]).parse(input)
}

fn empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    println!("Parsing empty parameter list");
    Ok((input, LinkedList::from([])))
}

fn non_empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    println!("Parsing non-empty parameter list");
    alt([more_than_one_parameter, parameter]).parse(input)
}

fn more_than_one_parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    println!("Parsing more than one parameter");
    let (rem, mut phead) = parameter(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = comma(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, mut ptail) = non_empty_parameter_list(rem)?;
    ptail.push_front(phead.pop_back().unwrap());
    Ok((rem, ptail))
}

fn non_ref_parameter(input: &str) -> IResult<&str, ParameterDefinition> {
    println!("Parsing non-ref parameter");
    let (rem, ident) = ident(input)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = colon(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, te) = type_expression(rem)?;
    let pd = ParameterDefinition {
        name,
        type_expression: te,
        is_reference: false,
    };

    Ok((rem, pd))
}

fn parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    println!("Parsing parameter");
    let res = alt([non_ref_parameter, ref_parameter]).parse(input)?;
    Ok((res.0, LinkedList::from([res.1])))
}

fn ref_parameter(input: &str) -> IResult<&str, ParameterDefinition> {
    println!("Parsing ref parameter");
    let (rem, _) = r#ref(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, ident) = ident(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = colon(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, te) = type_expression(rem)?;
    let pd = ParameterDefinition {
        name,
        type_expression: te,
        is_reference: true,
    };

    Ok((rem, pd))
}

fn variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    alt([non_empty_variable_list, empty_variable_list]).parse(input)
}

fn empty_variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_variable_list(input: &str) -> IResult<&str, LinkedList<VariableDefinition>> {
    let (rem, vd) = variable_definition(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, mut vl) = variable_list(rem)?;
    vl.push_front(vd);
    Ok((rem, vl))
}

fn statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    println!("Parsing statement list");
    alt([non_empty_statement_list, empty_statement_list]).parse(input)
}

fn empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    println!("Parsing non-empty statement list");
    let (rem, st) = statement(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, mut stl) = statement_list(rem)?;
    stl.push_front(st);
    Ok((rem, stl))
}

fn statement(input: &str) -> IResult<&str, Statement> {
    println!("Parsing statement");
    alt([
        empty_statement,
        if_statement,
        assign_statement,
        while_statement,
        compound_statement,
        call_statement,
    ])
    .parse(input)
}

fn empty_statement(input: &str) -> IResult<&str, Statement> {
    println!("Parsing empty statement");
    let (rem, _) = semic(input)?;
    println!("Empty statement: {}", rem);
    Ok((rem, Statement::EmptyStatement))
}

fn if_statement(input: &str) -> IResult<&str, Statement> {
    println!("Parsing if statement");
    alt([if_statement_with_else, if_statement_without_else]).parse(input)
}

fn if_statement_without_else(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = r#if(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, _) = lparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, st) = statement(rem)?;
    let if_stmt = IfStatement {
        condition: ex,
        then_branch: st,
        else_branch: None,
    };
    Ok((rem, Statement::IfStatement(Box::new(if_stmt))))
}

fn if_statement_with_else(input: &str) -> IResult<&str, Statement> {
    println!("Parsing if statement with else {}", input);
    let (rem, _) = r#if(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, _) = lparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, then_part) = statement(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = r#else(rem)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, else_part) = statement(rem)?;
    let if_stmt = IfStatement {
        condition: ex,
        then_branch: then_part,
        else_branch: Some(else_part),
    };

    Ok((rem, Statement::IfStatement(Box::new(if_stmt))))
}

fn while_statement(input: &str) -> IResult<&str, Statement> {
    println!("Parsing while statement");
    let (rem, _) = r#while(input)?;
    let (rem, _) = whitespace1(rem)?;
    let (rem, _) = lparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, cond) = expression(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, st) = statement(rem)?;
    let while_stmt = WhileStatement {
        condition: cond,
        body: st,
    };
    Ok((rem, Statement::WhileStatement(Box::new(while_stmt))))
}

fn compound_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = lcurl(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rcurl(rem)?;
    let stl = stl.into_iter().map(Box::new).collect();
    Ok((rem, Statement::CompoundStatement(stl)))
}

fn assign_statement(input: &str) -> IResult<&str, Statement> {
    println!("Parsing assign statement");
    let (rem, var) = variable(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = asgn(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, exp) = expression(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = semic(rem)?;

    let asgn_statement = AssignStatement {
        target: var,
        value: exp,
    };
    Ok((rem, Statement::AssignStatement(Box::new(asgn_statement))))
}

fn call_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, ident) = ident(input)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = lparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, arguments) = argument_list(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = rparen(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = semic(rem)?;
    let call_statement = CallStatement { name, arguments };

    Ok((rem, Statement::CallStatement(Box::new(call_statement))))
}

fn argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    println!("Parsing argument list");
    alt([non_empty_argument_list, empty_argument_list]).parse(input)
}

fn empty_argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    println!("Parsing non-empty argument list");
    alt([more_than_one_argument, expression_head]).parse(input)
}

fn expression_head(input: &str) -> IResult<&str, LinkedList<Expression>> {
    let (rem, exp) = expression(input)?;
    Ok((rem, LinkedList::from([exp])))
}

fn more_than_one_argument(input: &str) -> IResult<&str, LinkedList<Expression>> {
    println!("Parsing more than one argument");
    let (rem, mut ehead) = expression_head(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = comma(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, mut etail) = non_empty_argument_list(rem)?;
    etail.push_front(ehead.pop_back().unwrap());
    Ok((rem, etail))
}



// Parses a single [index] access
fn single_array_access(input: &str) -> IResult<&str, Expression> {
    delimited(
        preceded(space0, char('[')),
        preceded(space0, expression),
        preceded(space0, char(']')),
    ).parse(input)
}

// Parses a variable with 0 or more array accesses: myArray[1][2]
fn variable(input: &str) -> IResult<&str, Variable> {
    let (input, base) = named_var(input)?;

    fold_many0(
        single_array_access,
        move || base.clone(),
        |acc, idx| {
            Variable::ArrayAccess(Box::new(ArrayAccess {
                array: acc,
                index: idx,
            }))
        },
    ).parse(input)
}

fn named_var(input: &str) -> IResult<&str, Variable> {
    let (rem, _) = whitespace0(input)?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };

    Ok((rem, Variable::NamedVariable(name)))
}

fn expression(input: &str) -> IResult<&str, Expression> {
    println!("Parsing expression");
    expression0(input)
}

fn expression0(input: &str) -> IResult<&str, Expression> {
    println!("Parsing expression0");
    alt([
        le_expression,
        lt_expression,
        gt_expression,
        ge_expression,
        expression1,
    ])
    .parse(input)
}

fn le_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression1(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = le(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression0(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Lse,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn lt_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression1(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = lt(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression0(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Lst,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn gt_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression1(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = gt(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression0(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Grt,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn ge_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression1(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = ge(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression0(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Gre,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn expression1(input: &str) -> IResult<&str, Expression> {
    alt([eq_expression, ne_expression, expression2]).parse(input)
}

fn eq_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression2(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = eq(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression1(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Equ,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn ne_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression2(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = ne(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression1(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Neq,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn expression2(input: &str) -> IResult<&str, Expression> {
    alt([plus_expression, minus_expression, expression3]).parse(input)
}

fn expression3(input: &str) -> IResult<&str, Expression> {
    alt([star_expression, slash_expression, expression4]).parse(input)
}

fn star_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression4(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = star(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression3(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Mul,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn slash_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression4(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = slash(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression3(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Div,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn plus_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression3(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = plus(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression2(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Add,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn minus_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, left) = expression3(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, _) = minus(rem)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, right) = expression2(rem)?;
    let binary_expression = BinaryExpression {
        operator: Operator::Sub,
        left,
        right,
    };
    Ok((
        rem,
        Expression::BinaryExpression(Box::new(binary_expression)),
    ))
}

fn expression4(input: &str) -> IResult<&str, Expression> {
    alt([unary_expression, expression5]).parse(input)
}

fn unary_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, _) = minus(input)?;
    let (rem, _) = whitespace0(rem)?;
    let (rem, exp) = expression4(rem)?;
    let unary_expression = UnaryExpression {
        operator: crate::absyn::unary_expression::UnaryOperator::Minus,
        operand: exp,
    };

    Ok((rem, Expression::UnaryExpression(Box::new(unary_expression))))
}

fn intlit_exp(input: &str) -> IResult<&str, Expression> {
    let (rem, i) = intlit(input)?;
    let i = match i {
        Tokens::Intlit(i) => i,
        _ => panic!(),
    };

    Ok((rem, Expression::IntLiteral(i)))
}

fn variable_exp(input: &str) -> IResult<&str, Expression> {
    let (rem, i) = variable(input)?;
    Ok((rem, Expression::VariableExpression(Box::new(i))))
}

fn parentheses_exp(input: &str) -> IResult<&str, Expression> {
    let (rem, _) = lparen(input)?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = rparen(rem)?;
    Ok((rem, ex))
}

fn expression5(input: &str) -> IResult<&str, Expression> {
    alt([intlit_exp, variable_exp, parentheses_exp]).parse(input)
}
