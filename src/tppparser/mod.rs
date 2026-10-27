//! Módulo de Análise Sintática (Parser) - Esqueleto para a 2ª Fase do Compilador TPP.

use crate::tpplexer::Token;

/// Representação abstrata da Árvore Sintática (Syntax Tree) para as próximas fases.
#[derive(Debug, Clone, Default)]
pub struct SyntaxTree {
    pub root: Option<String>,
}

/// Trait base para estratégias de análise sintática (`yacc`, `topdown`).
pub trait BaseParser {
    fn parse(&mut self, tokens: &[Token]) -> Result<SyntaxTree, String>;
}
