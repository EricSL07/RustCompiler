#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TppType {
    Inteiro,
    Flutuante,
    Vazio,
}

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub var_type: TppType,
    pub dimensions: Vec<Option<usize>>,
    pub scope: String,
    pub Initialized: bool,
    pub used: bool,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct FuncSymbol {
    pub name: String,
    pub return_types: Vec<TppType>, // Vec para suportar retorno múltiplo da TPP v1.1
    pub params: Vec<VarSymbol>,
    pub used: bool,
    pub line: usize,
    pub column: usize,
}