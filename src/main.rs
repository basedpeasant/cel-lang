use std::fs;

mod util;
mod tokenize;
mod ast;
mod codegen;

const PROGRAM_NAME: &str = "cel";

fn read_src(filepath: &str) -> String {
    match fs::read_to_string(filepath) {
        Ok(src) => src,
        Err(e) => panic!("Could not find file at \"{}\": ({})", filepath, e)
    }
}

fn print_usage() {
    println!("Usage: {} FILENAME", PROGRAM_NAME);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        print_usage();
        std::process::exit(0);
    }
    let src = read_src(&args[1]);
    let tokens = tokenize::tokenize_start(&src);
    tokenize::print_tokens(&tokens);
    let ast = ast::ast_start(tokens);
    codegen::codegen_start(&ast);
}
