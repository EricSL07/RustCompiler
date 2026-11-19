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
    parser.parse_expressao_completa()
}

fn parse_program(input: &str, check_key: bool) -> Result<AstNode, String> {
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
fn testa_ponto_flutuante_e_notacao_cientifica() {
    let ast = parse_expr("3.14 + 1.5e-2 / 4", true).expect("Deveria aceitar float e notação científica");
    assert_eq!(ast.node_type, "ExpressionAdditive");
    assert_eq!(ast.children[2].node_type, "ExpressionMultiplicative");
}

#[test]
fn testa_expressao_relacional_e_logica() {
    let ast = parse_expr("!a && b + 1 < c * 2 || d = 0", true)
        .expect("Deveria aceitar operadores lógicos, relacionais e negação");
    assert_eq!(ast.node_type, "ExpressionLogic");
}

#[test]
fn testa_erro_fim_inesperado() {
    let err = parse_expr("1 +", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-EOF-INESPERADO");
}

#[test]
fn testa_erro_parentese_nao_fechado() {
    let err = parse_expr("(1 + 2", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-EOF-INESPERADO");

    let err_token = parse_expr("(1 + 2]", true).unwrap_err();
    assert_eq!(err_token, "ERR-SYN-IRRECUPERAVEL");
}

#[test]
fn testa_erro_token_sobrando() {
    let err = parse_expr("1 2", true).unwrap_err();
    assert_eq!(err, "ERR-SYN-EOF-INESPERADO");
}

#[test]
fn testa_programa_completo_basico() {
    let code = r#"
inteiro: a, b, c

inteiro principal()
    leia(a)
    leia(b)
    c := a + b * 2
    se c > 10 então
        escreva(c)
    senão
        escreva(0)
    fim
    retorna(0)
fim
"#;
    let ast = parse_program(code, true).expect("Programa válido deveria passar");
    assert_eq!(ast.node_type, "Programa");
    assert_eq!(ast.children.len(), 2);
}

#[test]
fn testa_programa_funcao_multiplo_retorno_arrays_repita() {
    let code = r#"
(inteiro, flutuante) calcula(inteiro: v[], flutuante: x)
    inteiro: i := 0
    repita
        v[i] := i + 1
        i := i + 1
    até i = 10
    retorna(i, x)
fim

principal()
    inteiro: dados[10]
    (a, b) := calcula(dados, 2.5)
fim
"#;
    let ast = parse_program(code, true).expect("Programa com múltiplo retorno, arrays e repita deveria passar");
    assert_eq!(ast.node_type, "Programa");
    assert_eq!(ast.children.len(), 2);
}

#[test]
fn testa_erro_indice_nao_fechado() {
    let code = r#"
inteiro principal()
    v[0 := 1
fim
"#;
    let err = parse_program(code, true).unwrap_err();
    assert_eq!(err, "ERR-SYN-INDICE");
}

#[test]
fn testa_erro_leia_invalido() {
    let code = r#"
inteiro principal()
    leia(123)
fim
"#;
    let err = parse_program(code, true).unwrap_err();
    assert_eq!(err, "ERR-SYN-LEIA");
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
fn testa_cli_parser_sucesso_programa() {
    let temp_file = std::env::temp_dir().join("tpp_parser_smoke_test.tpp");
    let code = "inteiro: a, b\ninteiro principal()\n  a := b + 1\n  retorna(0)\nfim\n";
    fs::write(&temp_file, code).expect("Falha ao criar arquivo .tpp temporário");

    let path_str = temp_file.to_string_lossy().to_string();
    let stdout = run_parser_cli(&["--parser", "on", "-k", &path_str]);

    let _ = fs::remove_file(&temp_file);
    assert_eq!(stdout, "WAR-SYN-ANA-SUCCESS");
}

#[test]
fn testa_syn_tests_validos_do_professor() {
    let root = project_root();
    let syn_dir = root.join("tests/syn-tests");
    let valid_files = [
        "syn-test-003.tpp",
        "syn-test-005.tpp",
        "syn-test-006.tpp",
        "syn-test-007.tpp",
        "syn-test-017.tpp",
        "syn-test-020.tpp",
        "syn-test-021.tpp",
        "syn-test-022.tpp",
        "syn-test-023.tpp",
        "syn-test-026.tpp",
    ];

    for file in valid_files {
        let tpp_path = format!("tests/syn-tests/{}", file);
        let stdout = run_parser_cli(&["--parser", "on", "-k", "-t", &tpp_path]);

        let _ = fs::remove_file(syn_dir.join(format!("{}.ast.dot", file)));
        let _ = fs::remove_file(syn_dir.join(format!("{}.ascii_tree.txt", file)));

        let expected = fs::read_to_string(syn_dir.join(format!("{}.syn.out", file)))
            .expect("Falha ao ler .syn.out")
            .replace("\r\n", "\n")
            .trim()
            .to_string();

        assert_eq!(stdout, expected, "Falha no teste válido {}", file);
    }
}
