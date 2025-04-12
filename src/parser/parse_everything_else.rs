use std::collections::LinkedList;

use nom::{
    IResult, Parser,
    branch::alt,
    character::complete::{char, space0},
    multi::fold_many0,
    sequence::{delimited, preceded},
};

use crate::{
    absyn::{
        absyn::{Definition, Expression, Program, Statement, TypeExpression, Variable},
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
        ident, intlit
    },
};

use super::{token_parser::{eq, ge, gt, le, lt, minus, ne, parse_tag, plus, slash, star}, tokens::Tokens};

pub fn parse(input: &str) -> Program {
    let (rem, n) = match program(input) {
        Ok(x) => x,
        Result::Err(e) => panic!("Parser Error: {:?}", e),
    };
    if !rem.is_empty() {
        panic!("input not empty")
    }

    n
}

fn program(input: &str) -> IResult<&str, Program> {
    let (rem, gd) = global_definition_list(input)?;
    let gd = gd.into_iter().map(Box::new).collect();
    Ok((rem, Program { definitions: gd }))
}

fn global_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    alt([non_empty_definition_list, empty_definition_list]).parse(input)
}

fn empty_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_definition_list(input: &str) -> IResult<&str, LinkedList<Definition>> {
    let (rem, dhead) = global_definition(input)?;
    let (rem, mut dtail) = global_definition_list(rem)?;
    dtail.push_front(dhead);
    Ok((rem, dtail))
}

fn global_definition(input: &str) -> IResult<&str, Definition> {
    alt([procedure_definition, type_definition]).parse(input)
}

fn variable_definition(input: &str) -> IResult<&str, VariableDefinition> {
    let (rem, _) = parse_tag(input, "var")?;
    let (rem, m1) = ident(rem)?;
    let name = match m1 {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = parse_tag(rem, ":")?;
    let (rem, te) = type_expression(rem)?;
    let (rem, _) = parse_tag(rem, ";")?;
    Ok((
        rem,
        VariableDefinition {
            name,
            type_expression: te,
        },
    ))
}

fn array_type_expression(input: &str) -> IResult<&str, TypeExpression> {
    let (rem, _) = parse_tag(input, "array")?;
    let (rem, _) = parse_tag(rem, "[")?;
    let (rem, intlit) = intlit(rem)?;

    let intlit = match intlit {
        Tokens::Intlit(i) => i,
        _ => panic!(),
    };
    let (rem, _) = parse_tag(rem, "]")?;
    let (rem, _) = parse_tag(rem, "of")?;
    let (rem, te) = type_expression(rem)?;
    let ate = ArrayTypeExpression {
        array_size: intlit as usize,
        base_type: te,
    };
    Ok((rem, TypeExpression::ArrayTypeExpression(Box::new(ate))))
}

fn named_type_expression(input: &str) -> IResult<&str, TypeExpression> {
    let (rem, name) = ident(input)?;
    let name = match name {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };

    Ok((rem, TypeExpression::NamedTypeExpression(name)))
}

fn type_expression(input: &str) -> IResult<&str, TypeExpression> {
    alt([array_type_expression, named_type_expression]).parse(input)
}

fn type_definition(input: &str) -> IResult<&str, Definition> {
    let (rem, _) = parse_tag(input, "type")?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = parse_tag(rem, "=")?;
    let (rem, te) = type_expression(rem)?;
    let (rem, _) = parse_tag(rem, ";")?;
    let type_def = TypeDefinition {
        name,
        type_expression: te,
    };

    Ok((rem, Definition::TypeDefinition(Box::new(type_def))))
}

fn procedure_definition(input: &str) -> IResult<&str, Definition> {
    let (rem, _) = parse_tag(input, "proc")?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = parse_tag(rem, "(")?;
    let (rem, pl) = parameter_list(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    let (rem, _) = parse_tag(rem, "{")?;
    let (rem, vl) = variable_list(rem)?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = parse_tag(rem, "}")?;
    let pd = ProcedureDefinition {
        name,
        parameters: pl,
        body: stl,
        variales: vl,
    };

    Ok((rem, Definition::ProcedureDefinition(Box::new(pd))))
}

fn parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    alt([non_empty_parameter_list, empty_parameter_list]).parse(input)
}

fn empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_parameter_list(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    alt([more_than_one_parameter, parameter]).parse(input)
}

fn more_than_one_parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    let (rem, mut phead) = parameter(input)?;
    let (rem, _) = parse_tag(rem, ",")?;
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
    let (rem, _) = parse_tag(rem, ":")?;
    let (rem, te) = type_expression(rem)?;
    let pd = ParameterDefinition {
        name,
        type_expression: te,
        is_reference: false,
    };

    Ok((rem, pd))
}

