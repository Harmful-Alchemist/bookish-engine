use crate::tokens::Token::{DivT, MulT, One, Zero};

//Tokenize
pub fn tokenize(inp: &String) -> Result<Vec<Token>, &'static str> {
    if !inp.is_ascii() {
        return Err("I only know ascii 😀");
    }

    Ok(inp
        .chars()
        .filter_map(|c| match c {
            '1' => Some(One),
            '0' => Some(Zero),
            '/' => Some(DivT),
            '*' => Some(MulT),
            _ => None,
        })
        .collect())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Token {
    Zero,
    One,
    DivT,
    MulT,
}
