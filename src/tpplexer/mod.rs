pub mod automatalib_lexer;
pub mod base;
pub mod log_error_lexer;
pub mod mandfa_lexer;
pub mod ply_lexer;
pub mod symtable_lexer;
pub mod tokens;

pub use automatalib_lexer::{AutomataLibLexer, AutomataLibManLexer, FormalDFA};
pub use base::{BaseLexer, Token};
pub use log_error_lexer::LogErrorLexer;
pub use mandfa_lexer::ManualDFALexer;
pub use ply_lexer::PLYLexer;
pub use symtable_lexer::{SymbolEntry, SymbolTable, SymbolTableLexer, SymbolTableManLexer};

/// Lista de estratégias léxicas suportadas pelo compilador.
pub const ALL_STRATEGIES: &[&str] = &[
    "ply",
    "mandfa",
    "symtable",
    "symtableman",
    "automatalib",
    "automatalibman",
];

/// Factory que retorna a instância do analisador léxico de acordo com a estratégia solicitada.
pub fn get_lexer(strategy: &str, check_key: bool) -> Result<Box<dyn BaseLexer>, String> {
    match strategy.to_lowercase().as_str() {
        "ply" | "regex" => Ok(Box::new(PLYLexer::new(check_key))),
        "mandfa" | "manual" | "dfa" => Ok(Box::new(ManualDFALexer::new(check_key))),
        "symtable" => Ok(Box::new(SymbolTableLexer::new(check_key))),
        "symtableman" => Ok(Box::new(SymbolTableManLexer::new(check_key))),
        "automatalib" | "automata" => Ok(Box::new(AutomataLibLexer::new(check_key))),
        "automatalibman" => Ok(Box::new(AutomataLibManLexer::new(check_key))),
        other => Err(format!("Estratégia léxica desconhecida: '{}'", other)),
    }
}
