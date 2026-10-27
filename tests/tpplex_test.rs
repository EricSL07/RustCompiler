use std::fs;
use std::path::PathBuf;
use std::process::Command;

use rust_tpp_compiler::tpplexer::ALL_STRATEGIES;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_compiler_case(lexer_type: &str, key_option: &str, input_file: &str) {
    let root = project_root();
    let bin = env!("CARGO_BIN_EXE_tpp_compiler");

    let mut cmd = Command::new(bin);
    cmd.current_dir(&root);
    cmd.arg("--lexer").arg(lexer_type);
    if !key_option.is_empty() {
        cmd.arg(key_option);
    }

    if !input_file.is_empty() {
        let path_file = format!("tests/lex-tests/{}", input_file);
        cmd.arg(&path_file);
    }

    let output = cmd
        .output()
        .expect("Falha ao executar o binário tpp_compiler");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let expected_path = root
        .join("tests/lex-tests")
        .join(format!("{}.lex.out", input_file));
    let expected_raw = fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Não foi possível ler {:?}: {}", expected_path, e));

    let generated_norm = stdout.replace("\r\n", "\n");
    let expected_norm = expected_raw.replace("\r\n", "\n");

    assert_eq!(
        generated_norm.trim(),
        expected_norm.trim(),
        "Divergência no teste [lexer={}, file='{}']\n--- Gerado ---\n{}\n--- Esperado ---\n{}",
        lexer_type,
        input_file,
        generated_norm.trim(),
        expected_norm.trim()
    );
}

fn active_lexer() -> String {
    std::env::var("TPP_LEXER").unwrap_or_else(|_| "ply".to_string())
}

macro_rules! lex_test_case {
    ($fn_name:ident, $file:expr) => {
        #[test]
        fn $fn_name() {
            let strat = active_lexer();
            if strat == "all" {
                for &s in ALL_STRATEGIES {
                    run_compiler_case(s, "-k", $file);
                }
            } else {
                run_compiler_case(&strat, "-k", $file);
            }
        }
    };
}

// 3 casos de erro de entrada global (equivalentes ao tpplex_test.py do professor)
lex_test_case!(test_execute_empty, "");
lex_test_case!(test_execute_teste_c, "teste.c");
lex_test_case!(test_execute_notexist_tpp, "notexist.tpp");

// 39 casos de teste .tpp do professor
lex_test_case!(test_execute_lex_test_001_tpp, "lex-test-001.tpp");
lex_test_case!(test_execute_lex_test_002_tpp, "lex-test-002.tpp");
lex_test_case!(test_execute_lex_test_003_tpp, "lex-test-003.tpp");
lex_test_case!(test_execute_lex_test_004_tpp, "lex-test-004.tpp");
lex_test_case!(test_execute_lex_test_005_tpp, "lex-test-005.tpp");
lex_test_case!(test_execute_lex_test_006_tpp, "lex-test-006.tpp");
lex_test_case!(test_execute_lex_test_007_tpp, "lex-test-007.tpp");
lex_test_case!(test_execute_lex_test_008_tpp, "lex-test-008.tpp");
lex_test_case!(test_execute_lex_test_009_tpp, "lex-test-009.tpp");
lex_test_case!(test_execute_lex_test_010_tpp, "lex-test-010.tpp");
lex_test_case!(test_execute_lex_test_011_tpp, "lex-test-011.tpp");
lex_test_case!(test_execute_lex_test_012_tpp, "lex-test-012.tpp");
lex_test_case!(test_execute_lex_test_013_tpp, "lex-test-013.tpp");
lex_test_case!(test_execute_lex_test_014_tpp, "lex-test-014.tpp");
lex_test_case!(test_execute_lex_test_015_tpp, "lex-test-015.tpp");
lex_test_case!(test_execute_lex_test_016_tpp, "lex-test-016.tpp");
lex_test_case!(test_execute_lex_test_017_tpp, "lex-test-017.tpp");
lex_test_case!(test_execute_lex_test_018_tpp, "lex-test-018.tpp");
lex_test_case!(test_execute_lex_test_019_tpp, "lex-test-019.tpp");
lex_test_case!(test_execute_lex_test_020_tpp, "lex-test-020.tpp");
lex_test_case!(test_execute_lex_test_021_tpp, "lex-test-021.tpp");
lex_test_case!(test_execute_lex_test_022_tpp, "lex-test-022.tpp");
lex_test_case!(test_execute_lex_test_023_tpp, "lex-test-023.tpp");
lex_test_case!(test_execute_lex_test_024_tpp, "lex-test-024.tpp");
lex_test_case!(test_execute_lex_test_025_tpp, "lex-test-025.tpp");
lex_test_case!(test_execute_lex_test_026_tpp, "lex-test-026.tpp");
lex_test_case!(test_execute_lex_test_027_tpp, "lex-test-027.tpp");
lex_test_case!(test_execute_lex_test_028_tpp, "lex-test-028.tpp");
lex_test_case!(test_execute_lex_test_029_tpp, "lex-test-029.tpp");
lex_test_case!(test_execute_lex_test_030_tpp, "lex-test-030.tpp");
lex_test_case!(test_execute_lex_test_031_tpp, "lex-test-031.tpp");
lex_test_case!(test_execute_lex_test_032_tpp, "lex-test-032.tpp");
lex_test_case!(test_execute_lex_test_033_tpp, "lex-test-033.tpp");
lex_test_case!(test_execute_lex_test_034_tpp, "lex-test-034.tpp");
lex_test_case!(test_execute_lex_test_035_tpp, "lex-test-035.tpp");
lex_test_case!(test_execute_lex_test_036_tpp, "lex-test-036.tpp");
lex_test_case!(test_execute_lex_test_037_tpp, "lex-test-037.tpp");
lex_test_case!(test_execute_lex_test_038_tpp, "lex-test-038.tpp");
lex_test_case!(test_execute_lex_test_039_tpp, "lex-test-039.tpp");

// Teste que valida todas as 6 estratégias léxicas contra todos os 42 casos de teste
#[test]
fn test_all_strategies_all_42_cases() {
    let files = [
        "",
        "teste.c",
        "notexist.tpp",
        "lex-test-001.tpp",
        "lex-test-002.tpp",
        "lex-test-003.tpp",
        "lex-test-004.tpp",
        "lex-test-005.tpp",
        "lex-test-006.tpp",
        "lex-test-007.tpp",
        "lex-test-008.tpp",
        "lex-test-009.tpp",
        "lex-test-010.tpp",
        "lex-test-011.tpp",
        "lex-test-012.tpp",
        "lex-test-013.tpp",
        "lex-test-014.tpp",
        "lex-test-015.tpp",
        "lex-test-016.tpp",
        "lex-test-017.tpp",
        "lex-test-018.tpp",
        "lex-test-019.tpp",
        "lex-test-020.tpp",
        "lex-test-021.tpp",
        "lex-test-022.tpp",
        "lex-test-023.tpp",
        "lex-test-024.tpp",
        "lex-test-025.tpp",
        "lex-test-026.tpp",
        "lex-test-027.tpp",
        "lex-test-028.tpp",
        "lex-test-029.tpp",
        "lex-test-030.tpp",
        "lex-test-031.tpp",
        "lex-test-032.tpp",
        "lex-test-033.tpp",
        "lex-test-034.tpp",
        "lex-test-035.tpp",
        "lex-test-036.tpp",
        "lex-test-037.tpp",
        "lex-test-038.tpp",
        "lex-test-039.tpp",
    ];
    for &strat in ALL_STRATEGIES {
        for &file in &files {
            run_compiler_case(strat, "-k", file);
        }
    }
}
