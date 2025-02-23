use std::fs::File;
use std::io::prelude::*;

mod interpreter;
use interpreter::*;

// fn transpile(filename: String, prog: &Program) -> Result<(), String> {
//     let mut file = match File::create(filename.clone()) {
//         Ok(f) => f,
//         Err(err) => return Err(format!("Could not open file `{filename}`: {err}")),
//     };
//
//     // .global _main
//     // .align 4
//     //
//     // .text
//     // _main:
//     writeln!(file, ".global _main").unwrap();
//     writeln!(file, ".align 4").unwrap();
//     writeln!(file, "").unwrap();
//     writeln!(file, ".text").unwrap();
//     writeln!(file, "_main:").unwrap();
//
//     writeln!(file, "  ; Get address of buffer and save it to x0").unwrap();
//     writeln!(file, "  adrp x0, buffer@PAGE").unwrap();
//     writeln!(file, "  add  x0, x0, buffer@PAGEOFF").unwrap();
//     writeln!(file, "").unwrap();
//
//     for (i, token) in prog.instr.iter().enumerate() {
//         match token {
//             Token::IncPtr(count) => {
//                 writeln!(file, "  add  x0, x0, #{count}").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::DecPtr(count) => {
//                 writeln!(file, "  sub  x0, x0, #{count}").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::IncByte(count) => {
//                 writeln!(file, "  ldr  x1, [x0]").unwrap();
//                 writeln!(file, "  add  x1, x1, #{count}").unwrap();
//                 writeln!(file, "  str  x1, [x0]").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::DecByte(count) => {
//                 writeln!(file, "  ldr  x1, [x0]").unwrap();
//                 writeln!(file, "  sub  x1, x1, #{count}").unwrap();
//                 writeln!(file, "  str  x1, [x0]").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::Output => {
//                 writeln!(file, "  str  x0, [sp]").unwrap();
//                 writeln!(file, "  mov  x1, x0").unwrap();
//                 writeln!(file, "  mov  x0, #1").unwrap();
//                 writeln!(file, "  mov  x2, #1").unwrap();
//                 writeln!(file, "  mov  x16, #4").unwrap();
//                 writeln!(file, "  svc  0").unwrap();
//                 writeln!(file, "  ldr  x0, [sp]").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::Input => todo!("Input"),
//             Token::JumpZero(jump_addr) => {
//                 writeln!(file, "  ldr  x1, [x0]").unwrap();
//                 writeln!(file, "  cmp  x1, #0").unwrap();
//                 writeln!(file, "  b.eq label{jump_addr}").unwrap();
//                 writeln!(file, "label{i}:").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//             Token::JumpNonZero(jump_addr) => {
//                 writeln!(file, "  ldr  x1, [x0]").unwrap();
//                 writeln!(file, "  cmp  x1, #0").unwrap();
//                 writeln!(file, "  b.ne label{jump_addr}").unwrap();
//                 writeln!(file, "label{i}:").unwrap();
//                 writeln!(file, "").unwrap();
//             }
//         }
//     }
//
//     writeln!(file, "  ; Exit syscall").unwrap();
//     writeln!(file, "  mov  x0, #0").unwrap();
//     writeln!(file, "  mov  x16, #1").unwrap();
//     writeln!(file, "  svc  0").unwrap();
//
//     writeln!(file, "").unwrap();
//     writeln!(file, ".data").unwrap();
//     writeln!(file, "buffer: .zero 30000").unwrap();
//
//     Ok(())
// }

