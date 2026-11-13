use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rust_tpp_compiler::tpplexer::{BaseLexer, LogosLexer};
use rust_tpp_compiler::tppparser::ast::AstNode;
use rust_tpp_compiler::tppparser::parser::Parser;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn parse_expr(input: &str, check_key: bool) -> Result<AstNode, String> {
    let mut lexer = LogosLexer::new(check_key);
    let tokens = lexer.tokenize(input);
    let mut parser = Parser::new(tokens, check_key);
    parser.parse()
}

fn run_parser_cli(args: &[&str]) -> String {
    let root = project_root();
    let bin = env!("CARGO_BIN_EXE_tpp_compiler");

    let output = Command::new(bin)
        .current_dir(&root)
        .args(args)
        .output()
        .expect("Falha ao executar o binário tpp_compiler");

    String::from_utf8_lossy(&output.stdout)
        .replace("\r\n", "\n")
        .trim()
        .to_string()
}

#[test]
fn testa_precedencia_aritmetica() {
    let ast = parse_expr("1 + 2 * 3", true).expect("Deveria fazer o parse sem erro");
    assert_eq!(ast.node_type, "ExpressionAdditive");
    assert_eq!(ast.children.len(), 3);
    assert_eq!(ast.children[2].node_type, "ExpressionMultiplicative");
}

#[test]
fn testa_parenteses_e_unario() {
    let ast = parse_expr("(1 + 2) * -3", true).expect("Deveria aceitar parênteses e unário");
    assert_eq!(ast.node_type, "ExpressionMultiplicative");
    assert_eq!(ast.children[1].node_type, "ExpressionAdditive");
    assert_eq!(ast.children[2].node_type, "ExpressionUnary");
}

#[test]
fn testa_ponto_flutuante() {
    let ast = parse_expr("3.14 + 2.0 / 4", true).expect("Deveria aceitar ponto flutuante");
    assert_eq!(ast.node_type, "ExpressionAdditive");
    assert_eq!(ast.children[2].node_type, "ExpressionMultiplicative");
}

#[test]
fn testa_erro_fim_inesperado() {
    let err = parse_expr("1 +", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-EOF-INESPERADO");
}

#[test]
fn testa_erro_parentese_nao_fechado() {
    let err = parse_expr("(1 + 2", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-IRRECUPERAVEL");
}

#[test]
fn testa_erro_token_sobrando() {
    let err = parse_expr("1 2", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-EOF-INESPERADO");
}

#[test]
fn testa_cli_erros_globais_com_parser() {
    assert_eq!(run_parser_cli(&["--parser", "on", "-k"]), "ERR-GLOB-USE");
    assert_eq!(
        run_parser_cli(&["--parser", "on", "-k", "tests/lex-tests/teste.c"]),
        "ERR-GLOB-NOT-TPP"
    );
    assert_eq!(
        run_parser_cli(&["--parser", "on", "-k", "tests/lex-tests/notexist.tpp"]),
        "ERR-GLOB-FILE-NOT-EXISTS"
    );
}

#[test]
fn testa_cli_parser_sucesso_expressao() {
    let temp_file = std::env::temp_dir().join("tpp_parser_smoke_test.tpp");
    fs::write(&temp_file, "1 + 2 * 3").expect("Falha ao criar arquivo .tpp temporário");

    let path_str = temp_file.to_string_lossy().to_string();
    let stdout = run_parser_cli(&["--parser", "on", "-k", &path_str]);

    let _ = fs::remove_file(&temp_file);
    assert_eq!(stdout, "WAR-SYN-ANA-SUCCESS");
}
