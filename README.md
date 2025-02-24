# Rust-bf

Interpreter for the esoteric programming language [brainfuck](https://en.wikipedia.org/wiki/Brainfuck) written in rust.

## Quickstart

Run interpreter

```terminal
$ cargo run interpret ./bf_code/fizz_buzz.bf
```

Transpile brainfuck code to LLVM IR and compile to native executable

```terminal
$ cargo run transpile ./bf_code/fizz_buzz.bf
$ ./bf_code/fizz_buzz
```