fn transpile(filename: &String, prog: &Program) -> Result<(), String> {
    if !filename.ends_with(".ll") {
        return Err(format!("filename must end with `.ll` but is `{filename}`"));
    }

    let mut file = match File::create(filename.clone()) {
        Ok(f) => f,
        Err(err) => return Err(format!("Could not open file `{filename}`: {err}")),
    };

    const BUFFER_SIZE: i32 = 1024;
    writeln!(
        file,
        "@buffer = internal global [{BUFFER_SIZE} x i32] zeroinitializer, align 4"
    )
    .unwrap();
    writeln!(file, "").unwrap();

    // What is the correct symbol name for stdout?
    const STDOUT_SYMBOL_NAME: &str = "@__stdoutp";
    writeln!(file, "{STDOUT_SYMBOL_NAME} = external global ptr, align 8").unwrap();
    const STDIN_SYMBOL_NAME: &str = "@__stdinp";
    writeln!(file, "{STDIN_SYMBOL_NAME} = external global ptr, align 8").unwrap();
    writeln!(file, "").unwrap();

    writeln!(file, "define i32 @main() {{").unwrap();
    writeln!(file, "  ; Get address of buffer and save it to ptr").unwrap();
    writeln!(file, "  %data_ptr = alloca ptr, align 8").unwrap();
    writeln!(file, "  store ptr @buffer, ptr %data_ptr, align 8").unwrap();
    writeln!(file, "").unwrap();

    let mut var_counter: i32 = 1;
    for (this_addr, token) in prog.instr.iter().enumerate() {
        match token {
            Token::IncPtr(count) => {
                writeln!(file, "  ; Advance %data_ptr by {count}").unwrap();
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = getelementptr inbounds i32, ptr %{}, i64 {count}",
                    var_counter - 1
                )
                .unwrap();
                writeln!(file, "  store ptr %{var_counter}, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::DecPtr(count) => {
                writeln!(file, "  ; Advance %data_ptr by -{count}").unwrap();
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = getelementptr inbounds i32, ptr %{}, i64 -{count}",
                    var_counter - 1
                )
                .unwrap();
                writeln!(file, "  store ptr %{var_counter}, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::IncByte(count) => {
                writeln!(file, "  ; Increment value at %data_ptr by {count}").unwrap();
                let ptr_num = var_counter;
                writeln!(file, "  %{ptr_num} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(file, "  %{var_counter} = load i32, ptr %{ptr_num}, align 4").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = add nsw i32 %{}, {count}",
                    var_counter - 1
                )
                .unwrap();
                writeln!(file, "  store i32 %{var_counter}, ptr %{ptr_num}, align 4").unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::DecByte(count) => {
                writeln!(file, "  ; Decrement value at %data_ptr by {count}").unwrap();
                let ptr_num = var_counter;
                writeln!(file, "  %{ptr_num} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(file, "  %{var_counter} = load i32, ptr %{ptr_num}, align 4").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = sub nsw i32 %{}, {count}",
                    var_counter - 1
                )
                .unwrap();
                writeln!(file, "  store i32 %{var_counter}, ptr %{ptr_num}, align 4").unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::Output => {
                writeln!(file, "  ; Print value at %data_ptr").unwrap();
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = load i32, ptr %{}, align 4",
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = load ptr, ptr {STDOUT_SYMBOL_NAME}, align 8"
                )
                .unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = call i32 @putc(i32 %{}, ptr %{})",
                    var_counter - 2,
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::Input => {
                writeln!(file, "  ; Read value from stdin into %data_ptr").unwrap();
                writeln!(
                    file,
                    "  %{var_counter} = load ptr, ptr {STDIN_SYMBOL_NAME}, align 8"
                )
                .unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = call i32 @getc(ptr %{})",
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                writeln!(
                    file,
                    "  store i32 %{}, ptr %{var_counter}, align 4",
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(file, "").unwrap();
            }

            Token::JumpZero(jump_addr) => {
                writeln!(
                    file,
                    "  ; Jump to l{jump_addr} if value at %data_ptr is zero"
                )
                .unwrap();
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = load i32, ptr %{}, align 4",
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = icmp eq i32 %{}, 0",
                    var_counter - 1
                )
                .unwrap();
                writeln!(
                    file,
                    "  br i1 %{var_counter}, label %l{jump_addr}, label %l{this_addr}"
                )
                .unwrap();
                var_counter += 1;
                writeln!(file, "l{this_addr}:").unwrap();
                writeln!(file, "").unwrap();
            }

            Token::JumpNonZero(jump_addr) => {
                writeln!(
                    file,
                    "  ; Jump to l{jump_addr} if value at %data_ptr is non-zero"
                )
                .unwrap();
                writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8").unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = load i32, ptr %{}, align 4",
                    var_counter - 1
                )
                .unwrap();
                var_counter += 1;
                writeln!(
                    file,
                    "  %{var_counter} = icmp ne i32 %{}, 0",
                    var_counter - 1
                )
                .unwrap();
                writeln!(
                    file,
                    "  br i1 %{var_counter}, label %l{jump_addr}, label %l{this_addr}"
                )
                .unwrap();
                var_counter += 1;
                writeln!(file, "l{this_addr}:").unwrap();
                writeln!(file, "").unwrap();
            }
        }
    }

    writeln!(file, "  ret i32 0").unwrap();
    writeln!(file, "}}").unwrap();
    writeln!(file, "").unwrap();
    writeln!(file, "declare i32 @putc(i32, ptr)").unwrap();
    writeln!(file, "declare i32 @getc(ptr)").unwrap();

    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err(format!(
            "Usage: {} <input file> [output file]: No input provided.",
            args[0]
        ));
    }

    let input_file = &args[1];
    let output_file = if args.len() > 2 {
        args[2].clone()
    } else if args[1].ends_with(".bf") {
        args[1].strip_suffix(".bf").unwrap().to_owned() + ".ll"
    } else {
        args[1].clone() + ".ll"
    };
    // println!("Input file: {}", input_file);
    // println!("Output file: {}", output_file);

    let mut lexer = Lexer::new(input_file)?;

    let mut prog = Program::new();
    prog.tokenize(&mut lexer)?;

    // let mut inter = Interpreter::new();
    // inter.run(&prog);

    transpile(&output_file, &prog)?;
    println!("Wrote LLVM IR to `{output_file}`.");

    Ok(())
}
