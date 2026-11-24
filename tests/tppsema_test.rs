use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rust_tpp_compiler::tpplexer::{BaseLexer, LogosLexer};
use rust_tpp_compiler::tppparser::parser::Parser;
use rust_tpp_compiler::tppsemantic::sema::{self, SemaResult};

fn analyze_src(src: &str, check_key: bool) -> SemaResult {
    let mut lexer = LogosLexer::new(check_key);
    let tokens = lexer.tokenize(src);
    let mut parser = Parser::new(tokens, check_key);
    let ast = parser.parse().expect("AST deveria ser gerada");
    sema::analyze(&ast, check_key)
}

fn run_cli(args: &[&str]) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
fn cli_sema_programa_valido_sem_erros() {
    let tmp_path = std::env::temp_dir().join("tpp_sema_ok_test.tpp");
    let src = "\
inteiro: g := 10

inteiro dobro(inteiro: n)
    retorna(n * 2)
fim

inteiro principal()
    inteiro: res
    res := dobro(g)
    escreva(res)
    retorna(0)
fim
";
    fs::write(&tmp_path, src).unwrap();

    let out = run_cli(&["-k", "--sema", "on", tmp_path.to_str().unwrap()]);
    let _ = fs::remove_file(&tmp_path);

    assert_eq!(out, "");
}

#[test]
fn cli_sema_reporta_chaves_e_mensagens_formatadas() {
    let tmp_path = std::env::temp_dir().join("tpp_sema_err_test.tpp");
    let src = "\
inteiro foo()
    inteiro: x
    x := y
    retorna(x)
fim
";
    fs::write(&tmp_path, src).unwrap();

    let out_keys = run_cli(&["-k", "--sema", "on", tmp_path.to_str().unwrap()]);
    assert!(out_keys.contains("ERR-SEM-VAR-NOT-DECL"));
    assert!(out_keys.contains("ERR-SEM-MAIN-NOT-DECL"));
    assert!(out_keys.contains("WAR-SEM-FUNC-DECL-NOT-USED"));

    let out_human = run_cli(&["--sema", "on", tmp_path.to_str().unwrap()]);
    let _ = fs::remove_file(&tmp_path);

    assert!(out_human.contains("Variável 'y' não declarada."));
    assert!(out_human.contains("Função 'principal' não declarada."));
    assert!(out_human.contains("Função 'foo' declarada, mas não utilizada."));
}

#[test]
fn sema_valida_chamadas_principal_e_aridade() {
    let src = "\
inteiro soma(inteiro: a, inteiro: b)
    retorna(a + b)
fim

inteiro outra()
    retorna(principal())
fim

inteiro principal()
    inteiro: r
    r := soma(1)
    r := soma(1, 2, 3)
    r := nao_existe()
    r := principal()
    escreva(r)
    retorna(0)
fim
";
    let res = analyze_src(src, true);
    assert!(res.has_errors);
    assert!(res.messages.contains(&"ERR-SEM-CALL-FUNC-MAIN-NOT-ALLOWED".to_string()));
    assert!(res.messages.contains(&"WAR-SEM-CALL-REC-FUNC-MAIN".to_string()));
    assert!(res.messages.contains(&"ERR-SEM-CALL-FUNC-WITH-FEW-ARGS".to_string()));
    assert!(res.messages.contains(&"ERR-SEM-CALL-FUNC-WITH-MANY-ARGS".to_string()));
    assert!(res.messages.contains(&"ERR-SEM-CALL-FUNC-NOT-DECL".to_string()));
    assert!(res.messages.contains(&"WAR-SEM-FUNC-DECL-NOT-USED".to_string()));
}

#[test]
fn sema_valida_comparacao_encadeada_e_tupla_invalida() {
    let src = "\
inteiro principal()
    inteiro: a
    a := 1 < 2 < 3
    a := (1, 2) + 3
    escreva(a)
    retorna(0)
fim
";
    let res = analyze_src(src, true);
    assert!(res.has_errors);
    assert!(res.messages.contains(&"WAR-SEM-REL-CHAIN".to_string()));
    assert!(res.messages.contains(&"ERR-SEM-TUPLE-NOT-ALLOWED".to_string()));
}

#[test]
fn sema_gera_ast_podada_e_anotada() {
    let src = "\
inteiro principal()
    inteiro: a
    a := 1 + 2.5
    escreva(a)
    retorna(0)
fim
";
    let res = analyze_src(src, true);
    let ascii = res.pruned_ast.to_ascii_tree();

    assert!(ascii.contains("ExpressionAdditive [flutuante]"));
    assert!(ascii.contains("Literal: 1 [inteiro]"));
    assert!(ascii.contains("Literal: 2.5 [flutuante]"));
    assert!(ascii.contains("Op: +"));
    assert!(ascii.contains("Var [inteiro]"));
    assert!(!ascii.contains("ListaParametros"));
}

#[test]
fn cli_sem_tests_do_professor() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sem_dir = root.join("tests").join("sem-tests");

    let mut files: Vec<PathBuf> = fs::read_dir(&sem_dir)
        .expect("Falha ao ler tests/sem-tests")
        .filter_map(|e| e.ok().map(|entry| entry.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("tpp"))
        .collect();
    files.sort();

    assert!(!files.is_empty(), "Nenhum arquivo .tpp encontrado em tests/sem-tests");

    let mut failures = Vec::new();
    for tpp_path in &files {
        let out_path = PathBuf::from(format!("{}.sem.out", tpp_path.display()));
        if !out_path.exists() {
            continue;
        }
        let mut expected_lines: Vec<String> = fs::read_to_string(&out_path)
            .unwrap()
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        expected_lines.sort();

        let mut actual_lines: Vec<String> = run_cli(&["--sema", "on", "-k", tpp_path.to_str().unwrap()])
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        actual_lines.sort();

        if actual_lines != expected_lines {
            let name = tpp_path.file_name().unwrap().to_string_lossy();
            failures.push(format!(
                "=== {} ===\nEsperado:\n{}\n\nObtido:\n{}",
                name,
                expected_lines.join("\n"),
                actual_lines.join("\n")
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} de {} testes semânticos divergiram:\n\n{}",
        failures.len(),
        files.len(),
        failures.join("\n\n")
    );
}
