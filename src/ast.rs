
use crate::tokenize::{is_operator, Token, TokenType};

pub struct NumberNode {
    pub val: i64,
}

pub struct VariableNode {
    pub symbol: Token,
}

#[derive(Copy, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Copy, Clone)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64
}

pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub op: Operation
}

pub struct VariableDeclNode {
    pub symbol: Token,
    pub rhs: Option<Expression>,
    pub type_: Type
}

pub enum Expression {
    Binary(BinaryOpNode),
    Number(NumberNode),
    Variable(VariableNode),
}

pub enum ExpressionStatementWithBlock {
    //TODO:
}

pub enum ExpressionStatement {
    Expression(Expression),
    ExpressionWithBlock(ExpressionStatementWithBlock),
}

pub enum DeclNode {
    //TODO:
    Proc(ProcNode),
    Var(VariableDeclNode)
}

pub enum Statement {
    ExpressionStatement(ExpressionStatement),
    Declaration(DeclNode)
}

#[derive(Clone)]
pub struct Scope {
    pub parent_scope: Option<usize>,
    pub id: usize
}

pub struct ProcNode {
    pub name: Token,
    pub block: BlockNode,
    pub return_type: Option<Type>
}

pub struct BlockNode {
    pub statements: Vec<Statement>,
    pub scope: usize,
}

pub struct Ast {
    pub root_block: Option<BlockNode>,
    tokens: Vec<Token>,
    index: usize,
    pub scopes: Vec<Scope>
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

fn type_to_string(type_: &Type) -> &'static str {
    match type_ {
        Type::U8 => "U8",
        Type::U16 => "U16",
        Type::U32 => "U32",
        Type::U64 => "U64",
        Type::I8 => "i8",
        Type::I16 => "i16",
        Type::I32 => "i32",
        Type::I64 => "i64",
    }
}

fn print_variable_decl(var_decl: &VariableDeclNode, level: usize) {
    println!("{}Variable Declaration:", indent(level));
    println!("{}  name: \"{}\"", indent(level), var_decl.symbol.tok);
    println!("{}  type: {}", indent(level), type_to_string(&var_decl.type_));
    if let Some(ref rhs) = var_decl.rhs {
        println!("{}  initializer:", indent(level));
        print_expression(rhs, level + 2);
    } else {
        println!("{}  initializer: (none)", indent(level));
    }
}

fn print_declaration(decl: &DeclNode, level: usize) {
    match decl {
        DeclNode::Proc(proc_node) => {
            print_proc(proc_node, level);
        }
        DeclNode::Var(variable_decl) => {
            print_variable_decl(variable_decl, level);
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

fn parse_primary(ast: &mut Ast, scope: usize) -> Expression {
    let current_token = ast.get_current_token().unwrap();
    match current_token.tt {
        TokenType::Number => {
            let ret = Expression::Number(NumberNode {
                val: current_token.tok.parse().unwrap(),
            });
            ast.advance();
            return ret;
        },
        TokenType::Word => {
            let ret = Expression::Variable(VariableNode { symbol: current_token.clone() });
            ast.advance();
            return ret;
        }
        _ => panic!("(Parse Primary) Unexpected Token \"{}\"", current_token.tok)
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
    let mut lhs = parse_primary(ast, scope);
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

fn get_type(token: &Token) -> Type {
    match token.tok.as_str() {
        "U8"  => Type::U8,
        "U16" => Type::U16,
        "U32" => Type::U32,
        "U64" => Type::U64,
        "i8"  => Type::I8,
        "i16" => Type::I16,
        "i32" => Type::I32,
        "i64" => Type::I64,
        _     => panic!("Unrecognized type: \"{}\"", token.tok)
    }
}

fn create_variable_declaration(ast: &mut Ast, scope: usize) -> VariableDeclNode {
    let name = ast.get_current_token().unwrap().clone();
    ast.advance();
    let type_token = ast.get_current_token().unwrap().clone();
    let peek = ast.get_peek().unwrap();
    if peek.tt == TokenType::SemiColon { // Declaration without rhs
        
    } else if peek.tt == TokenType::ShortAssign { // Declaration with rhs
        ast.advance();
    } else {
        panic!("Unexpected token \"{:?}\"", ast.get_current_token().unwrap().tok);
    }
    ast.advance();

    let rhs: Option<Expression>;

    if ast.get_current_token().unwrap().tt == TokenType::SemiColon {
        rhs = None;
    } else {
        rhs = Some(create_expr(ast, scope));
    }
    
    VariableDeclNode {
        symbol: name.clone(),
        rhs,
        type_: get_type(&type_token),
    }
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
        // TODO: handle the return type here
    ast.advance();
    ast.match_token(TokenType::OpenCurly);
    ast.advance(); // {
        // TODO: handle body here
    let proc_block = create_block(ast, false, Some(parent_scope));
    ast.match_token(TokenType::CloseCurly);
    // ast.advance(); // }

    ProcNode {
        name: name,
        block: proc_block,
        return_type: None    
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
                    } else if peek.tt == TokenType::Word || peek.tt == TokenType::ShortAssign {
                        // variable declaration
                        // TODO: shortassign
                        let variable_decl = create_variable_declaration(ast, block.scope);
                        block.statements.push(Statement::Declaration(DeclNode::Var(variable_decl)));
                    } else {
                        todo!("handle errors");
                    }
                } else if is_operator(&peek.tt) {
                    assert!(!root, "Expressions are not allowed in the top level scope");
                    // expression statement
                    let expr = create_expr(ast, ast.scopes[block.scope].id);
                    block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
                } else {
                    unreachable!();
                }
            },
            TokenType::CloseCurly => {
                if root {
                    panic!("Unexpected Close Curly Bracket found in top level scope");
                } else {
                    break;
                }
            }
            _ => todo!("Unexpected Token \"{:?}\":{}", current_token.tt, ast.index),
        }

        ast.advance();
        if ast.index >= ast.tokens.len() {
            break;
        }
    }
    return block;
}

pub fn ast_start(tokens: Vec<Token>) -> Ast {
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
