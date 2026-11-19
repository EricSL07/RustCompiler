#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode {
    // Representa um nó na árvore de sintaxe abstrata (AST)
    pub node_type: String,
    pub column: usize,
    pub line: usize,
    pub children: Vec<AstNode>,
}

// Implementação de métodos para a estrutura AstNode
impl AstNode {
    // Cria um novo nó da AST com o tipo especificado e sem filhos
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            node_type: label.into(),
            column: 0,
            line: 0,
            children: Vec::new(),
        }
    }

    pub fn with_position(label: impl Into<String>, line: usize, column: usize, children: Vec<AstNode>) -> Self {
        Self {
            node_type: label.into(),
            line,
            column,
            children,
        }
    }

    /// Renderiza a AST em formato de árvore ASCII legível.
    pub fn to_ascii_tree(&self) -> String {
        let mut out = String::new();
        self.fmt_ascii(&mut out, "", true, true);
        out
    }

    fn fmt_ascii(&self, out: &mut String, prefix: &str, is_last: bool, is_root: bool) {
        if is_root {
            out.push_str(&self.node_type);
            out.push('\n');
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            out.push_str(prefix);
            out.push_str(connector);
            out.push_str(&self.node_type);
            out.push('\n');
        }

        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{}    ", prefix)
        } else {
            format!("{}│   ", prefix)
        };

        let len = self.children.len();
        for (idx, child) in self.children.iter().enumerate() {
            child.fmt_ascii(out, &child_prefix, idx + 1 == len, false);
        }
    }

    /// Gera a representação Graphviz (`.dot`) da AST.
    pub fn to_dot(&self) -> String {
        let mut out = String::from("digraph tree {\n");
        let mut counter = 0usize;
        self.fmt_dot(&mut out, &mut counter);
        out.push_str("}\n");
        out
    }

    fn fmt_dot(&self, out: &mut String, counter: &mut usize) -> usize {
        let my_id = *counter;
        *counter += 1;
        let escaped = self.node_type.replace('"', "\\\"");
        out.push_str(&format!("    n{} [label=\"{}\"];\n", my_id, escaped));
        for child in &self.children {
            let child_id = child.fmt_dot(out, counter);
            out.push_str(&format!("    n{} -> n{};\n", my_id, child_id));
        }
        my_id
    }
}
