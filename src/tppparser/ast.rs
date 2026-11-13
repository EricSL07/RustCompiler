#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    // Representa um nó na árvore de sintaxe abstrata (AST)
    pub node_type: String,
    pub children: Vec<AstNode>,
}

// Implementação de métodos para a estrutura AstNode
impl AstNode {
    // Cria um novo nó da AST com o tipo especificado e sem filhos
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            node_type: label.into(),
            children: Vec::new(),
        }
    }

    // Cria um novo nó da AST com o tipo especificado e filhos fornecidos
    pub fn with_children(label: impl Into<String>, children: Vec<AstNode>) -> Self {
        Self {
            node_type: label.into(),
            children,
        }
    }

}