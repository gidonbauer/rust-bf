use std::fs::File;
use std::io::prelude::*;
use std::io::Error;

use super::tokenizer::*;

fn increment_data_pointer(file: &mut File, count: i32, mut var_counter: i32) -> Result<i32, Error> {
    writeln!(file, "  ; Advance %data_ptr by {count}")?;
    writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8")?;
    var_counter += 1;
    writeln!(
        file,
        "  %{var_counter} = getelementptr inbounds i32, ptr %{}, i64 {count}",
        var_counter - 1
    )?;
    writeln!(file, "  store ptr %{var_counter}, ptr %data_ptr, align 8")?;
    var_counter += 1;
    writeln!(file, "")?;

    Ok(var_counter)
}

fn increment_value(file: &mut File, count: i32, mut var_counter: i32) -> Result<i32, Error> {
    writeln!(file, "  ; Increment value at %data_ptr by {count}")?;
    let ptr_num = var_counter;
    writeln!(file, "  %{ptr_num} = load ptr, ptr %data_ptr, align 8")?;
    var_counter += 1;
    writeln!(file, "  %{var_counter} = load i32, ptr %{ptr_num}, align 4")?;
    var_counter += 1;
    writeln!(
        file,
        "  %{var_counter} = add nsw i32 %{}, {count}",
        var_counter - 1
    )?;
    writeln!(file, "  store i32 %{var_counter}, ptr %{ptr_num}, align 4")?;
    var_counter += 1;
    writeln!(file, "")?;

    Ok(var_counter)
}

fn conditional_jump(
    file: &mut File,
    eq_zero: bool,
    jump_addr: usize,
    this_addr: usize,
    mut var_counter: i32,
) -> Result<i32, Error> {
    writeln!(
        file,
        "  ; Jump to l{jump_addr} if value at %data_ptr is {}",
        if eq_zero { "zero" } else { "non-zero" }
    )?;
    writeln!(file, "  %{var_counter} = load ptr, ptr %data_ptr, align 8")?;
    var_counter += 1;
    writeln!(
        file,
        "  %{var_counter} = load i32, ptr %{}, align 4",
        var_counter - 1
    )?;
    var_counter += 1;
    writeln!(
        file,
        "  %{var_counter} = icmp {} i32 %{}, 0",
        if eq_zero { "eq" } else { "ne" },
        var_counter - 1
    )?;
    writeln!(
        file,
        "  br i1 %{var_counter}, label %l{jump_addr}, label %l{this_addr}"
    )?;
    var_counter += 1;
    writeln!(file, "l{this_addr}:")?;
    writeln!(file, "")?;

    Ok(var_counter)
}

pub fn transpile(filename: &String, prog: &Program) -> Result<(), String> {
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
                assert!(*count < i32::MAX as usize);
                var_counter = increment_data_pointer(&mut file, *count as i32, var_counter)
                    .map_err(|e| e.to_string())?;
            }
            Token::DecPtr(count) => {
                assert!(*count < i32::MAX as usize);
                var_counter = increment_data_pointer(&mut file, -(*count as i32), var_counter)
                    .map_err(|e| e.to_string())?;
            }

            Token::IncByte(count) => {
                var_counter = increment_value(&mut file, *count as i32, var_counter)
                    .map_err(|e| e.to_string())?;
            }
            Token::DecByte(count) => {
                var_counter = increment_value(&mut file, -(*count as i32), var_counter)
                    .map_err(|e| e.to_string())?;
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
                var_counter = conditional_jump(&mut file, true, *jump_addr, this_addr, var_counter)
                    .map_err(|e| e.to_string())?;
            }
            Token::JumpNonZero(jump_addr) => {
                var_counter =
                    conditional_jump(&mut file, false, *jump_addr, this_addr, var_counter)
                        .map_err(|e| e.to_string())?;
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
