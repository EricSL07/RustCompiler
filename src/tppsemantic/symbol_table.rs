use std::collections::HashMap;

// Tipos de dados suportados pela TPP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TppType {
    Inteiro,
    Flutuante,
    Vazio,
}

// Estrutura para armazenar informações sobre variáveis
#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub var_type: TppType,
    pub dimensions: Vec<Option<usize>>,
    pub scope: String,
    pub initialized: bool,
    pub used: bool,
    pub line: usize,
    pub column: usize,
}

// Estrutura para armazenar informações sobre funções
#[derive(Debug, Clone)]
pub struct FuncSymbol {
    pub name: String,
    pub return_types: Vec<TppType>, // Vec para suportar retorno múltiplo da TPP v1.1
    pub params: Vec<VarSymbol>,
    pub used: bool,
    pub line: usize,
    pub column: usize,
}

// Estrutura para armazenar a tabela de símbolos
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub global_vars: HashMap<String, VarSymbol>,
    pub local_vars: HashMap<String, HashMap<String, VarSymbol>>, // Escopo -> (Nome da variável -> VarSymbol)
    pub functions: HashMap<String, FuncSymbol>,
}

// Implementação de métodos para TppType
impl TppType {
    pub fn from_str(type_str: &str) -> Option<TppType> {
        match type_str.to_ascii_lowercase().as_str() {
            "inteiro" => Some(TppType::Inteiro),
            "flutuante" => Some(TppType::Flutuante),
            "vazio" => Some(TppType::Vazio),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TppType::Inteiro => "inteiro",
            TppType::Flutuante => "flutuante",
            TppType::Vazio => "vazio",
        }
    }
}

// Implementação de métodos para SymbolTable
impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            global_vars: HashMap::new(),
            local_vars: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    // Declara uma variável na tabela de símbolos, verificando se já foi declarada no mesmo escopo
    pub fn declare_variable(&mut self, var: VarSymbol) -> Result<(), TppType> {
        if var.scope == "global" {
            if let Some(existing) = self.global_vars.get(&var.name) {
                return Err(existing.var_type);
            }
            self.global_vars.insert(var.name.clone(), var);
        } else {
            let scope_vars = self.local_vars.entry(var.scope.clone()).or_insert_with(HashMap::new);
            if let Some(existing) = scope_vars.get(&var.name) {
                return Err(existing.var_type);
            }
            scope_vars.insert(var.name.clone(), var);
        }
        Ok(())
    }

    // Declara uma função na tabela de símbolos, verificando se já foi declarada
    pub fn declare_function(&mut self, func: FuncSymbol) -> Result<(), TppType> {
        if let Some(existing) = self.functions.get(&func.name) {
            return Err(existing.return_types.first().copied().unwrap_or(TppType::Vazio));
        }
        self.functions.insert(func.name.clone(), func);
        Ok(())
    }

    // Recupera uma variável da tabela de símbolos, considerando o escopo
    pub fn get_variable(&self, name: &str, scope: &str) -> Option<&VarSymbol> {
        if scope != "global" {
            if let Some(scope_vars) = self.local_vars.get(scope) {
                if let Some(var) = scope_vars.get(name) {
                    return Some(var);
                }
            }
        }
        self.global_vars.get(name)
    }

    // Recupera uma variável mutável da tabela de símbolos, considerando o escopo
    pub fn get_variable_mut(&mut self, name: &str, scope: &str) -> Option<&mut VarSymbol> {
        if scope != "global" {
            if let Some(scope_vars) = self.local_vars.get_mut(scope) {
                if let Some(var) = scope_vars.get_mut(name) {
                    return Some(var);
                }
            }
        }
        self.global_vars.get_mut(name)
    }

    // Recupera uma função da tabela de símbolos
    pub fn get_function(&self, name: &str) -> Option<&FuncSymbol> {
        self.functions.get(name)
    }

    // Recupera uma função mutável da tabela de símbolos
    pub fn get_function_mut(&mut self, name: &str) -> Option<&mut FuncSymbol> {
        self.functions.get_mut(name)
    }
}