use crate::{Chemical, ChemToken, NumberToken, NumberOperator};
use std::collections::HashMap;
use std::boxed::Box;

#[derive(Debug, Default)]
pub struct ParseError {
    position:u32,
    msg:String
}

const AMMONIA:&str = "($/1:hydrogen;$/3:nitrogen;)";
const DIETHYLAMINE:&str = "($/2:ethanol;$/2:*AMMONIA;)@374;";
const OIL:&str = "($/2:weldingfuel;$/2:carbon;$/2:hydrogen;)";
const EPHEDRINE:&str = "($/3:sugar;$/3:hydrogen;$/3:*OIL;$/2:*DIETHYLAMINE;)";
const METH_FORMULA:&str = "($/3:phosphorus;$/3:hydrogen;$/3:iodine;$/3:*EPHEDRINE;)@374;";
const SULFURIC_ACID:&str = "($/2:sulfur;$/2:oxygen;$/2:hydrogen;)";
const FLUOROSULFURIC_ACID:&str = "($/3:*SULFURIC_ACID;$/3:fluorine;$/3:hydrogen;$/3:potassium;)@374;";
const STABILIZING_AGENT:&str = "($/2:iron;$/2:hydrogen;$/2:oxygen;)";
const PHLOGISTON:&str = "($/4:*STABILIZING_AGENT;!1;$/4:phosphorus;$/4:plasma;$/4:*SULFURIC_ACID;)";
const LIQUID_DARK_MATTER:&str = "($/4:*STABILIZING_AGENT;!1;$/4:radium;$/4:plasma;$/4:carbon;)";
const SMOKE_POWDER:&str = "($/3:*STABILIZING_AGENT;!1;$/3:potassium;$/3:sugar;$/3:phosphorus;)";
const FLUOROSURFACTANT:&str = "($/3:fluorine;$/3:*OIL;$/3:*SULFURIC_ACID;)";
const ACETONE:&str = "($/3:*OIL;$/3:weldingfuel;$/3:oxygen;)";
const ATRAZINE:&str = "($/3:chlorine;$/3:nitrogen;$/3:hydrogen;)";
const PHENOL:&str = "($/3:*OIL;$/3:chlorine;$/3:water;)";
const SALICYLIC_ACID:&str = "($/5:sodium;$/5:*PHENOL;$/5:carbon;$/5:oxygen;$/5:*SULFURIC_ACID;)";
const PERFLUORODECALIN:&str = "($/2:hydrogen;$/2:*SALICYLIC;$/2:fluorine;)@374;";
const PENTETIC_ACID:&str = "($/6:weldingfuel;$/6:chlorine;$/6:fluorine;$/6:*AMMONIA;$/6:*FORMALDEHYDE;$/6:*CYANIDE;)";
const FORMALDEHYDE:&str = "($/2:ethanol;$/2:oxygen;$/2:silver;)@424;";
const CYANIDE:&str = "($/1:*AMMONIA;$/1:*OIL;$/1:oxygen;)@374;";


lazy_static!{
    pub static ref SUB_MAP: HashMap<&'static str, &'static str> = [
        ("FLUOROSULFURIC_ACID", FLUOROSULFURIC_ACID),
        ("METH", METH_FORMULA),
        ("AMMONIA", AMMONIA),
        ("DIETHYLAMINE", DIETHYLAMINE),
        ("OIL", OIL),
        ("EPHEDRINE", EPHEDRINE),
        ("SULFURIC_ACID", SULFURIC_ACID),
        ("STABILIZING_AGENT", STABILIZING_AGENT),
        ("PHLOGISTON", PHLOGISTON),
        ("LIQUID_DARK_MATTER", LIQUID_DARK_MATTER),
        ("SMOKE_POWDER", SMOKE_POWDER),
        ("FLUOROSURFACTANT", FLUOROSURFACTANT),
        ("ACETONE", ACETONE),
        ("ATRAZINE", ATRAZINE),
        ("PHENOL", PHENOL),
        ("SALICYLIC", SALICYLIC_ACID),
        ("PHENOL", PHENOL),
        ("PERFLUORODECALIN", PERFLUORODECALIN),
        ("PENTETIC", PENTETIC_ACID),
        ("FORMALDEHYDE", FORMALDEHYDE),
        ("CYANIDE", CYANIDE),
    ].iter().copied().collect();
}

