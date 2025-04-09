use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, digit1, hex_digit1, multispace0, multispace1, satisfy},
    combinator::verify,
    multi::many0,
    sequence::preceded,
};

use super::tokens::Tokens;

pub fn r#if(input: &str) -> IResult<&str, Tokens> {
    let res = tag("if")(input)?;
    Ok((res.0, Tokens::If))
}

pub fn r#type(input: &str) -> IResult<&str, Tokens> {
    let res = tag("type")(input)?;
    Ok((res.0, Tokens::Type))
}

pub fn proc(input: &str) -> IResult<&str, Tokens> {
    let res = tag("proc")(input)?;
    Ok((res.0, Tokens::Proc))
}

pub fn array(input: &str) -> IResult<&str, Tokens> {
    let res = tag("array")(input)?;
    Ok((res.0, Tokens::Array))
}

pub fn of(input: &str) -> IResult<&str, Tokens> {
    let res = tag("of")(input)?;
    Ok((res.0, Tokens::Of))
}

pub fn r#ref(input: &str) -> IResult<&str, Tokens> {
    let res = tag("ref")(input)?;
    Ok((res.0, Tokens::Ref))
}

pub fn var(input: &str) -> IResult<&str, Tokens> {
    let res = tag("var")(input)?;
    Ok((res.0, Tokens::Var))
}

pub fn r#else(input: &str) -> IResult<&str, Tokens> {
    let res = tag("else")(input)?;
    Ok((res.0, Tokens::Else))
}

pub fn r#while(input: &str) -> IResult<&str, Tokens> {
    let res = tag("while")(input)?;
    Ok((res.0, Tokens::While))
}

pub fn lparen(input: &str) -> IResult<&str, Tokens> {
    let res = tag("(")(input)?;
    Ok((res.0, Tokens::Lparen))
}

pub fn rparen(input: &str) -> IResult<&str, Tokens> {
    let res = tag(")")(input)?;
    Ok((res.0, Tokens::Rparen))
}

pub fn lbrack(input: &str) -> IResult<&str, Tokens> {
    let res = tag("[")(input)?;
    Ok((res.0, Tokens::Lbrack))
}

pub fn rbrack(input: &str) -> IResult<&str, Tokens> {
    let res = tag("]")(input)?;
    Ok((res.0, Tokens::Rbrack))
}

pub fn lcurl(input: &str) -> IResult<&str, Tokens> {
    let res = tag("{")(input)?;
    Ok((res.0, Tokens::Lcurl))
}

pub fn rcurl(input: &str) -> IResult<&str, Tokens> {
    let res = tag("}")(input)?;
    Ok((res.0, Tokens::Rcurl))
}

pub fn eq(input: &str) -> IResult<&str, Tokens> {
    let res = tag("=")(input)?;
    Ok((res.0, Tokens::Eq))
}

pub fn ne(input: &str) -> IResult<&str, Tokens> {
    let res = tag("#")(input)?;
    Ok((res.0, Tokens::Ne))
}

pub fn lt(input: &str) -> IResult<&str, Tokens> {
    let res = tag("<")(input)?;
    Ok((res.0, Tokens::Lt))
}

pub fn gt(input: &str) -> IResult<&str, Tokens> {
    let res = tag(">")(input)?;
    Ok((res.0, Tokens::Gt))
}

pub fn ge(input: &str) -> IResult<&str, Tokens> {
    let res = tag(">=")(input)?;
    Ok((res.0, Tokens::Ge))
}

pub fn le(input: &str) -> IResult<&str, Tokens> {
    let res = tag("<=")(input)?;
    Ok((res.0, Tokens::Le))
}

pub fn plus(input: &str) -> IResult<&str, Tokens> {
    let res = tag("+")(input)?;
    Ok((res.0, Tokens::Plus))
}

pub fn minus(input: &str) -> IResult<&str, Tokens> {
    let res = tag("-")(input)?;
    Ok((res.0, Tokens::Minus))
}

pub fn star(input: &str) -> IResult<&str, Tokens> {
    let res = tag("*")(input)?;
    Ok((res.0, Tokens::Star))
}

pub fn slash(input: &str) -> IResult<&str, Tokens> {
    let res = tag("/")(input)?;
    Ok((res.0, Tokens::Slash))
}

pub fn colon(input: &str) -> IResult<&str, Tokens> {
    let res = tag(":")(input)?;
    Ok((res.0, Tokens::Colon))
}

pub fn comma(input: &str) -> IResult<&str, Tokens> {
    let res = tag(",")(input)?;
    Ok((res.0, Tokens::Comma))
}

pub fn semic(input: &str) -> IResult<&str, Tokens> {
    let res = tag(";")(input)?;
    Ok((res.0, Tokens::Semic))
}

pub fn asgn(input: &str) -> IResult<&str, Tokens> {
    let res = tag(":=")(input)?;
    Ok((res.0, Tokens::Asgn))
}

pub fn ident_first_char(input: &str) -> IResult<&str, char> {
    verify(anychar, |c: &char| c.is_alphabetic() || *c == '_').parse(input)
}

pub fn ident_later_chars(input: &str) -> IResult<&str, String> {
    let res = many0(satisfy(|c| c.is_alphanumeric() || c == '_')).parse(input)?;
    Ok((res.0, res.1.iter().collect()))
}

pub fn ident(input: &str) -> IResult<&str, Tokens> {
    let (rem, m) = ident_first_char(input)?;
    let (rem1, m1) = ident_later_chars(rem)?;

    Ok((rem1, Tokens::Ident(format!("{}{}", m, m1))))
}

pub fn int(input: &str) -> IResult<&str, Tokens> {
    let res = digit1(input)?;
    Ok((res.0, Tokens::Intlit(res.1.parse::<i64>().unwrap())))
}

pub fn newline(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = tag("'\\n'")(input)?;
    Ok((rem, Tokens::Intlit(10)))
}

pub fn whitespace0(input: &str) -> IResult<&str, ()> {
    let res = multispace0(input)?;
    Ok((res.0, ()))
}

pub fn whitespace1(input: &str) -> IResult<&str, ()> {
    let res = multispace1(input)?;
    Ok((res.0, ()))
}

pub fn character(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = tag("'")(input)?;
    let res = anychar(rem)?;
    let (rem, _) = tag("'")(res.0)?;
    Ok((rem, Tokens::Intlit(res.1 as i64)))
}

pub fn hex_num(input: &str) -> IResult<&str, Tokens> {
    let res = preceded(tag("0x"), hex_digit1).parse(input)?;
    Ok((
        res.0,
        Tokens::Intlit(i64::from_str_radix(res.1, 16).unwrap()),
    ))
}

pub fn intlit(input: &str) -> IResult<&str, Tokens> {
    alt([hex_num, int, character]).parse(input)
}
