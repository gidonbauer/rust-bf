use std::process::Command;

mod interpreter;
mod lexer;
mod tokenizer;
mod transpiler;

use interpreter::*;
use tokenizer::*;
use transpiler::*;

enum Target {
    INTERPRET,
    TRANSPILE,
}

fn usage(prog: String) {
    eprintln!("Usage: {prog} <interpret|transpile> <input file> [...]");
}

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    let program_name = args.next().unwrap();

    let target = match args.next().as_deref() {
        Some("interpret") => Target::INTERPRET,
        Some("transpile") => Target::TRANSPILE,
        Some(s) => {
            usage(program_name);
            return Err(format!(
                "Expected target `interpret` or `transpile` but got `{s}`."
            ));
        }
        None => {
            usage(program_name);
            return Err(format!(
                "Expected target `interpret` or `transpile` but got nothing."
            ));
        }
    };

    let input_file = if let Some(s) = args.next() {
        s
    } else {
        usage(program_name);
        return Err(format!("Expected input file but got nothing."));
    };

    let mut lexer = Lexer::new(&input_file)?;
    let mut prog = Program::new();
    prog.tokenize(&mut lexer)?;

    match target {
        Target::INTERPRET => {
            let mut inter = Interpreter::new();
            inter.run(&prog);
        }
        Target::TRANSPILE => {
            let llvm_file = if let Some(s) = args.next() {
                s
            } else {
                if input_file.ends_with(".bf") {
                    input_file.strip_suffix(".bf").unwrap().to_owned() + ".ll"
                } else {
                    input_file.clone() + ".ll"
                }
            };

            let exe_file = if let Some(s) = args.next() {
                s
            } else {
                if input_file.ends_with(".bf") {
                    input_file.strip_suffix(".bf").unwrap().to_owned()
                } else {
                    "a.out".into()
                }
            };

            transpile(&llvm_file, &prog)?;
            println!("Wrote LLVM IR to `{llvm_file}`.");

            let _compile_output = match Command::new("clang")
                .arg("-o")
                .arg(exe_file.clone())
                .arg(llvm_file)
                .output()
            {
                Ok(o) => o,
                Err(e) => return Err(format!("Could not compile LLVM IR to executable: {e}")),
            };
            println!("Wrote executable to `{exe_file}`.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod test;