fn get_substitute_formula(name:String) -> &'static str {
    let name = name.to_ascii_uppercase();
    let name = name.as_str();
    if !SUB_MAP.contains_key(&name) {
        panic!("MISSING: {}", name);
    }
    return SUB_MAP[name];
}

impl ParseError {
    fn new(position:u32) -> ParseError {
        return ParseError{position, ..Default::default()}
    }


    fn with_msg(position:u32, msg:&str) -> ParseError {
        return ParseError{position, msg:msg.to_string()}
    }
}

pub fn parse(string:String) -> (Result<ChemToken, ParseError>, u32) {
    let mut tokens = tokenize(string);
    let mut tokens_copy = tokens.clone();
    let final_quantity;
    let quantity = parse_number(&mut tokens_copy).unwrap();
    let maybe_x = peek(&tokens_copy).unwrap();
    if maybe_x == 'x' {
        assert_token(&mut tokens_copy, 'x').unwrap();
        final_quantity = match quantity {
            NumberToken::Constant(val) => {val},
            _ => panic!("Invalid term in constant")
        }
    } else {
        final_quantity = 1;
    }
    (if maybe_x == 'x' {parse_group_or_base(&mut tokens_copy, None)} else {parse_group_or_base(&mut tokens, None)}, final_quantity)
}

fn parse_quantity(string:String, quantity:NumberToken) -> Result<ChemToken, ParseError> {
    let mut tokens = tokenize(string);
    parse_group_or_base(&mut tokens, Some(quantity))
}

fn tokenize(string:String) -> Vec<char> {
    return string.chars().rev().collect()
}

fn parse_group_or_base(tokens: &mut Vec<char>, last_quantity:Option<NumberToken>) -> Result<ChemToken, ParseError> {
    let quantity = parse_number(tokens)?;
    assert_token(tokens, ':')?;
    if tokens.is_empty() {
        return Err(ParseError::with_msg(tokens.len() as u32, "missing (, end of feed"));
    }
    let next = peek(tokens)?;
    match next {
        '(' => return parse_group(tokens, quantity),
        '*' => return parse_subbed_chem(tokens, quantity),
        _ => return parse_base_chem(tokens, quantity)
    }
}

fn assert_token(tokens: &mut Vec<char>, matches:char) -> Result<(), ParseError> {
    let token = tokens.pop();
    if token.is_none() {
        return Err(ParseError::with_msg(tokens.len() as u32, "bad assert, end of feed"));
    }
    let token = token.unwrap();
    if token != matches {
        return Err(ParseError::with_msg(tokens.len() as u32, format!("bad assert: got {}, expected {}", token,  matches).as_str() ));
    } else {
        return Ok(());
    }
}

fn parse_subbed_chem(tokens: &mut Vec<char>, quantity:NumberToken) -> Result<ChemToken, ParseError> {
    assert_token(tokens, '*')?;
    let name = parse_name(tokens)?;
    let formula = get_substitute_formula(name);
    let (result, _) = parse(format!("{}:{}", quantity.as_text(), formula));
    let mut result = result.unwrap();
    let priority = parse_priority(tokens)?;
    result.priority = priority;
    return Ok(result);
}

/// chem group of format "50:(<chem>,..)@<temp>;" where the "@<temp>;" is optional
fn parse_group(tokens: &mut Vec<char>, quantity:NumberToken) -> Result<ChemToken, ParseError> {
    let mut chems = vec![];
    assert_token(tokens, '(')?;
    while peek(tokens)?!=')' {
        chems.push(parse_group_or_base(tokens, Some(quantity.clone()))?);
        if tokens.is_empty() {
            return Err(ParseError::with_msg(tokens.len() as u32, "missing ), end of feed"));
        }
    }
    assert_token(tokens, ')')?;
    let temp;
    if !tokens.is_empty() && peek(tokens)?=='@' {
        tokens.pop();
        let temp_token = parse_number(tokens)?;
        match temp_token {
            NumberToken::Constant(val) => {
                temp = Some(val);
            },
            _ => panic!("Bad value in temp constant")
        }
        let token = tokens.pop();
        if token.is_none() {
            return Err(ParseError::with_msg(tokens.len() as u32, "group hit end of feed"));
        }
        let token = token.unwrap();
        if token != ';' {
            return Err(ParseError::with_msg(tokens.len() as u32, "missing semicolon after temp"));
        }
    } else {
        temp = None;
    }
    let priority = parse_priority(tokens)?;
    return Ok(ChemToken {quantity:quantity, priority, concrete_quantity:None, chemical: Chemical {chemicals:chems, temp:temp, ..Default::default()}});
}

