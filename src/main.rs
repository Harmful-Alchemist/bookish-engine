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
