pub mod base;
pub mod log_error_lexer;
pub mod logos_lexer;
pub mod tokens;

pub use base::{BaseLexer, Token};
pub use log_error_lexer::LogErrorLexer;
pub use logos_lexer::LogosLexer;
pub use tokens::TokenKind;

/// Estratégia léxica oficial do compilador em Rust utilizando o crate `logos`.
pub const ALL_STRATEGIES: &[&str] = &["logos"];

/// Retorna a instância do analisador léxico baseado em `logos`.
pub fn get_lexer(strategy: &str, check_key: bool) -> Result<Box<dyn BaseLexer>, String> {
    match strategy.to_lowercase().as_str() {
        "logos" | "default" | "" => Ok(Box::new(LogosLexer::new(check_key))),
        other => Err(format!("Estratégia léxica desconhecida: '{}'", other)),
    }
}
