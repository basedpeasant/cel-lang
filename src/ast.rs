
use std::rc::Rc;

use crate::tokenize::{is_operator, Token, TokenType};

#[derive(Debug)]
pub struct NumberNode {
    pub val: i64,
}

#[derive(Debug)]
pub struct VariableNode {
    pub symbol: Token,
}

#[derive(Debug, Copy, Clone)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Assign,
    ArrayIndex
}

#[derive(Debug, Clone)]
pub struct CustomType {
    pub name: Option<Token>,
    pub fields: Vec<(Token, Type)>,
}

#[derive(Debug, Clone)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    Array((usize, Box<Type>)),
    Slice(Box<Type>),
    Custom(CustomType)
}

#[derive(Debug)]
pub struct IndexNode {
    pub base: Box<Expression>,
    pub index: Box<Expression>
}

#[derive(Debug)]
pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub op: Operation
}

#[derive(Debug)]
pub struct ArrayLiteral {
    pub elements: Vec<Expression>,
    pub size: Option<usize>
}

#[derive(Debug)]
pub struct VariableDeclNode {
    pub symbol: Token,
    pub rhs: Option<Expression>,
    pub type_: Type
}

#[derive(Debug)]
pub struct CallNode {
    pub name: Token,
    pub args: Vec<Expression>
}

#[derive(Debug)]
pub struct StringLiteral {
    pub str: String
}

#[derive(Debug)]
pub enum Expression {
    Index(IndexNode),
    Binary(BinaryOpNode),
    Number(NumberNode),
    Variable(VariableNode),
    Call(CallNode),
    String(StringLiteral),
    Array(ArrayLiteral)
}

pub enum ExpressionStatementWithBlock {
    //TODO:
}

pub enum ExpressionStatement {
    Return(ReturnNode),
    Expression(Expression),
    Defer(Expression),
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

pub struct ReturnNode {
    pub expr: Expression
}

pub struct ProcNode {
    pub name: Token,
    pub block: BlockNode,
    pub args: Vec<VariableDeclNode>,
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
    pub scopes: Vec<Scope>,
    pub types: Vec<Type>,
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
        Operation::Assign => "=",
        Operation::ArrayIndex => panic!("Array indexing \"[]\" is not a binary operation")
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
        ExpressionStatement::Return(ret) => {
            println!("{}Return:", indent(level));
            print_expression(&ret.expr, level + 1);
        }
        ExpressionStatement::Defer(expr) => {
            println!("{}Defer:", indent(level));
            print_expression(&expr, level + 1);
        }
    }
}

fn type_to_string(type_: &Type) -> String {
    match type_ {
        Type::U8 => "U8".to_string(),
        Type::U16 => "U16".to_string(),
        Type::U32 => "U32".to_string(),
        Type::U64 => "U64".to_string(),
        Type::I8 => "i8".to_string(),
        Type::I16 => "i16".to_string(),
        Type::I32 => "i32".to_string(),
        Type::I64 => "i64".to_string(),
        Type::Array(arr) => {
            let type_str = type_to_string(&*arr.1);
            return format!("[{}; {}]", type_str, arr.0);
        },
        Type::Custom(custom) => {
            todo!("Not implemented yet")
        }
        Type::Slice(slice) => todo!("Slices not implemented yet")
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
        Expression::Call(call) => {
            println!("{}Call: {}", indent(level), call.name.tok);
        },
        Expression::String(str) => {
            println!("{}String: {}", indent(level), str.str);
        },
        Expression::Array(arr) => {
            println!("{}ArrayLiteral: [", indent(level));
            for expr in &arr.elements {
                print_expression(expr, level + 2);
            }
            println!("{}]", indent(level));
        },
        Expression::Index(index) => {
            print!("{}ArrayIndex: ", indent(level));
            print_expression(&index.base, 0);
            println!("{}[", indent(level));
            print_expression(&index.index, level + 2);
            println!("{}]", indent(level));
        }
    }
}

fn print_expression_root(expr: &Expression) {
    println!("=== Expression ===");
    print_expression(expr, 0);
    println!("==================");
}

