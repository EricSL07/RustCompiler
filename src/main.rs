use std::env;
use std::fs;
use std::path::Path;
use std::process;

use rust_tpp_compiler::myerror::MyError;
use rust_tpp_compiler::tpplexer::get_lexer;

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
    println!("  --lexer <ESTRATÉGIA>  Estratégia Léxica: ply, mandfa, symtable, symtableman, automatalib, automatalibman (padrão: ply)");
    println!("  --parser <ESTRATÉGIA> Estratégia Sintática: yacc, topdown (padrão: yacc)");
    println!("  --sema, --semantic    Estratégia Semântica: sema (padrão: default)");
    println!("  --gencode, --codegen  Estratégia de Geração de Código: llvm (padrão: llvm)");
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

    let mut lexer_strategy = String::from("ply");
    let mut _parser_strategy = String::from("yacc");
    let mut _sema_strategy = String::from("default");
    let mut _gencode_strategy = String::from("llvm");
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
            "--lexer" => {
                if i + 1 >= args.len() {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
                lexer_strategy = args[i + 1].clone();
                i += 2;
            }
            "--parser" => {
                if i + 1 >= args.len() {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
                _parser_strategy = args[i + 1].clone();
                i += 2;
            }
            "--sema" | "--semantic" => {
                if i + 1 >= args.len() {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
                _sema_strategy = args[i + 1].clone();
                i += 2;
            }
            "--gencode" | "--codegen" => {
                if i + 1 >= args.len() {
                    println!(
                        "{}",
                        ge.new_error(check_key, "ERR-GLOB-USE", None, None, &[], &[])
                    );
                    return;
                }
                _gencode_strategy = args[i + 1].clone();
                i += 2;
            }
            s if s.starts_with("--lexer=") => {
                lexer_strategy = s.trim_start_matches("--lexer=").to_string();
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
    let mut lexer = match get_lexer(&lexer_strategy, check_key) {
        Ok(l) => l,
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };

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

    // Esqueleto para as próximas fases do compilador:
    // Fase 2: Análise Sintática (tppparser)
    // Fase 3: Análise Semântica (tppsema)
    // Fase 4: Geração de Código (tppcodegen)
}
