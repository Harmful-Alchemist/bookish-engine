use crate::Node::{DivN, MulN, NumberN, Temp};
use crate::Parser::{Digit, DivE, DivP, Exactly, Expr, MulE, MulP, Number, Or, Then};
use crate::Token::{DivT, MulT, One, Zero};
use std::env;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Need exactly one argument".to_string());
    }
    let tokens = tokenize(&args[1])?;
    // dbg!(&tokens);
    let parsed = Expr.parse(tokens, &[])?;
    //dbg!(&parsed);
    if !parsed.0.is_empty() {
        return Err(format!("Remaining input '{:?}'!", parsed.0));
    }
    let nodes = parsed.1;

    println!("{}", format!("{nodes:?}"));
    Ok(())
}

#[derive(Clone, Debug)]
enum Parser {
    Or(Box<Self>, Box<Self>),
    Then(Box<Self>, Box<Self>),
    Exactly(u8, Box<Self>),
    Digit,
    Number,
    Expr,
    MulP,
    DivP,
    MulE,
    DivE,
    If{pred: fn(Vec<Token>,Vec<Node>) -> bool, parser: Self}
}

impl Parser {
    fn parse(
        &self,
        tokens: Vec<Token>,
        nodes: &[Node],
    ) -> Result<(Vec<Token>, Vec<Node>), String> {
        match self {
            Then(first, second) => match first.parse(tokens.clone(), nodes)? {
                (first_ts, first_ns) if first_ts.len() < tokens.len() => {
                    match second.parse(first_ts.clone(), &first_ns)? {
                        (second_ts, second_ns) if second_ts.len() < first_ts.len() => {
                            Ok((second_ts, second_ns.to_vec()))
                        }
                        _ => Ok((tokens, nodes.to_vec())),
                    }
                }
                _ => Ok((tokens, nodes.to_vec())),
            },
            Expr => Or(Box::new(DivE), Box::new(MulE)).parse(tokens, nodes),
            DivP => match tokens[..] {
                [DivT, ..] => Ok((tokens[1..].to_vec(), nodes.to_vec())),
                _ => Ok((tokens, nodes.to_vec())),
            },
            MulP => match tokens[..] {
                [MulT, ..] => Ok((tokens[1..].to_vec(), nodes.to_vec())),
                _ => Ok((tokens, nodes.to_vec())),
            },
            Number => {
                return match Exactly(4, Box::new(Digit)).parse(tokens.clone(), nodes)? {
                    (new_tokens, new_ns) if new_ns.len() >= 4 && new_tokens.len() < tokens.len() => {
                        let mut new_nodes = vec![NumberN(to_i8(&new_ns[0..4])?)];
                        new_nodes.append(&mut nodes.to_vec());
                        //dbg!(&new_nodes);
                        Ok((new_tokens, new_nodes))
                    }
                    _ => Ok((tokens, nodes.to_vec())),
                };
            }
            Or(first, second) => match first.parse(tokens.clone(), nodes)? {
                //TODO ah the number works? And still in nodes? Dunno...
                (new_tokens, new_nodes) if new_tokens.len() < tokens.len() => {
                    Ok((new_tokens, new_nodes))
                }
                _ => second.parse(tokens, nodes),
            },
            Exactly(amount, parser) => {
                let mut next = (tokens.clone(), nodes.to_vec());
                for i in 0..*amount {
                    // dbg!(i);
                    match parser.parse(next.0.clone(), &next.1)? {
                        (new_tokens, new_nodes) if new_tokens.len() < next.0.len() => {
                            next = (new_tokens, new_nodes)
                        }
                        _ => return Ok((tokens, nodes.to_vec())),
                    }
                }
                Ok(next)
            }
            Digit => {
                // dbg!(&tokens);
                match tokens[0] {
                    Zero => {
                        let mut new_nodes = vec![Temp(0)];
                        new_nodes.append(&mut nodes.to_vec());
                        // dbg!(&new_nodes);
                        // dbg!(&tokens);
                        // dbg!(&tokens[1..]);
                        Ok((tokens[1..].to_vec(), new_nodes))
                    }
                    One => {
                        let mut new_nodes = vec![Temp(1)];
                        new_nodes.append(&mut nodes.to_vec());
                        Ok((tokens[1..].to_vec(), new_nodes))
                    }
                    _ => Ok((tokens, nodes.to_vec())),
                }
            }
            MulE => {
                //dbg!("MulE");
                match Then(
                    Box::new(Then(
                        Box::new(Number), //right recursive doesn't work next step
                        Box::new(MulP),
                    )),
                    Box::new(Number),
                )
                    .parse(tokens.clone(), nodes)?
                {
                    (new_ts, new_ns) if new_ts.len() < tokens.len() => {
                        let mut newer_ns = vec![MulN {
                            lhs: Box::new(new_ns[0].clone()),
                            rhs: Box::new(new_ns[1].clone()),
                        }];
                        newer_ns.append(&mut new_ns[2..].to_vec());
                        Ok((new_ts, newer_ns))
                    }
                    _ => Ok((tokens, nodes.to_vec())),
                }
            }
            DivE => {
                //dbg!("MulE");
                match Then(
                    Box::new(Then(
                        Box::new(Number), //right recursive doesn't work next step
                        Box::new(DivP),
                    )),
                    Box::new(Number),
                )
                    .parse(tokens.clone(), nodes)?
                {
                    (new_ts, new_ns) if new_ts.len() < tokens.len() => {
                        let mut newer_ns = vec![DivN {
                            lhs: Box::new(new_ns[0].clone()),
                            rhs: Box::new(new_ns[1].clone()),
                        }];
                        newer_ns.append(&mut new_ns[2..].to_vec());
                        Ok((new_ts, newer_ns))
                    }
                    _ => Ok((tokens, nodes.to_vec())),
                }
            }
        }
    }
}

fn to_i8(inp: &[Node]) -> Result<i8, &'static str> {
    let mut res = 0;
    //dbg!(&inp);
    for x in inp {
        res = res << 1;
        match x {
            Temp(0) => {}
            Temp(1) => res = res + 1,
            _ => return Err("Uh no"),
        }
    }
    Ok(res)
}

#[derive(Clone, Debug,Eq,PartialEq)]
enum Node {
    NumberN(i8),
    MulN { rhs: Box<Self>, lhs: Box<Self> },
    DivN { rhs: Box<Self>, lhs: Box<Self> },
    Temp(i8),
}

fn tokenize(inp: &String) -> Result<Vec<Token>, &'static str> {
    if !inp.is_ascii() {
        return Err("I only know ascii 😀");
    }

    Ok(inp
        .chars()
        .filter_map(|c| {
            // dbg!(&c);
            match c {
                '1' => Some(One),
                '0' => Some(Zero),
                '/' => Some(DivT),
                '*' => Some(MulT),
                _ => None,
            }
        })
        .collect())
}

#[derive(Debug, Copy, Clone,Eq, PartialEq)]
enum Token {
    Zero,
    One,
    DivT,
    MulT,
}
