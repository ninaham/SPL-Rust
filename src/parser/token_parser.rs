use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, digit1, hex_digit1, multispace1, not_line_ending, satisfy},
    combinator::verify,
    multi::many0,
    sequence::{pair, preceded},
};

use super::tokens::Tokens;

pub fn parse_tag<'a>(input: &'a str, word: &'a str) -> IResult<&'a str, Tokens> {
    let (rem, _) = parse_whitespace(input)?;
    let res = tag(word)(rem)?;
    let (rem, _) = parse_whitespace(res.0)?;
    Ok((rem, Tokens::Other))
}

fn ident_first_char(input: &str) -> IResult<&str, char> {
    verify(anychar, |c: &char| c.is_alphabetic() || *c == '_').parse(input)
}

fn ident_later_chars(input: &str) -> IResult<&str, String> {
    let res = many0(satisfy(|c| c.is_alphanumeric() || c == '_')).parse(input)?;
    Ok((res.0, res.1.iter().collect()))
}

pub fn ident(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = parse_whitespace(input)?;
    let (rem, m) = ident_first_char(rem)?;
    let (rem1, m1) = ident_later_chars(rem)?;
    let (rem1, _) = parse_whitespace(rem1)?;
    Ok((rem1, Tokens::Ident(format!("{}{}", m, m1))))
}

fn int(input: &str) -> IResult<&str, Tokens> {
    let res = digit1(input)?;
    Ok((res.0, Tokens::Intlit(res.1.parse::<i64>().unwrap())))
}

fn newline(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = tag("'\\n'")(input)?;
    Ok((rem, Tokens::Intlit(10)))
}

pub fn parse_whitespace(input: &str) -> IResult<&str, ()> {
    let comment: fn(_) -> _ = comment;
    let res = many0(alt([multispace1, comment])).parse(input)?;
    Ok((res.0, ()))
}

fn character(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = tag("'")(input)?;
    let res = anychar(rem)?;
    let (rem, _) = tag("'")(res.0)?;
    Ok((rem, Tokens::Intlit(res.1 as i64)))
}

fn hex_num(input: &str) -> IResult<&str, Tokens> {
    let res = preceded(tag("0x"), hex_digit1).parse(input)?;
    Ok((
        res.0,
        Tokens::Intlit(i64::from_str_radix(res.1, 16).unwrap()),
    ))
}

pub fn intlit(input: &str) -> IResult<&str, Tokens> {
    let (rem, _) = parse_whitespace(input)?;
    let (rem, x) = alt([hex_num, int, character, newline]).parse(rem)?;
    let (rem, _) = parse_whitespace(rem)?;

    Ok((rem, x))
}

fn comment(input: &str) -> IResult<&str, &str> {
    let res = pair(tag("//"), not_line_ending).parse(input)?;
    Ok((res.0, ""))
}

pub fn eq(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "=")?;
    Ok((res.0, Tokens::Eq))
}

pub fn ne(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "#")?;
    Ok((res.0, Tokens::Ne))
}

pub fn lt(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "<")?;
    Ok((res.0, Tokens::Lt))
}

pub fn gt(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, ">")?;
    Ok((res.0, Tokens::Gt))
}

pub fn ge(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, ">=")?;
    Ok((res.0, Tokens::Ge))
}

pub fn le(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "<=")?;
    Ok((res.0, Tokens::Le))
}

pub fn plus(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "+")?;
    Ok((res.0, Tokens::Plus))
}

pub fn minus(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "-")?;
    Ok((res.0, Tokens::Minus))
}

pub fn star(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "*")?;
    Ok((res.0, Tokens::Star))
}

pub fn slash(input: &str) -> IResult<&str, Tokens> {
    let res = parse_tag(input, "/")?;
    Ok((res.0, Tokens::Slash))
}