fn parse_postfix(ast: &mut Ast, base: Expression, scope: usize) -> Expression {
    let current_token = ast.get_current_token().unwrap();
    if current_token.tt == TokenType::OpenSquare {
        ast.advance();
        let index = create_expr(ast, scope);
        ast.match_token(TokenType::CloseSquare);
        ast.advance();
        let expr = Expression::Index(IndexNode { base: Box::new(base), index: Box::new(index) });
        return parse_postfix(ast, expr, scope);
    } else {
        return base;
    }
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
            let peek = ast.get_peek().unwrap();
            if peek.tt == TokenType::OpenParen {
                let name = current_token.clone();
                ast.advance();
                ast.advance();
                // handle args TODO:
                let mut args = Vec::<Expression>::new();
                loop {
                    args.push(create_expr(ast, scope));
                    let current_token = ast.get_current_token().unwrap().clone();
                    if current_token.tt == TokenType::CloseParen {
                        break;
                    } else if current_token.tt == TokenType::Comma {
                        ast.advance();
                    }
                }
                let ret = Expression::Call(CallNode { name, args });
                ast.advance();
                return ret;
            } else {
                let ret = Expression::Variable(VariableNode { symbol: current_token.clone() });
                ast.advance();
                return ret;
            }
        },
        TokenType::DoubleQuote => {
            let ret = Expression::String(StringLiteral { str: current_token.tok.clone() });
            ast.advance();
            return ret;
        },
        TokenType::OpenSquare => {
            ast.advance();
            let mut arr = ArrayLiteral {
                elements: vec!(),
                size: None
            };
            let mut current_token = ast.get_current_token().unwrap();
            loop {
                if current_token.tt == TokenType::CloseSquare {
                    break;
                } else if current_token.tt == TokenType::Comma {
                    ast.advance();
                }
                arr.elements.push(create_expr(ast, scope));
                current_token = ast.get_current_token().unwrap();
            }
            ast.match_token(TokenType::CloseSquare);
            ast.advance();
            let ret = Expression::Array(arr);
            return ret;
        },
        _ => panic!("(Parse Primary) Unexpected Token \"{}\"", current_token.tok)
    }
}

fn get_op(token: &Token) -> Operation {
    match token.tt {
        TokenType::Plus  => Operation::Add,
        TokenType::Sub   => Operation::Sub,
        TokenType::Slash => Operation::Div,
        TokenType::Star  => Operation::Mul,
        TokenType::Assign => Operation::Assign,
        TokenType::OpenSquare => Operation::ArrayIndex,
        _ => panic!("Unknown Operator \"{}\"", token.tok)
    }
}

fn get_prec(op: TokenType) -> i32 {
    match op {
        TokenType::OpenSquare => 6,
        TokenType::Star | TokenType::Slash => 5,
        TokenType::Plus | TokenType::Sub => 4,
        TokenType::Assign => 3,
        TokenType::SemiColon | TokenType::Comma
        | TokenType::CloseParen | TokenType:: CloseSquare => -1,
        _ => panic!("Unknown Operator {:?}", op),
    }
}

fn ast_create_binary(op: Operation, lhs: Expression, rhs: Expression) -> BinaryOpNode {
    return BinaryOpNode { lhs: Box::new(lhs), rhs: Box::new(rhs), op };
}

fn create_expr_with_prec(ast: &mut Ast, min_prec: i32, scope: usize) -> Expression {
    let mut lhs = parse_primary(ast, scope);
    lhs = parse_postfix(ast, lhs, scope);
    loop {
        let op_token = ast.get_current_token();
        if op_token.is_none() {
            break;
        }
        let op_token = op_token.unwrap().clone();
        let prec = get_prec(op_token.tt);
        if prec == -1 || prec < min_prec {
            break;
        }

        // TODO: check if this actually succeeded
        ast.advance();
        // if success { // TODO: something like this
        //     return lhs
        // }

        let rhs = create_expr_with_prec(ast, prec, scope);

        lhs = Expression::Binary(ast_create_binary(get_op(&op_token), lhs, rhs));

        lhs = parse_postfix(ast, lhs, scope);
    }
    return lhs;
}

fn create_expr(ast: &mut Ast, scope: usize) -> Expression {
    return create_expr_with_prec(ast, 0, scope);
}

fn get_type(token: &Token) -> Type {
    match token.tok.as_str() {
        "u8"  => Type::U8,
        "u16" => Type::U16,
        "u32" => Type::U32,
        "u64" => Type::U64,
        "i8"  => Type::I8,
        "i16" => Type::I16,
        "i32" => Type::I32,
        "i64" => Type::I64,
        _     => panic!("Unrecognized type: \"{}\"", token.tok)
    }
}

