use std::env;
use std::fs;
use std::path::Path;

use rust_tpp_compiler::myerror::MyError;
use rust_tpp_compiler::tpplexer::{BaseLexer, LogosLexer};
use rust_tpp_compiler::tppparser::parser::Parser;

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
    print!("  -t                     Gera a árvore de sintaxe abstrata (AST) e a imprime no console.\n");
    print!("  --parser on|off        Habilitar ou desabilitar a fase de análise sintática (padrão: on)\n");
    print!("  --lexer on|off         Habilitar ou desabilitar a fase de análise léxica (padrão: on)\n");
    println!();
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

    let mut run_parser = false;
    let mut run_lexer = true;

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
            "-t" => {
                run_parser = true;
                i += 1;
            }
            "--parser" => {
                i += 1;
                if i < args.len() {
                    let parser_arg = &args[i];
                    if parser_arg == "on" || parser_arg == "yacc" {
                        run_parser = true;
                    } else if parser_arg == "off" {
                        run_parser = false;
                    } else {
                        println!(
                            "{}",
                            ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                        );
                        return;
                    }
                } else {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
                i += 1;
            }
            "--lexer" => {
                i += 1;
                if i < args.len() {
                    let lexer_arg = &args[i];
                    if lexer_arg == "on" {
                        run_lexer = true;
                    } else if lexer_arg == "off" {
                        run_lexer = false;
                    } else {
                        println!(
                            "{}",
                            ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                        );
                        return;
                    }
                } else {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
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

    // Fase 2: Análise Sintática
    if !run_parser {
        if run_lexer {
            lexer.input(&source_file);
            while let Some(token) = lexer.token() {
                println!("{}", token.token_type);
            }
        }
        return;
    }

    let tokens = lexer.tokenize(&source_file);
    let mut parser = Parser::new(tokens, check_key);
    match parser.parse() {
        Ok(_ast) => {
            let pe = MyError::new("ParserErrors");
            println!("{}", pe.new_error(check_key, "WAR-SYN-ANA-SUCCESS", None, None, &[], &[]));
        }
        Err(err) => println!("{}", err),
    }


}
