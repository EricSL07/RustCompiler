# Projeto e Implementação da Fase de Análise Sintática de um Compilador para a Linguagem TPP em Rust

**Autor:** Éric Seles Lourenço  
**Instituição:** Universidade Tecnológica Federal do Paraná (UTFPR)  
**Disciplina:** OPT011 — Implementação de Linguagens de Programação  

---

## Resumo

Este artigo apresenta o projeto e a implementação da fase de análise sintática de um compilador para a linguagem TPP, desenvolvido em Rust. A análise sintática é uma etapa crucial no processo de compilação, responsável por validar a estrutura do código-fonte com base na gramática da linguagem e gerar uma representação no formato de uma árvore sintática abstrata (AST), que será utilizada nas fases subsequentes do compilador. O trabalho aborda a construção de um analisador sintático utilizando técnicas de parsing, detalhando a gramática da linguagem TPP, a implementação do parser em Rust e os desafios enfrentados durante o desenvolvimento.

**Palavras-chave:** Compiladores, Análise Sintática, Parsing, Linguagem TPP, Rust, Logos.

## Introdução

Louden (2012) define análise sintática como a etapa do compilador que determina a sintaxe, ou estrutura, de um programa de acordo com a gramática da linguagem de programação. A análise sintática é responsável por verificar se a sequência de tokens gerada na fase de análise léxica está de acordo com as regras gramaticais da linguagem. Caso o código-fonte esteja correto, o analisador sintático constrói uma árvore sintática abstrata (AST), que representa a estrutura hierárquica do programa e serve como base para as fases subsequentes do compilador, como análise semântica e geração de código.

Um f