fn is_type(ast: &Ast, token: &Token) -> bool {
    match token.tok.as_str() {
        "u8"  => true,
        "u16" => true,
        "u32" => true,
        "u64" => true,
        "i8"  => true,
        "i16" => true,
        "i32" => true,
        "i64" => true,
        _     => false // TODO: handle custom types
    }
}

fn create_variable_declaration(ast: &mut Ast, scope: usize, name: Token) -> VariableDeclNode {
    // let name = ast.get_current_token().unwrap().clone();
    let r#type: Type = extract_type(ast);
    // let current_token = ast.get_current_token().unwrap();
    let size = match &r#type {
        Type::Array(arr) => arr.0,
        _ => 0
    };
    let peek = ast.get_peek().unwrap().clone();
    if peek.tt == TokenType::ShortAssign { // Declaration with rhs
        ast.advance();
    } else if peek.tt == TokenType::SemiColon {
        // do nothing
    } else {
        panic!("Unexpected token \"{}\"", peek.tok);
    }
    ast.advance();

    let rhs: Option<Expression>;

    if ast.get_current_token().unwrap().tt == TokenType::SemiColon {
        rhs = None;
    } else {
        let mut r = create_expr(ast, scope);
        // have to make sure that the size info is in the rhs
        // for codegen purposes
        match &mut r {
            Expression::Array(arr) => {
                assert!(size >= arr.elements.len(), "Excess elements in array initialization");
                arr.size = Some(size);
            }
            _ => {}
        }
        rhs = Some(r);
    }
    
    VariableDeclNode {
        symbol: name.clone(),
        rhs,
        type_: r#type,
    }
}

