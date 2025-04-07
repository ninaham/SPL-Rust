use nom::{bytes::complete::tag, IResult};

pub fn r#if(input: &str) -> IResult<&str, &str> {
    tag("if")(input)
}

pub fn r#type(input: &str) -> IResult<&str, &str> {
    tag("type")(input)
}

pub fn proc(input: &str) -> IResult<&str, &str> {
    tag("proc")(input)
}

pub fn array(input: &str) -> IResult<&str, &str> {
    tag("array")(input)
}

pub fn of(input: &str) -> IResult<&str, &str> {
    tag("of")(input)
}

pub fn r#ref(input: &str) -> IResult<&str, &str> {
    tag("ref")(input)
}

pub fn var(input: &str) -> IResult<&str, &str> {
    tag("var")(input)
}

pub fn r#else(input: &str) -> IResult<&str, &str> {
    tag("else")(input)
}

pub fn r#while(input: &str) -> IResult<&str, &str> {
    tag("while")(input)
}

pub fn lparen(input: &str) -> IResult<&str, &str> {
    tag("(")(input)
}

pub fn rparen(input: &str) -> IResult<&str, &str> {
    tag(")")(input)
}

pub fn lbrack(input: &str) -> IResult<&str, &str> {
    tag("[")(input)
}

pub fn rbrack(input: &str) -> IResult<&str, &str> {
    tag("]")(input)
}

pub fn lcurl(input: &str) -> IResult<&str, &str> {
    tag("{")(input)
}

pub fn rcurl(input: &str) -> IResult<&str, &str> {
    tag("{")(input)
}

pub fn eq(input: &str) -> IResult<&str, &str> {
    tag("=")(input)
}

pub fn ne(input: &str) -> IResult<&str, &str> {
    tag("#")(input)
}

pub fn lt(input: &str) -> IResult<&str, &str> {
    tag("<")(input)
}

pub fn gt(input: &str) -> IResult<&str, &str> {
    tag(">")(input)
}

pub fn ge(input: &str) -> IResult<&str, &str> {
    tag(">=")(input)
}

pub fn le(input: &str) -> IResult<&str, &str> {
    tag("<=")(input)
}

pub fn plus(input: &str) -> IResult<&str, &str> {
    tag("+")(input)
}

pub fn minus(input: &str) -> IResult<&str, &str> {
    tag("-")(input)
}

pub fn star(input: &str) -> IResult<&str, &str> {
    tag("*")(input)
}

pub fn slash(input: &str) -> IResult<&str, &str> {
    tag("/")(input)
}

pub fn colon(input: &str) -> IResult<&str, &str> {
    tag(":")(input)
}

pub fn comma(input: &str) -> IResult<&str, &str> {
    tag(",")(input)
}

pub fn semic(input: &str) -> IResult<&str, &str> {
    tag(";")(input)
}

pub fn asgn(input: &str) -> IResult<&str, &str> {
    tag(":=")(input)
}