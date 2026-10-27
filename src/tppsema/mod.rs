//! Módulo de Análise Semântica (Semantic) - Esqueleto para a 3ª Fase do Compilador TPP.

use crate::tppparser::SyntaxTree;

/// Estrutura esqueleto do Analisador Semântico.
#[derive(Debug, Clone, Default)]
pub struct SemanticAnalyzer {
    pub check_key: bool,
}

impl SemanticAnalyzer {
    pub fn new(check_key: bool) -> Self {
        Self { check_key }
    }

    pub fn check_semantic_rules(&self, tree: &SyntaxTree) -> Result<SyntaxTree, String> {
        Ok(tree.clone())
    }
}
