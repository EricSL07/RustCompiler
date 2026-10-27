use std::time::Instant;

use rust_tpp_compiler::tpplexer::{get_lexer, ALL_STRATEGIES};

const BENCHMARK_BLOCKS: usize = 500;

fn generate_large_tpp_code(num_blocks: usize) -> String {
    let mut blocks = Vec::with_capacity(num_blocks);
    for i in 0..num_blocks {
        blocks.push(format!(
            "
        {{ Bloco estruturado de processamento e stress léxico #{i} }}
        inteiro: cont_{i}, limite_{i}, acum_{i}
        flutuante: taxa_{i}, delta_{i}, total_{i}
        
        cont_{i} := 0
        acum_{i} := {} + 42
        taxa_{i} := 1.75e-3
        delta_{i} := 0.0052
        limite_{i} := {i} + 50
        
        repita
            se cont_{i} >= 10 então
                total_{i} := taxa_{i} * 3.14159 + cont_{i} / 2.0 - delta_{i}
            senão
                total_{i} := delta_{i} * 1.5e2
            fim
            cont_{i} := cont_{i} + 1
            acum_{i} := acum_{i} * 2 - 1
        até cont_{i} = limite_{i}
        ",
            i * 7
        ));
    }
    blocks.join("\n")
}

#[test]
fn test_large_token_sequence_performance_and_conformance() {
    let code = generate_large_tpp_code(BENCHMARK_BLOCKS);

    let mut ref_lexer = get_lexer("ply", true).unwrap();
    let baseline_tokens: Vec<String> = ref_lexer
        .tokenize(&code)
        .into_iter()
        .map(|t| t.token_type)
        .collect();

    assert!(
        baseline_tokens.len() > 30_000,
        "Esperados >30.000 tokens, obtidos {}",
        baseline_tokens.len()
    );

    for &strategy in ALL_STRATEGIES {
        let mut lexer = get_lexer(strategy, true).unwrap();
        let start = Instant::now();
        let tokens: Vec<String> = lexer
            .tokenize(&code)
            .into_iter()
            .map(|t| t.token_type)
            .collect();
        let elapsed = start.elapsed().as_secs_f64();
        let throughput = tokens.len() as f64 / elapsed.max(1e-9);

        println!(
            "[PERFORMANCE] Lexer: {:<15} | Tokens: {} | Tempo: {:.4}s | Vazão: {:.0} tok/s",
            strategy,
            tokens.len(),
            elapsed,
            throughput
        );

        assert_eq!(
            tokens, baseline_tokens,
            "Divergência na sequência de tokens para a estratégia '{}'",
            strategy
        );
    }
}
