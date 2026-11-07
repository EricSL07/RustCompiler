use std::env;
use std::fs;
use std::path::Path;

use rust_tpp_compiler::myerror::MyError;
use rust_tpp_compiler::tpplexer::{BaseLexer, LogosLexer};

fn print_help() {
    println!("Compilador TPP em Rust [tppc]");
    println!();
    println!("Uso: tpp_compiler [OPÇÕES] <fonte>");
    println!();
    println!("Argumentos posicionais:");
    println!("  fonte                 arquivo-fonte .tpp");
    println!();
    println!("Opções:");
    println!("  -h, --help            Exibe esta mensagem de ajuda e sai");
    println!("  -k                    Imprimir apenas as chaves de Erro.");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut check_key = false;
    for arg in &args {
        if arg == "-k" {
            check_key = true;
        } else if arg == "-h" || arg == "--help" {
            print_help();
            return;
        }
    }

    let mut fonte: Option<String> = None;

    let ge = MyError::new("GlobalErrors");

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        match arg.as_str() {
            "-k" => {
                check_key = true;
                i += 1;
            }
            s if s.starts_with('-') => {
                println!(
                    "{}",
                    ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                );
                return;
            }
            positional => {
                fonte = Some(positional.to_string());
                i += 1;
            }
        }
    }

    let Some(fonte_path) = fonte else {
        println!(
            "{}",
            ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
        );
        return;
    };

    if fonte_path.trim().is_empty() {
        println!(
            "{}",
            ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
        );
        return;
    }

    if !fonte_path.ends_with(".tpp") {
        println!(
            "{}",
            ge.new_error(check_key, "ERR-GLOB-NOT-TPP", None, None, &[], &[])
        );
        return;
    }

    if !Path::new(&fonte_path).exists() {
        println!(
            "{}",
            ge.new_error(check_key, "ERR-GLOB-FILE-NOT-EXISTS", None, None, &[], &[])
        );
        return;
    }

    // Fase 1: Análise Léxica
    let mut lexer = LogosLexer::new(check_key);

    let source_file = match fs::read_to_string(&fonte_path) {
        Ok(content) => content,
        Err(_) => {
            println!(
                "{}",
                ge.new_error(check_key, "ERR-GLOB-FILE-NOT-EXISTS", None, None, &[], &[])
            );
            return;
        }
    };

    lexer.input(&source_file);
    while let Some(token) = lexer.token() {
        println!("{}", token.token_type);
    }

}
