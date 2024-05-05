use crate::parsing::Node::{DivN, MulN, NumberN, Temp};
use crate::parsing::Parser::{
    Digit, DivE, DivP, Exactly, Expr, If, MulE, MulP, Number, Or, Repeat, Then,
};
use crate::tokens::Token;
use crate::tokens::Token::{DivT, MulT, One, Zero};

//Parsing
#[derive(Clone, Debug)]
pub enum Parser {
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
    If {
        pred: fn(Vec<Token>, &[Node]) -> bool,
        parser: Box<Self>,
    },
    Repeat(Box<Self>),
}

impl Parser {
    pub(crate) fn parse(
        &self,
        tokens: Vec<Token>,
        nodes: &[Node],
    ) -> Result<(Vec<Token>, Vec<Node>), String> {
        match self {
            Repeat(parser) => {
                let mut prev = (tokens, nodes.to_vec());
                let mut next = parser.parse(prev.0.clone(), &prev.1)?;
                while !prev.0.is_empty() && next.0.len() < prev.0.len() {
                    prev = (next.0.clone(), next.1);
                    next = parser.parse(prev.0.clone(), &prev.1)?;
                }
                Ok((prev.0, prev.1.to_vec()))
            }
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
            Expr => Repeat(Box::new(Or(Box::new(DivE), Box::new(MulE)))).parse(tokens, nodes), //Inner or empty but eh.
            DivP => match tokens[..] {
                [DivT, ..] => Ok((tokens[1..].to_vec(), nodes.to_vec())),
                _ => Ok((tokens, nodes.to_vec())),
            },
            MulP => match tokens[..] {
                [MulT, ..] => Ok((tokens[1..].to_vec(), nodes.to_vec())),
                _ => Ok((tokens, nodes.to_vec())),
            },
            Number => match Exactly(4, Box::new(Digit)).parse(tokens.clone(), nodes)? {
                (new_tokens, new_ns) if new_ns.len() >= 4 && new_tokens.len() < tokens.len() => {
                    let mut new_nodes = vec![NumberN(to_i8(&new_ns[0..4])?)];
                    new_nodes.append(&mut nodes.to_vec());
                    Ok((new_tokens, new_nodes))
                }
                _ => Ok((tokens, nodes.to_vec())),
            },
            Or(first, second) => match first.parse(tokens.clone(), nodes)? {
                (new_tokens, new_nodes) if new_tokens.len() < tokens.len() => {
                    Ok((new_tokens, new_nodes))
                }
                _ => second.parse(tokens, nodes),
            },
            Exactly(amount, parser) => {
                let mut next = (tokens.clone(), nodes.to_vec());
                for _ in 0..*amount {
                    match parser.parse(next.0.clone(), &next.1)? {
                        (new_tokens, new_nodes) if new_tokens.len() < next.0.len() => {
                            next = (new_tokens, new_nodes)
                        }
                        _ => return Ok((tokens, nodes.to_vec())),
                    }
                }
                Ok(next)
            }
            Digit => match tokens[0] {
                Zero => {
                    let mut new_nodes = vec![Temp(0)];
                    new_nodes.append(&mut nodes.to_vec());
                    Ok((tokens[1..].to_vec(), new_nodes))
                }
                One => {
                    let mut new_nodes = vec![Temp(1)];
                    new_nodes.append(&mut nodes.to_vec());
                    Ok((tokens[1..].to_vec(), new_nodes))
                }
                _ => Ok((tokens, nodes.to_vec())),
            },
            MulE => op(tokens, nodes, false),
            DivE => op(tokens, nodes, true),
            If { pred, parser } => {
                if pred(tokens.clone(), nodes) {
                    parser.parse(tokens, nodes)
                } else {
                    Ok((tokens, nodes.to_vec()))
                }
            }
        }
    }
}

fn op(tokens: Vec<Token>, nodes: &[Node], div: bool) -> Result<(Vec<Token>, Vec<Node>), String> {
    if tokens.is_empty() {
        return Ok((tokens, nodes.to_vec()));
    }

    let parser = if div { DivP } else { MulP };
    match Then(
        Box::new(Or(
            Box::new(If {
                pred: |_, y| matches!(y.first(), Some(MulN { .. }) | Some(DivN { .. })),
                parser: Box::new(parser.clone()),
            }),
            Box::new(Then(Box::new(Number), Box::new(parser))),
        )),
        Box::new(Number),
    )
    .parse(tokens.clone(), nodes)?
    {
        (new_ts, new_ns) if new_ts.len() < tokens.len() => {
            let lhs = Box::new(new_ns[0].clone());
            let rhs = Box::new(new_ns[1].clone());
            let node = if div {
                DivN { rhs: lhs, lhs: rhs }
            } else {
                MulN { rhs: lhs, lhs: rhs }
            };
            let mut newer_ns = vec![node];
            newer_ns.append(&mut new_ns[2..].to_vec());
            Ok((new_ts, newer_ns))
        }
        _ => Ok((tokens, nodes.to_vec())),
    }
}

fn to_i8(inp: &[Node]) -> Result<i8, &'static str> {
    let mut res = 0;
    for x in inp {
        res >>= 1;
        match x {
            Temp(0) => {}
            Temp(1) => res += 8,
            _ => return Err("Uh no"),
        }
    }
    Ok(res)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
    NumberN(i8),
    MulN { lhs: Box<Self>, rhs: Box<Self> },
    DivN { lhs: Box<Self>, rhs: Box<Self> },
    Temp(i8),
}