fn create_proc(ast: &mut Ast, parent_scope: usize, name: Token) -> ProcNode {
    ast.advance(); // proc
    ast.advance(); // (
    ast.match_token(TokenType::OpenParen);
    // TODO: handle the arguments here
    ast.advance();
    let mut args = Vec::<VariableDeclNode>::new();
    loop {
        let mut current_token = ast.get_current_token().unwrap().clone();
        if current_token.tt == TokenType::CloseParen {
            break;
        } else if current_token.tt == TokenType::Comma {
            ast.advance();
            current_token = ast.get_current_token().unwrap().clone();
        }
        let symbol = current_token;
        ast.advance();
        ast.match_token(TokenType::Colon);
        ast.advance();
        let r#type = ast.get_current_token().unwrap();
        args.push(VariableDeclNode {
            symbol: symbol.clone(),
            rhs: None,
            type_: get_type(r#type),
        });
        ast.advance();
    }
    ast.match_token(TokenType::CloseParen);
    ast.advance();
    let mut return_type: Option<Type> = None;
    if ast.get_current_token().unwrap().tt == TokenType::Arrow {
        ast.advance();
        return_type = Some(get_type(ast.get_current_token().unwrap()));
        ast.advance();
    }
    ast.match_token(TokenType::OpenCurly);
    ast.advance(); // {
        // TODO: handle body here
    let proc_block = create_block(ast, false, Some(parent_scope));
    ast.match_token(TokenType::CloseCurly);
    // ast.advance(); // }

    ProcNode {
        name,
        block: proc_block,
        args,
        return_type    
    }
}

fn create_new_scope(ast: &mut Ast, parent_scope: Option<usize>) -> usize {
    ast.scopes.push(Scope {
        parent_scope,
        id: ast.scopes.len()
    });
    return ast.scopes.len() - 1;
}

fn extract_type(ast: &mut Ast) -> Type {
    // after the colon
    let current_token = ast.get_current_token().unwrap();
    match current_token.tt {
        TokenType::Word => {
            // primitive type or custom type e.g. i32 or Vec2
            if is_type(ast, current_token) {
                return get_type(current_token);
            } else {
                todo!("implement custom types");
            }
        },
        TokenType::Hat => {
            // pointer type
            todo!("implement pointer types");
        },
        TokenType::OpenSquare => {
            ast.advance();
            if ast.get_current_token().unwrap().tt == TokenType::CloseSquare {
                // SLICE
                todo!("implement slice");
            } else {
                let size = ast.get_current_token().unwrap().clone().tok.parse::<usize>().unwrap();
                ast.advance();
                ast.match_token(TokenType::CloseSquare);
                ast.advance();
                return Type::Array((size, Box::new(extract_type(ast))));
            }
        },
        TokenType::Type => {
            // declaration of a new type
            ast.advance();
            let current_token = ast.get_current_token().unwrap();
            let mut fields = Vec::<(Token, Type)>::new();
            match current_token.tt {
                TokenType::Struct => {
                    ast.advance();
                    ast.match_token(TokenType::OpenCurly);
                    ast.advance();
                    loop {
                        let current_token = ast.get_current_token().unwrap();
                        if current_token.tt == TokenType::CloseCurly {
                            break;
                        }
                        ast.match_token(TokenType::Word);
                        let name = ast.get_current_token().unwrap().clone();
                        ast.advance();
                        ast.match_token(TokenType::Colon);
                        ast.advance();
                        let r#type = extract_type(ast);
                        ast.advance();
                        ast.match_token(TokenType::SemiColon);
                        ast.advance();
                        fields.push((name, r#type));
                    }
                    ast.match_token(TokenType::CloseCurly);
                    return Type::Custom(CustomType { name: None, fields });
                },
                _ => panic!("Unexpected type class"),
            }
        }
        _ => panic!("Unexpected token \"{}\" found during type extraction", current_token.tok)
    }
}

fn create_block(ast: &mut Ast, root: bool, parent_scope: Option<usize>) -> BlockNode {
    let mut block = BlockNode{
        statements: vec!(),
        scope: create_new_scope(ast, parent_scope),
    };
    let mut defers = Vec::<Statement>::new();
    
    loop {
        let current_token = ast.get_current_token().unwrap();
        match current_token.tt {
            TokenType :: Word => {
                // expression or Decl
                let peek = ast.get_peek().unwrap(); // TODO: handle errors for this later
                if peek.tt == TokenType::Colon {
                    let name = current_token.clone();
                    ast.advance();
                    let peek = ast.get_peek().unwrap();
                    // declaration
                    if peek.tt == TokenType::Proc {
                        assert!(root, "inner functions are not supported currently");
                        let proc = create_proc(ast, ast.scopes[block.scope].id, name);
                        block.statements.push(Statement::Declaration(DeclNode::Proc(proc)));
                    } else if peek.tt == TokenType::Type {
                        ast.advance();
                        let mut new_type = extract_type(ast);
                        match &mut new_type {
                            Type::Custom(custom) => custom.name = Some(name),
                            _ => unreachable!()
                        }
                        ast.types.push(new_type);
                    } else {
                        ast.advance();
                        let variable_decl = create_variable_declaration(ast, block.scope, name);
                        block.statements.push(Statement::Declaration(DeclNode::Var(variable_decl)));
                    }
                } else if is_operator(&peek.tt) || peek.tt == TokenType::OpenParen {
                    assert!(!root, "Expressions are not allowed in the top level scope");
                    // expression statement
                    let expr = create_expr(ast, ast.scopes[block.scope].id);
                    block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
                } else {
                    unreachable!();
                }
            },
            TokenType::Number => {
                let expr = create_expr(ast, ast.scopes[block.scope].id);
                block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
            },
            TokenType::CloseCurly => {
                if root {
                    panic!("Unexpected Close Curly Bracket found in top level scope");
                } else {
                    break;
                }
            },
            TokenType::Return => {
                ast.advance();
                let expr = create_expr(ast, ast.scopes[block.scope].id);
                block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Return(ReturnNode { expr })));
            },
            TokenType::Defer => {
                assert!(!root, "Defer statements are not allowed in the top level scope");
                ast.advance();
                let expr = create_expr(ast, ast.scopes[block.scope].id);
                defers.push(Statement::ExpressionStatement(ExpressionStatement::Defer(expr)));
            }
            _ => todo!("Unexpected Token \"{:?}\":{}", current_token.tt, ast.index),
        }

        ast.advance();
        if ast.index >= ast.tokens.len() {
            break;
        }
    }

    for expr in defers.into_iter().rev() {
        block.statements.push(expr);
    }
    
    return block;
}

pub fn ast_start(tokens: Vec<Token>) -> Ast {
    let mut ast = Ast {
        root_block: None,
        tokens,
        index: 0,
        scopes: vec!(),
        types: vec!(),
    };

    ast.root_block = Some(create_block(&mut ast, true, None));
    
    print_ast(&ast);
    return ast;    
}

// TODO: improve error handling
// TODO: improve error messages
