
use std::rc::Rc;

use crate::tokenize::{is_operator, Token, TokenType};

struct NumberNode {
    val: i64,
}

struct VariableNode {
    symbol: Token,
}

#[derive(Copy, Clone)]
enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

struct BinaryOpNode {
    lhs: Box<Expression>,
    rhs: Box<Expression>,
    op: Operation
}

struct VariableDeclNode {
    symbol: Token,
    rhs: Option<Expression>,
}

enum Expression {
    Binary(BinaryOpNode),
    Number(NumberNode),
    Variable(VariableNode),
}

enum ExpressionStatementWithBlock {
    //TODO:
}

enum ExpressionStatement {
    Expression(Expression),
    ExpressionWithBlock(ExpressionStatementWithBlock),
}

enum DeclNode {
    //TODO:
    Proc(ProcNode)
}

enum Statement {
    ExpressionStatement(ExpressionStatement),
    Declaration(DeclNode)
}

struct Scope {
    parent_scope: Option<usize>,
    id: usize
}

struct ProcNode {
    name: Token,
    block: BlockNode,
}

struct BlockNode {
    statements: Vec<Statement>,
    scope: usize,
}

pub struct Ast {
    root_block: Option<BlockNode>,
    tokens: Vec<Token>,
    index: usize,
    scopes: Vec<Scope>
}

impl Ast {
    fn get_current_token(&self) -> Option<&Token> {
        if self.index >= self.tokens.len() {
            return None;
        }
        return Some(&self.tokens[self.index]);
    }

    fn get_peek(&self) -> Option<&Token> {
        if self.index + 1 >= self.tokens.len() {
            return None;
        }
        Some(&self.tokens[self.index + 1])
    }

    fn match_token(&self, expected: TokenType) {
        let current_tt = self.get_current_token().unwrap().tt;
        if current_tt != expected {
            panic!("Expected \"{:?}\" but got \"{:?}\"", expected, current_tt);
        }
    }

    fn advance(&mut self) {
        self.index += 1;
    }
}

// Helper function to convert Operation to string
fn op_to_string(op: Operation) -> &'static str {
    match op {
        Operation::Add => "+",
        Operation::Sub => "-",
        Operation::Mul => "*",
        Operation::Div => "/",
    }
}

fn tt_to_string(tt: &TokenType) -> String {
    format!("{:?}", tt)
}

// ============ PRINT AST FUNCTIONS ============
pub fn print_ast(ast: &Ast) {
    println!("=== AST ===");
    if let Some(ref root) = ast.root_block {
        print_block(root, 0);
    } else {
        println!("(empty)");
    }
    println!("===========");
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn print_block(block: &BlockNode, level: usize) {
    println!("{}Block {{", indent(level));
    if block.statements.is_empty() {
        println!("{}  (no statements)", indent(level));
    } else {
        for (i, stmt) in block.statements.iter().enumerate() {
            println!("{}  [{}]", indent(level), i);
            print_statement(stmt, level + 2);
        }
    }
    println!("{}}}", indent(level));
}

fn print_statement(stmt: &Statement, level: usize) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            print_expression_statement(expr_stmt, level);
        }
        Statement::Declaration(decl) => {
            print_declaration(decl, level);
        }
    }
}

fn print_expression_statement(expr_stmt: &ExpressionStatement, level: usize) {
    match expr_stmt {
        ExpressionStatement::Expression(expr) => {
            println!("{}ExpressionStatement:", indent(level));
            print_expression(expr, level + 1);
        }
        ExpressionStatement::ExpressionWithBlock(_) => {
            println!("{}ExpressionStatement (with block):", indent(level));
            println!("{}  (TODO: implement)", indent(level));
        }
    }
}

fn print_declaration(decl: &DeclNode, level: usize) {
    match decl {
        DeclNode::Proc(proc_node) => {
            print_proc(proc_node, level);
        }
    }
}

fn print_proc(proc: &ProcNode, level: usize) {
    println!("{}Proc Declaration:", indent(level));
    println!("{}  name: \"{}\"", indent(level), proc.name.tok);
    println!("{}  body:", indent(level));
    print_block(&proc.block, level + 2);
}

fn print_expression(expr: &Expression, level: usize) {
    match expr {
        Expression::Number(num) => {
            println!("{}Number: {}", indent(level), num.val);
        }
        Expression::Variable(var) => {
            println!("{}Variable: \"{}\"", indent(level), var.symbol.tok);
        }
        Expression::Binary(bin) => {
            println!("{}BinaryOp: {}", indent(level), op_to_string(bin.op));
            println!("{}  lhs:", indent(level));
            print_expression(&bin.lhs, level + 2);
            println!("{}  rhs:", indent(level));
            print_expression(&bin.rhs, level + 2);
        }
    }
}

fn print_expression_root(expr: &Expression) {
    println!("=== Expression ===");
    print_expression(expr, 0);
    println!("==================");
}

