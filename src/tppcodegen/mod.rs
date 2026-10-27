//! Módulo de Geração de Código (CodeGen) - Esqueleto para a 4ª Fase do Compilador TPP.

use crate::tppparser::SyntaxTree;

/// Estrutura esqueleto do Gerador de Código Intermediário / Final (ex.: LLVM-IR).
#[derive(Debug, Clone, Default)]
pub struct CodeGenerator {
    pub target: String,
}

impl CodeGenerator {
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
        }
    }

    pub fn generate(&self, _tree: &SyntaxTree) -> Result<String, String> {
        Ok(String::new())
    }
}