fn parse_priority(tokens: &mut Vec<char>) -> Result<u32, ParseError> {
    if tokens.is_empty() {
        return Ok(0);
    }
    let next = peek(tokens)?;
    if next == '!' {
        assert_token(tokens, '!')?;
        let number = match parse_number(tokens)? {
            NumberToken::Constant(val) => {
                val
            },
            _ => panic!("Bad constant in priority")
        };
        assert_token(tokens, ';')?;
        return Ok(number);
    } else {
        return Ok(0);
    }
}

fn peek(tokens: &Vec<char>) -> Result<char, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::with_msg(tokens.len() as u32, format!("peeked at end of line").as_str()));
    } else {
        return Ok(*tokens.last().unwrap());
    }
}

/// basic chem of format "<amount>:<name>;" ie "50:nitrogen;"
fn parse_base_chem(tokens: &mut Vec<char>, quantity:NumberToken) -> Result<ChemToken, ParseError> {
    let chem_name = parse_name(tokens)?;
    let priority = parse_priority(tokens)?;
    return Ok(ChemToken {quantity:quantity, priority, concrete_quantity:None, chemical: Chemical {name:Some(chem_name), ..Default::default()}});
}

fn parse_name(tokens: &mut Vec<char>) -> Result<String, ParseError> {
    let mut buffer = vec![];
    let token = tokens.pop();
    if token.is_none() {
        return Err(ParseError::with_msg(tokens.len() as u32, "name empty"));
    }
    buffer.push(token.unwrap());
    let mut token = tokens.pop();
    while token.is_some() && token.unwrap() != ';' {
        buffer.push(token.unwrap());
        token = tokens.pop();
    }
    if token.is_none() { // no semicolon
        return Err(ParseError::with_msg(tokens.len() as u32, "missing semicolon"));
    }
    let name:String =  buffer.iter().collect();
    return Ok(name);
}

fn parse_number(tokens: &mut Vec<char>) -> Result<NumberToken,ParseError> {
    let peek_res = peek(tokens)?;
    if peek_res.is_digit(10) {
        let mut digit = parse_digit(tokens);
        if digit.is_none() {
            return Err(ParseError::with_msg(tokens.len() as u32, "number parse error, bad digit"));
        }
        let mut sum = digit.unwrap();
        digit = parse_digit(tokens);
        while digit.is_some() {
            sum *= 10;
            sum += digit.unwrap();
            digit = parse_digit(tokens);
        }
        return Ok(NumberToken::Constant(sum));
    } else if peek_res == '$' {
        assert_token(tokens, '$')?;
        let peek_res = peek(tokens).unwrap();
        let numerator;
        if peek_res.is_digit(10) {
            numerator = parse_digit(tokens).unwrap();
        } else {
            numerator = 1;
        }
        assert_token(tokens, '/')?;
        let digit = parse_digit(tokens).unwrap();
        return Ok(NumberToken::Calculated(NumberOperator::new(numerator,digit)))
    } else {
        return Err(ParseError::with_msg(tokens.len() as u32, format!("number parse error, NAN: {}", peek_res).as_str()));
    }
}

fn parse_digit(tokens: &mut Vec<char>) -> Option<u32> {
    let token = tokens.pop();
    if token.is_none() {
        return None;
    }
    let token = token.unwrap();
    if !token.is_digit(10) {
        tokens.push(token);
        return None;
    }
    return Some(token.to_digit(10).unwrap());
}