fn ast_parse_primary(ast: &mut Ast, scope: usize) -> Expression {
    let current_token = ast.get_current_token().unwrap();
    match current_token.tt {
        TokenType::Number => {
            let ret = Expression::Number(NumberNode {
                val: current_token.tok.parse().unwrap(),
            });
            ast.advance();
            return ret;
        },
        _ => panic!("(Parse Primary) Unexpected Token")
    }
}

fn get_op(token: &Token) -> Operation {
    match token.tt {
        TokenType::Plus  => Operation::Add,
        TokenType::Sub   => Operation::Sub,
        TokenType::Slash => Operation::Div,
        TokenType::Star  => Operation::Mul,
        _ => panic!("Unknown Operator \"{}\"", token.tok)
    }
}

fn get_prec(op: TokenType) -> i32 {
    match op {
        TokenType::Star | TokenType::Slash => 5,
        TokenType::Plus | TokenType::Sub => 4,
        TokenType::SemiColon => -1,
        _ => panic!("Unknown Operator {:?}", op),
    }
}

fn ast_create_binary(op: Operation, lhs: Expression, rhs: Expression) -> BinaryOpNode {
    return BinaryOpNode { lhs: Box::new(lhs), rhs: Box::new(rhs), op };
}

fn create_expr_with_prec(ast: &mut Ast, min_prec: i32, scope: usize) -> Expression {
    let mut lhs = ast_parse_primary(ast, scope);
    loop {
        let op_token = ast.get_current_token();
        if op_token.is_none() {
            break;
        }
        let op_token = op_token.unwrap().clone();
        let prec = get_prec(op_token.tt);
        if prec == -1 {
            break;
        }

        // TODO: check if this actually succeeded
        ast.advance();
        // if success { // TODO: something like this
        //     return lhs
        // }

        let rhs = create_expr_with_prec(ast, prec, scope);

        lhs = Expression::Binary(ast_create_binary(get_op(&op_token), lhs, rhs));
    }
    return lhs;
}

fn create_expr(ast: &mut Ast, scope: usize) -> Expression {
    return create_expr_with_prec(ast, 0, scope);
}

fn create_proc(ast: &mut Ast, parent_scope: usize) -> ProcNode {
    let name = ast.get_current_token().unwrap().clone();
    ast.advance(); // proc
    ast.advance(); // ::
    // procedure
    ast.match_token(TokenType::DoubleColon);
    ast.advance();
    ast.match_token(TokenType::OpenParen);
        // TODO: handle the arguments here
    ast.advance();
    ast.match_token(TokenType::CloseParen);
    ast.advance();
    ast.match_token(TokenType::OpenCurly);
    ast.advance(); // {
        // TODO: handle body here
    let proc_block = create_block(ast, false, Some(parent_scope));
    ast.match_token(TokenType::CloseCurly);
    ast.advance(); // }

    ProcNode {
        name: name,
        block: proc_block    
    }
}

fn create_new_scope(ast: &mut Ast, parent_scope: Option<usize>) -> usize {
    ast.scopes.push(Scope {
        parent_scope,
        id: ast.scopes.len()
    });
    return ast.scopes.len() - 1;
}

fn create_block(ast: &mut Ast, root: bool, parent_scope: Option<usize>) -> BlockNode {
    let mut block = BlockNode{
        statements: vec!(),
        scope: create_new_scope(ast, parent_scope),
    };
    
    loop {
        let current_token = ast.get_current_token().unwrap();
        match current_token.tt {
            TokenType::Number | TokenType :: Word => {
                // expression or Decl
                let peek = ast.get_peek().unwrap(); // TODO: handle errors for this later
                if peek.tt == TokenType::Word ||
                   peek.tt == TokenType::ShortAssign ||
                   peek.tt == TokenType::DoubleColon ||
                   peek.tt == TokenType::Proc
               {
                    // declaration
                    if peek.tt == TokenType::Proc {
                        assert!(root, "inner functions are not supported currently");
                        let proc = create_proc(ast, ast.scopes[block.scope].id);
                        block.statements.push(Statement::Declaration(DeclNode::Proc(proc)));
                    } else {
                        todo!("handle errors");
                    }
                } else if is_operator(&peek.tt) {
                    assert!(!root, "Expressions are not allowed in the top level scope");
                    // expression statement
                    let expr = create_expr(ast, ast.scopes[block.scope].id);
                    block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
                }
            },
            TokenType::CloseCurly => {
                if root {
                    panic!("Unexpected Close Curly Bracket found in top level scope");
                } else {
                    break;
                }
            }
            _ => todo!("Unexpected Token \"{:?}\"", current_token.tt),
        }

        ast.advance();
        if ast.index >= ast.tokens.len() {
            break;
        }
    }
    return block;
}

pub fn ast_create(tokens: Vec<Token>) -> Ast {
    let mut ast = Ast {
        root_block: None,
        tokens,
        index: 0,
        scopes: vec!()
    };

    ast.root_block = Some(create_block(&mut ast, true, None));
    
    print_ast(&ast);
    return ast;    
}

// TODO: improve error handling
// TODO: improve error messages