fn parameter(input: &str) -> IResult<&str, LinkedList<ParameterDefinition>> {
    let res = alt([non_ref_parameter, ref_parameter]).parse(input)?;
    Ok((res.0, LinkedList::from([res.1])))
}

fn ref_parameter(input: &str) -> IResult<&str, ParameterDefinition> {
    let (rem, _) = parse_tag(input, "ref")?;
    let (rem, ident) = ident(rem)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };
    let (rem, _) = parse_tag(rem, ":")?;
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
    let (rem, mut vl) = variable_list(rem)?;
    vl.push_front(vd);
    Ok((rem, vl))
}

fn statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    alt([non_empty_statement_list, empty_statement_list]).parse(input)
}

fn empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_statement_list(input: &str) -> IResult<&str, LinkedList<Statement>> {
    let (rem, st) = statement(input)?;
    let (rem, mut stl) = statement_list(rem)?;
    stl.push_front(st);
    Ok((rem, stl))
}

fn statement(input: &str) -> IResult<&str, Statement> {
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
    let (rem, _) = parse_tag(input, ";")?;
    Ok((rem, Statement::EmptyStatement))
}

fn if_statement(input: &str) -> IResult<&str, Statement> {
    alt([if_statement_with_else, if_statement_without_else]).parse(input)
}

fn if_statement_without_else(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = parse_tag(input, "if")?;
    let (rem, _) = parse_tag(rem, "(")?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    let (rem, st) = statement(rem)?;
    let if_stmt = IfStatement {
        condition: ex,
        then_branch: st,
        else_branch: None,
    };
    Ok((rem, Statement::IfStatement(Box::new(if_stmt))))
}

fn if_statement_with_else(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = parse_tag(input, "if")?;
    let (rem, _) = parse_tag(rem, "(")?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    let (rem, then_part) = statement(rem)?;
    let (rem, _) = parse_tag(rem, "else")?;
    let (rem, else_part) = statement(rem)?;
    let if_stmt = IfStatement {
        condition: ex,
        then_branch: then_part,
        else_branch: Some(else_part),
    };

    Ok((rem, Statement::IfStatement(Box::new(if_stmt))))
}

fn while_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = parse_tag(input, "while")?;
    let (rem, _) = parse_tag(rem, "(")?;
    let (rem, cond) = expression(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    let (rem, st) = statement(rem)?;
    let while_stmt = WhileStatement {
        condition: cond,
        body: st,
    };
    Ok((rem, Statement::WhileStatement(Box::new(while_stmt))))
}

fn compound_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, _) = parse_tag(input, "{")?;
    let (rem, stl) = statement_list(rem)?;
    let (rem, _) = parse_tag(rem, "}")?;
    let stl = stl.into_iter().map(Box::new).collect();
    Ok((rem, Statement::CompoundStatement(stl)))
}

fn assign_statement(input: &str) -> IResult<&str, Statement> {
    let (rem, var) = variable(input)?;
    let (rem, _) = parse_tag(rem, ":=")?;
    let (rem, exp) = expression(rem)?;
    let (rem, _) = parse_tag(rem, ";")?;
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
    let (rem, _) = parse_tag(rem, "(")?;
    let (rem, arguments) = argument_list(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    let (rem, _) = parse_tag(rem, ";")?;
    let call_statement = CallStatement { name, arguments };

    Ok((rem, Statement::CallStatement(Box::new(call_statement))))
}

fn argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    alt([non_empty_argument_list, empty_argument_list]).parse(input)
}

