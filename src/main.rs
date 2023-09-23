mod lexer;
mod ast;
mod asm;
mod typecheck;

use argin::Argin;
use std::process;

fn cli() -> Argin {
    let mut args = Argin::new();
    args.add_positional_arg();
    args.add_positional_arg();
    args.add_flag("-r");
    return args.parse();
}

fn help() {
    println!("Usage: ./langin [FILE] [OPTIONS]");
    println!("    -r: run the final executable");
}

fn error(error: &Box<dyn std::error::Error>) -> bool {
    println!("[ERROR]: {}", error.to_string());
    process::exit(1);
}

fn error_no_log(error: &Box<dyn std::error::Error>) -> bool {
    println!("{}", error.to_string());
    process::exit(1);
}

fn main() {
    let args = cli();
    let file = match args.pos_arg.get(1) {
        Some(arg) => arg,
        None => {
            help();
            process::exit(1);
        },
    };

    println!("[INFO]: lexing `{}`", file);
    let tokens = match lexer::lex(file) {
        Ok(tokens) => tokens,
        Err(error) => {
            println!("[ERROR] `{}`: {}", file, error.to_string());
            process::exit(1);
        },
    };

    println!("[INFO]: parsing `{}`", file);
    let parsed = ast::Ast::parse(&tokens);
    if let Err(error) = parsed {
        println!("{}", error.to_string());
        process::exit(1);
    }
    let parsed = parsed.unwrap();

    //println!("ast: {:#?}", parsed);

    println!("[INFO]: type checking");
    let mut typechecker = typecheck::TypeChecker::new();
    let _ = typechecker.check(&parsed, false).is_err_and(|err| error_no_log(&err));

    println!("[INFO]: generating linux-x86_64-fasm");
    let mut codegen = match asm::CodeGen::new(&file) {
        Ok(codegen) => codegen,
        Err(error) => {
            println!("[ERROR] `{}`: {}", file, error.to_string());
            process::exit(1);
        },
    };

    let _ = codegen.generate(&parsed, true).is_err_and(|err| error(&err));

    // flush the buffer
    let _ = codegen.flush().is_err_and(|err| error(&err));

    let output = codegen.assemble();
    let _ = output.as_ref().is_err_and(|err| error(err));
    println!("[FASM]:\n{}", output.unwrap());

    println!("[INFO]: compilation done");

    if args.flags.contains(&"-r".to_string()) {
        let _ = codegen.run().is_err_and(|err| error_no_log(&err));
    }
}


