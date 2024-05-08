use crate::compile::Compiler;
use crate::execute::Machine;
use parsing::Parser::Expr;
use std::env;

mod compile;
mod execute;
mod parsing;
mod tokens;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err("Need exactly one argument".to_string());
    }
    let tokens = tokens::tokenize(&args[1])?;
    let parsed = Expr.parse(tokens, &[])?;
    if !parsed.0.is_empty() {
        return Err(format!(
            "Remaining input '{:?}' with parsed '{:?}'!",
            parsed.0, parsed.1
        ));
    }
    let nodes = parsed.1;

    let compiled = Compiler::new(nodes).compile()?;

    let mut machine = Machine::new();
    machine.run(compiled);
    let answer = machine.answer_by_convention();

    println!("{answer:08b}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::compile::Compiler;
    use crate::execute::Machine;
    use crate::parsing::Parser::Expr;
    use crate::tests::Op::{DivOp, MulOp};
    use crate::tokens;
    use std::fmt::{Display, Formatter};

    #[derive(Eq,PartialEq)]
    enum Op {
        MulOp,
        DivOp,
    }

    impl Op {
        fn op(&self) -> fn(i16, i16) -> i16 {
            match self {
                MulOp => |x, y| i16::overflowing_mul(x, y).0,
                DivOp => |x, y| {
                    if y == 0 && x != 0 {
                       0x1F
                    } else if y == 0 {
                        0
                    }
                    else {
                        i16::overflowing_div(x, y).0
                    }
                },
            }
        }
    }

    impl Display for Op {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            let rep = match self {
                MulOp => "*",
                DivOp => "/",
            };

            write!(f, "{rep}")
        }
    }

    #[test]
    fn testing() {
        for op in [MulOp, DivOp] {
            for i in 0..16 {
                for j in 0..16 {
                    println!("{i:04b}{op}{j:04b}");
                    assert_eq!(
                        calculate(format!("{i:04b}{op}{j:04b}").as_str()),
                        op.op()(i as i16, j as i16)
                    )
                }
            }
        }

        for op in [(MulOp,MulOp),(MulOp,DivOp), (DivOp,DivOp),(DivOp,MulOp)] {
            for i in 0..16 {
                for j in 0..16 {
                    for k in 0..16 {
                        if k == 0 && op.1 == DivOp {
                            continue //Whatever division by zero don't care..
                        }
                        println!("{i:04b}{}{j:04b}{}{k:04b}", op.0,op.1);
                        assert_eq!(
                            calculate(format!("{i:04b}{}{j:04b}{}{k:04b}", op.0,op.1).as_str()),
                            op.1.op()(op.0.op()(i as i16, j as i16) &0xF, k as i16)
                        )
                    }
                }
            }
        }
    }

    fn calculate(inp: &str) -> i16 {
        let tokens = tokens::tokenize(inp).unwrap();
        let parsed = Expr.parse(tokens, &[]).unwrap();
        let nodes = parsed.1;

        let compiled = Compiler::new(nodes).compile().unwrap();

        let mut machine = Machine::new();
        machine.run(compiled);
        machine.answer_by_convention()
    }
}