fn empty_argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    Ok((input, LinkedList::from([])))
}

fn non_empty_argument_list(input: &str) -> IResult<&str, LinkedList<Expression>> {
    alt([more_than_one_argument, expression_head]).parse(input)
}

fn expression_head(input: &str) -> IResult<&str, LinkedList<Expression>> {
    let (rem, exp) = expression(input)?;
    Ok((rem, LinkedList::from([exp])))
}

fn more_than_one_argument(input: &str) -> IResult<&str, LinkedList<Expression>> {
    let (rem, mut ehead) = expression_head(input)?;
    let (rem, _) = parse_tag(rem, ",")?;
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
    )
    .parse(input)
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
    )
    .parse(input)
}

fn named_var(input: &str) -> IResult<&str, Variable> {
    let (rem, ident) = ident(input)?;
    let name = match ident {
        Tokens::Ident(name) => name,
        _ => panic!(),
    };

    Ok((rem, Variable::NamedVariable(name)))
}

fn expression(input: &str) -> IResult<&str, Expression> {
    expression0(input)
}

fn expression0(input: &str) -> IResult<&str, Expression> {
    let (mut inp, mut expr) = expression2(input)?;
    loop {
        if let Ok((rem, op)) = alt([eq, ne, le, lt, ge, gt]).parse(inp) {
            let (rem, right) = expression2(rem)?;
            expr = Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: match op {
                    Tokens::Eq => Operator::Equ,
                    Tokens::Ne => Operator::Neq,
                    Tokens::Le => Operator::Lse,
                    Tokens::Lt => Operator::Lst,
                    Tokens::Ge => Operator::Gre,
                    Tokens::Gt => Operator::Grt,
                    _ => unreachable!(),
                },
                left: expr,
                right,
            }));
            inp = rem;
        } else {
            break;
        }
    }
    Ok((inp, expr))
}

fn expression2(input: &str) -> IResult<&str, Expression> {
    let (mut inp, mut expr) = expression3(input)?;
    loop {
        if let Ok((rem, op)) = alt([plus, minus]).parse(inp) {
            let (rem, right) = expression3(rem)?;
            expr = Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: match op {
                    Tokens::Plus => Operator::Add,
                    Tokens::Minus => Operator::Sub,
                    _ => unreachable!(),
                },
                left: expr,
                right,
            }));
            inp = rem;
        } else {
            break;
        }
    }
    Ok((inp, expr))
}

fn expression3(input: &str) -> IResult<&str, Expression> {
    let (mut inp, mut expr) = expression4(input)?;
    loop {
        if let Ok((rem, op)) = alt([star, slash]).parse(inp) {
            let (rem, right) = expression4(rem)?;
            expr = Expression::BinaryExpression(Box::new(BinaryExpression {
                operator: match op {
                    Tokens::Star => Operator::Mul,
                    Tokens::Slash => Operator::Div,
                    _ => unreachable!(),
                },
                left: expr,
                right,
            }));
            inp = rem;
        } else {
            break;
        }
    }
    Ok((inp, expr))
}

fn expression4(input: &str) -> IResult<&str, Expression> {
    alt([unary_expression, expression5]).parse(input)
}

fn unary_expression(input: &str) -> IResult<&str, Expression> {
    let (rem, _) = parse_tag(input, "-")?;
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
    let (rem, _) = parse_tag(input, "(")?;
    let (rem, ex) = expression(rem)?;
    let (rem, _) = parse_tag(rem, ")")?;
    Ok((rem, ex))
}

fn expression5(input: &str) -> IResult<&str, Expression> {
    alt([intlit_exp, variable_exp, parentheses_exp]).parse(input)
}
