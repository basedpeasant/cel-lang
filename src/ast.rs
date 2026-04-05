
use std::{any::Any, collections::HashMap};

use crate::tokenize::{self, is_operator, print_tokens, Token, TokenType, create_token};
#[derive(Debug, Clone)]
pub struct NumberNode {
    pub val: i64,
}

#[derive(Debug, Clone)]
pub struct VariableNode {
    pub symbol: Token,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Or,
    LogicalOr,
    Gt,
    Gte,
    Lt,
    Lte,
    Equal,
    NotEqual,
    Reference,
    Assign,
    ArrayIndex,
    Access,
    Not,
    And,
    Mod
}

#[derive(Debug, Clone)]
pub struct CustomType {
    pub name: Option<Token>,
    pub fields: Vec<(Token, Type)>,
}

#[derive(Debug, Clone)]
pub struct ChoiceType {
    pub name: Option<Token>,
    pub fields: Vec<(Option<Token>, Type)>, // Option<Token> for labelled ones
}

#[derive(Debug, Clone, Copy)]
pub enum Attribute {
    Extern,
    Static,
    // Immutable,
}

#[derive(Debug, Clone)]
pub enum Type {
    VoidPtr(Vec<Attribute>),
    U8(Vec<Attribute>),
    U16(Vec<Attribute>),
    U32(Vec<Attribute>),
    U64(Vec<Attribute>),
    I8(Vec<Attribute>),
    I16(Vec<Attribute>),
    I32(Vec<Attribute>),
    I64(Vec<Attribute>),
    Bool(Vec<Attribute>),
    String(Vec<Attribute>),
    Proc(Vec<Attribute>, Vec<VariableDeclNode>, Vec<Box<Type>>),
    Pointer(Box<Type>),
    DynamicArray(Box<Type>),
    Array((usize, Box<Type>)),
    Slice(Box<Type>),
    Custom(Vec<Attribute>, CustomType),
    Choice(Vec<Attribute>, ChoiceType)
}

#[derive(Debug, Clone)]
pub struct IndexNode {
    pub base: Box<Expression>,
    pub index: Box<Expression>
}

#[derive(Debug, Clone)]
pub struct BinaryOpNode {
    pub lhs: Box<Expression>,
    pub rhs: Box<Expression>,
    pub op: Operation
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub elements: Vec<Expression>,
    pub size: Option<usize>
}

#[derive(Debug, Clone)]
pub struct VariableDeclNode {
    pub symbol: Token,
    pub rhs: Option<Expression>,
    pub type_: Type,
    pub is_arg: bool
}

#[derive(Debug, Clone)]
pub struct CallNode {
    pub name: Token,
    pub args: Vec<Expression>
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub str: String
}

#[derive(Debug, Clone)]
pub struct CharLiteral {
    pub c: String
}

#[derive(Debug, Clone)]
pub struct StructDeclarationNode {
    pub name: Option<Token>,
    pub exprs: Vec<Expression>
}

#[derive(Debug, Clone)]
pub enum Expression {
    Index(IndexNode),
    Binary(BinaryOpNode),
    Number(NumberNode),
    Variable(VariableNode),
    Call(CallNode),
    String(StringLiteral),
    Char(CharLiteral),
    Array(ArrayLiteral),
    Struct(StructDeclarationNode),
    Reference(Box<Expression>), // TODO: this should actually be unary node
    Not(Box<Expression>), // TODO: this should actually be unary node
}

#[derive(Clone)]
pub struct IfNode {
    pub block: BlockNode,
    pub condition: Option<Expression>,
    pub next: Option<Box<IfNode>>,
    pub is_else: bool,
}

#[derive(Clone, PartialEq)]
pub enum MatchKind {
    Switch,
    Choice,
}

#[derive(Clone)]
pub struct MatchNode {
    pub match_type: MatchKind,
    pub fields: Vec<(Option<Token>, Type)>,
    pub subject: VariableDeclNode,
    pub blocks: Vec<BlockNode>,
    pub needs_deref: bool,
    pub token: Token, // used for debugging purposes
    // pub var: Token
}

#[derive(Clone)]
pub struct ForNode {
    pub block: BlockNode,
    pub is_classic_for: bool,
    pub init: Option<Box<Statement>>,
    pub condition: Expression,
    pub post: Option<Expression>,
}

#[derive(Clone)]
pub enum ExpressionStatementWithBlock {
    If(IfNode),
    For(ForNode),
    Match(MatchNode)
}

#[derive(Clone)]
pub enum ExpressionStatement {
    Return(ReturnNode),
    Expression(Expression),
    Defer(Expression),
    ExpressionWithBlock(ExpressionStatementWithBlock),
}

#[derive(Clone)]
pub enum DeclNode {
    //TODO:
    Proc(ProcNode),
    Var(VariableDeclNode)
}

#[derive(Clone)]
pub enum Statement {
    ExpressionStatement(ExpressionStatement),
    Declaration(DeclNode),
    Break,
    Continue
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub parent_scope: Option<usize>,
    pub id: usize,
    pub map: HashMap<String, VariableDeclNode>
}

#[derive(Clone)]
pub struct ReturnNode {
    pub expr: Expression
}

#[derive(Debug, Clone)]
pub struct ProcNodeHeader {
    pub name: Token,
    pub args: Vec<VariableDeclNode>,
    pub return_type: Option<Type>,
    pub attributes: Vec<Attribute>
}

#[derive(Clone)]
pub struct ProcNode {
    pub name: Token,
    pub block: BlockNode,
    pub args: Vec<VariableDeclNode>,
    pub return_type: Option<Type>,
    pub attributes: Vec<Attribute>
}

#[derive(Clone)]
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
    pub strings: HashMap<String, StringLiteral>,
    pub procs: Vec<ProcNodeHeader>
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
            let current_token = self.get_current_token();
            println!("CurrentToken: {:?}", current_token);
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
        Operation::Equal => "==",
        Operation::NotEqual => "!=",
        Operation::Gte => ">=",
        Operation::Gt => ">",
        Operation::Lte => "<=",
        Operation::Lt => "<",
        Operation::Or => "|",
        Operation::LogicalOr => "||",
        Operation::And => "&",
        Operation::Reference => "&",
        Operation::Access => ".",
        Operation::Not => "!",
        Operation::Mod => "%",
        Operation::ArrayIndex => panic!("Array indexing \"[]\" is not a binary operation")
    }
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
        },
        Statement::Declaration(decl) => {
            print_declaration(decl, level);
        },
        Statement::Break => {
            println!("{}Break", indent(level));
        },
        Statement::Continue => {
            println!("{}Continue", indent(level));
        }
    }
}

fn print_if_statement(r#if: &IfNode, level: usize) {
    println!("{}If Statement:", indent(level));
    println!("{}Condition:", indent(level + 2));
    if r#if.condition.is_some() {
        print_expression(r#if.condition.as_ref().unwrap(), level + 4);
    } else {
        println!("{}None", indent(level + 4));
    }
    print_block(&r#if.block, level + 2);
}

fn print_for_statement(r#for: &ForNode, level: usize) {
    print!("{}For Statement", indent(level));
    if r#for.is_classic_for {
        println!(" (Classic):");
    } else {
        println!(" (TODO):"); // TODO: add print for other types
    }
    println!("{}Condition:", indent(level + 2));
    print_expression(&r#for.condition, level + 4);
    print_block(&r#for.block, level + 2);
}

fn print_expression_statement(expr_stmt: &ExpressionStatement, level: usize) {
    match expr_stmt {
        ExpressionStatement::Expression(expr) => {
            println!("{}ExpressionStatement:", indent(level));
            print_expression(expr, level + 1);
        }
        ExpressionStatement::ExpressionWithBlock(expr_with_block) => {
            println!("{}ExpressionStatement (with block):", indent(level));
            match expr_with_block {
                ExpressionStatementWithBlock::If(r#if) => {
                    print_if_statement(&r#if, level);
                },
                ExpressionStatementWithBlock::For(r#for) => {
                    print_for_statement(&r#for, level);
                },
                ExpressionStatementWithBlock::Match(r#match) => {
                    // todo!("implement printing for match")
                    print!("{}Match Statement", indent(level));
                    println!(" (TODO):"); // TODO: add print for other types
                }
            }
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
        Type::VoidPtr(_) => "void_ptr".to_string(),
        Type::U8(_) => "u8".to_string(),
        Type::U16(_) => "u16".to_string(),
        Type::U32(_) => "u32".to_string(),
        Type::U64(_) => "u64".to_string(),
        Type::I8(_) => "i8".to_string(),
        Type::I16(_) => "i16".to_string(),
        Type::I32(_) => "i32".to_string(),
        Type::I64(_) => "i64".to_string(),
        Type::Bool(_) => "bool".to_string(),
        Type::String(_) => "string".to_string(),
        Type::Proc(attributes, args, return_type) => {
            let mut str = String::new();
            for attribute in attributes {
                str.push_str(&format!("@{:?} ", attribute));
            }
            str.push_str("proc(");
            for (i, arg) in args.iter().enumerate() {
                str.push_str(&format!("{}", type_to_string(&arg.type_)));
                if i != args.len() - 1 {
                    str.push_str(", ")
                }
            }
            str.push(')');

            if return_type.len() > 0 {
                str.push_str(" -> ");
                for ret in return_type {
                    str.push_str(&format!("{}", type_to_string(ret)));
                }
            }
            
            str
        },
        Type::Array(arr) => {
            let type_str = type_to_string(&*arr.1);
            return format!("[{}; {}]", type_str, arr.0);
        },
        Type::DynamicArray(arr) => {
            let type_str = type_to_string(arr);
            return format!("[{}; dynamic]", type_str);
        },
        Type::Custom(_, custom) => {
            return custom.name.as_ref().unwrap().tok.clone();
        },
        Type::Choice(_, custom) => {
            return custom.name.as_ref().unwrap().tok.clone();
        },
        Type::Pointer(ptr) => format!("{}{}", "*", type_to_string(&ptr)),
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
            for arg in &call.args {
                print_expression(&arg, level + 2);
            }
        },
        Expression::String(str) => {
            println!("{}String: {}", indent(level), str.str);
        },
        Expression::Char(char) => {
            println!("{}Char: {}", indent(level), char.c);
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
        },
        Expression::Struct(struct_) => {
            println!("{}StructDeclaration({:?}): ", indent(level), struct_.name);
            for expr in &struct_.exprs {
               print_expression(&expr, level + 2); 
            }
        },
        Expression::Reference(expr) => {
            println!("{}Reference:", indent(level));
            print_expression(expr, level);
        },
        Expression::Not(expr) => {
            println!("{}Not:", indent(level));
            print_expression(expr, level);
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
                ast.match_token(TokenType::CloseParen);
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
            let string_literal = StringLiteral { str: current_token.tok.clone() };
            ast.strings.insert(current_token.tok.clone(), string_literal.clone());
            let ret = Expression::String(string_literal);
            ast.advance();
            return ret;
        },
        TokenType::SingleQuote => {
            let mut new = String::new();
            if current_token.tok.len() > 1 {
                if current_token.tok.len() > 2 {
                    panic!("Multiline character literal detected: {:?}", current_token);
                }
                if current_token.tok.chars().nth(0).unwrap() == '\\' {
                    match current_token.tok.chars().nth(1).unwrap() {
                        'n' => {}, // newline
                        '\\' => {}, // slash
                        't' => {}, // tabs
                        'r' => {}, // carriage return
                        '\'' => {}, // single quote
                        _ => panic!("Unsupported escape sequence '{}': {:?}", current_token.tok, current_token),
                    }
                } else {
                    panic!("Multiline character literal detected: {:?}", current_token);
                }
            } else if current_token.tok.len() == 0 {
                panic!("Single quote character literal should not be empty: {:?}", current_token);
            }
            let char_literal = CharLiteral{ c: current_token.tok.clone() };
            // ast.strings.insert(current_token.tok.clone(), string_literal.clone());
            let ret = Expression::Char(char_literal);
            ast.advance();
            return ret;
        }
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
        TokenType::Ampersand => {
            ast.advance();
            let peek = ast.get_peek().unwrap().clone();
            let mut lhs = parse_primary(ast, scope);
            if peek.tt == TokenType::Dot {
                ast.advance();
                let rhs = parse_primary(ast, scope);
                lhs = Expression::Binary(ast_create_binary(Operation::Access, lhs, rhs));
            }
            return Expression::Reference(Box::new(lhs));
        },
        TokenType::Not => {
            ast.advance();
            return Expression::Not(Box::new(parse_primary(ast, scope)));
        }
        _ => panic!("(Parse Primary) Unexpected Token \"{:?}\"", current_token)
    }
}

fn get_op(token: &Token) -> Operation {
    match token.tt {
        TokenType::Plus  => Operation::Add,
        TokenType::Sub   => Operation::Sub,
        TokenType::Slash => Operation::Div,
        TokenType::Star  => Operation::Mul,
        TokenType::Assign => Operation::Assign,
        TokenType::Or  => Operation::Or,
        TokenType::LogicalOr  => Operation::LogicalOr,
        TokenType::Equal => Operation::Equal,
        TokenType::NotEqual => Operation::NotEqual,
        TokenType::Gt => Operation::Gt,
        TokenType::Gte => Operation::Gte,
        TokenType::Lt => Operation::Lt,
        TokenType::Lte => Operation::Lte,
        TokenType::OpenSquare => Operation::ArrayIndex,
        TokenType::Dot => Operation::Access,
        TokenType::Ampersand => Operation::And,
        TokenType::Percent => Operation::Mod,
        _ => panic!("Unknown Operator \"{}\"", token.tok)
    }
}

fn get_prec(ast: &Ast, op: TokenType) -> i32 {
    match op {
        TokenType::OpenSquare => 11,
        TokenType::Dot => 10,
        TokenType::Star | TokenType::Slash | TokenType::Percent => 9,
        TokenType::Plus | TokenType::Sub => 8,
        TokenType::Gt
        | TokenType::Gte
        | TokenType::Lt
        | TokenType::Lte => 7,
        TokenType::Equal
        | TokenType::NotEqual => 6,
        TokenType::Or => 5,
        TokenType::LogicalOr => 4,
        TokenType::Ampersand => 3,
        TokenType::Assign => 2,
        TokenType::SemiColon
        | TokenType::Comma
        | TokenType::CloseParen
        | TokenType::CloseSquare
        | TokenType::OpenCurly
        | TokenType::CloseCurly  => -1,
        _ => {
            println!("{:?}", ast.get_current_token());
            panic!("Unknown Operator \"{:?}\"", op);
        },
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
        let prec = get_prec(ast, op_token.tt);
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
        "u8"  => Type::U8(vec!()),
        "u16" => Type::U16(vec!()),
        "u32" => Type::U32(vec!()),
        "u64" => Type::U64(vec!()),
        "i8"  => Type::I8(vec!()),
        "i16" => Type::I16(vec!()),
        "i32" => Type::I32(vec!()),
        "i64" => Type::I64(vec!()),
        "bool" => Type::Bool(vec!()),
        "void_ptr"  => Type::VoidPtr(vec!()),
        "string" => Type::String(vec!()),
        _     => Type::Custom(vec!(), CustomType { name: Some(token.clone()), fields: vec!() }),
        // _     => panic!("Unrecognized type: \"{}\"", token.tok)
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
        "void_ptr"  => true,
        "string" => true,
        _     => false // TODO: handle custom types
    }
}

fn create_variable_declaration(ast: &mut Ast, scope: usize, name: Token, skip_scope: bool) -> VariableDeclNode {
    // let name = ast.get_current_token().unwrap().clone();
    let r#type: Type = extract_type(ast);
    // let current_token = ast.get_current_token().unwrap();
    let size = match &r#type {
        Type::Array(arr) => arr.0,
        _ => 0
    };
    let peek = ast.get_peek().unwrap().clone();
    let mut struct_initialization = false; // we have to check if there is a struct initialization
    if peek.tt == TokenType::Assign { // Declaration with rhs
        ast.advance();
    } else if peek.tt == TokenType::SemiColon {
        // do nothing
    } else {
        panic!("Unexpected token \"{}\"", peek.tok);
    }
    ast.advance();
    {
        // checking if theres struct initialization taking place
        let current_token = ast.get_current_token().unwrap();
        let peek = ast.get_peek().unwrap();
        if current_token.tt == TokenType::Word && peek.tt == TokenType::OpenCurly {
            // struct initialization
            struct_initialization = true;
        }
    }

    let rhs: Option<Expression>;

    if ast.get_current_token().unwrap().tt == TokenType::SemiColon {
        rhs = None;
    } else {
        if !struct_initialization {
            let mut r = create_expr(ast, scope);
            // have to make sure that the size info is in the rhs
            // for codegen purposes
            match &mut r {
                Expression::Array(arr) => {
                    if !(size >= arr.elements.len()) {
                        println!("{:?}", ast.get_current_token().unwrap());
                    }
                    assert!(size >= arr.elements.len(), "Excess elements in array initialization");
                    arr.size = Some(size);
                }
                _ => {}
            }
            rhs = Some(r);
        } else {
            ast.match_token(TokenType::Word);
            let name = ast.get_current_token().unwrap().clone();
            ast.advance();
            ast.match_token(TokenType::OpenCurly);
            ast.advance();
            let mut exprs = vec!();
            let mut current_token = ast.get_current_token().unwrap().clone();
            while current_token.tt != TokenType::CloseCurly {
                let expr = create_expr(ast, scope);
                exprs.push(expr);
                if ast.get_current_token().unwrap().tt == TokenType::Comma {
                    ast.advance();
                }
                current_token = ast.get_current_token().unwrap().clone();
            }
            ast.advance();
            rhs = Some(Expression::Struct(StructDeclarationNode { name: Some(name), exprs }));
        }
    }

    let var_decl = VariableDeclNode {
        symbol: name.clone(),
        rhs,
        type_: r#type,
        is_arg: false,
    };

    match &var_decl.type_ {
        Type::Proc(attributes, args, return_types) => {
            let return_type = if return_types.len() == 0 {
                None
            } else if return_types.len() > 1 {
                unreachable!("Does not support multiple return types yet")
            } else {
                Some(*return_types[0].clone())
            };
            ast.procs.push(ProcNodeHeader { name: var_decl.symbol.clone(), args: args.clone(), return_type: return_type.clone(), attributes: attributes.to_vec() });
        },
        _ => {}
    }

    if !skip_scope {
        // we will skip scope because it gets added after parsing the for loop block
        // into the for loop's scope
        ast.scopes[scope].map.insert(name.clone().tok, var_decl.clone());
    }

    return var_decl;
}

fn create_proc(ast: &mut Ast, parent_scope: usize, name: Token) -> ProcNode {
    // let attributes = extract_attributes(ast);
    // ast.match_token(TokenType::Proc);
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
        let var_decl = VariableDeclNode {
            symbol: symbol.clone(),
            rhs: None,
            type_: extract_type(ast),
            is_arg: true
        };
        args.push(var_decl);
        ast.advance();
    }
    ast.match_token(TokenType::CloseParen);
    ast.advance();
    let mut return_type: Option<Type> = None;
    if ast.get_current_token().unwrap().tt == TokenType::Arrow {
        ast.advance();
        return_type = Some(extract_type(ast));
        ast.advance();
    }
    ast.match_token(TokenType::OpenCurly);
    ast.advance(); // {
        // TODO: handle body here
    let proc_block = create_block(ast, false, Some(parent_scope));
    for arg in &args {
        assert!(!ast.scopes[proc_block.scope].map.contains_key(&arg.symbol.tok), "symbol \"{}\" already exists in block", arg.symbol.tok);
        ast.scopes[proc_block.scope].map.insert(arg.symbol.tok.clone(), arg.clone());
    }
    ast.match_token(TokenType::CloseCurly);
    // ast.advance(); // }

    ast.procs.push(ProcNodeHeader { name: name.clone(), args: args.clone(), return_type: return_type.clone(), attributes: vec!() });
    ProcNode {
        name,
        block: proc_block,
        args,
        return_type,
        attributes: vec!() // TODO: incredibly scuffed might have to redo proc ast
    }
}

fn create_new_scope(ast: &mut Ast, parent_scope: Option<usize>) -> usize {
    ast.scopes.push(Scope {
        parent_scope,
        id: ast.scopes.len(),
        map: HashMap::new()
    });
    return ast.scopes.len() - 1;
}

fn extract_attributes(ast: &mut Ast) -> Vec<Attribute> {
    let mut ret = vec!();

    loop {
        let mut current_token = ast.get_current_token().unwrap();
        if current_token.tt == TokenType::At {
            ast.advance();
            current_token = ast.get_current_token().unwrap();
            match ast.get_current_token().unwrap().tok.as_str() {
                "extern" => ret.push(Attribute::Extern),
                "static" => ret.push(Attribute::Static),
                _ => panic!("Attribute \"{}\" is not recognized", ast.get_current_token().unwrap().tok)
            }
            ast.advance();
        } else {
            break;
        }
    }
    

    ret
}

fn extract_type(ast: &mut Ast) -> Type {
    // after the colon
    let current_token = ast.get_current_token().unwrap();
    match current_token.tt {
        TokenType::Word => {
            // primitive type or custom type e.g. i32 or Vec2
            // if is_type(ast, current_token) {
            return get_type(current_token);
            // } else {
            //     todo!("implement custom type \"{}\"", current_token.tok);
            // }
        },
        TokenType::Star => {
            // pointer type
            ast.advance();
            return Type::Pointer(Box::new(extract_type(ast)));
        },
        TokenType::OpenSquare => {
            ast.advance();
            let current_token = ast.get_current_token().unwrap();
            if current_token.tt == TokenType::CloseSquare {
                // SLICE
                todo!("implement slice");
            } if current_token.tt == TokenType::Plus {
                // Dynamic array
                ast.advance();
                ast.match_token(TokenType::CloseSquare);
                ast.advance();
                return Type::DynamicArray(Box::new(extract_type(ast)));
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
            match current_token.tt {
                TokenType::Struct => {
                    let mut fields = Vec::<(Token, Type)>::new();
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
                        let current_token = ast.get_current_token().unwrap();
                        if current_token.tt == TokenType::Comma {
                            ast.advance();
                        } else if current_token.tt != TokenType::CloseCurly {
                            // TODO: maek this error better lol
                            println!("{:?}", current_token);
                            panic!("Seems you have missed a comma!");
                        }
                        fields.push((name, r#type));
                    }
                    ast.match_token(TokenType::CloseCurly);
                    return Type::Custom(vec!(), CustomType { name: None, fields });
                },
                TokenType::Choice => {
                    ast.advance();
                    let mut fields = Vec::<(Option<Token>, Type)>::new();
                    ast.match_token(TokenType::OpenCurly);
                    ast.advance();
                    loop {
                        // TODO: named fields
                        let current_token = ast.get_current_token().unwrap();
                        if current_token.tt == TokenType::CloseCurly {
                            break;
                        }
                        ast.match_token(TokenType::Word);
                        let r#type = extract_type(ast);
                        ast.advance();
                        let current_token = ast.get_current_token().unwrap();
                        if current_token.tt == TokenType::Comma {
                            ast.advance();
                        } else if current_token.tt != TokenType::CloseCurly {
                            // TODO: maek this error better lol
                            println!("{:?}", current_token);
                            panic!("Seems you have missed a comma!");
                        }
                        fields.push((None, r#type));
                    }
                    ast.match_token(TokenType::CloseCurly);
                    return Type::Choice(vec!(), ChoiceType { name: None, fields });
                }
                _ => panic!("Unexpected type class"),
            }
        },
        TokenType::Proc => {
            ast.match_token(TokenType::Proc);
            ast.advance(); // proc
            ast.advance(); // (
            // ast.match_token(TokenType::CloseParen);
            // TODO: handle the arguments here
            // ast.advance();
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
                let var_decl = VariableDeclNode {
                    symbol: symbol.clone(),
                    rhs: None,
                    type_: extract_type(ast),
                    is_arg: true,
                };
                args.push(var_decl);
                ast.advance();
            }
            // ast.match_token(TokenType::CloseParen);
            // ast.advance();
            // let mut return_type: Option<Type> = None;
            let mut return_type = vec!();
            if ast.get_peek().unwrap().tt == TokenType::Arrow {
                ast.advance();
                ast.advance();
                return_type.push(Box::new(extract_type(ast)));
                // ast.advance();
            }
            let peek = ast.get_peek().unwrap();
            if peek.tt == TokenType::OpenCurly {
                todo!("implement proc fully in extract_type?(maybe not tbh)");
            } else if peek.tt == TokenType::SemiColon {
                // a procedure definition (likely an extern)
            } else {
                panic!("Unexpected peek \"{}\"", peek.tok);
            }

            Type::Proc(vec!(), args, return_type)
        }
        TokenType::At => {
            // attribute
            let attributes = extract_attributes(ast);
            let mut r#type = extract_type(ast);
            match &mut r#type {
                Type::Proc(proc_attributes, _, _) => {
                      for attribute in attributes {
                          proc_attributes.push(attribute.clone());
                      }
                },
                Type::Custom(custom_attributes, _) => {
                      for attribute in attributes {
                          custom_attributes.push(attribute.clone());
                      }
                }
                _ => todo!("not implemented attributes for types outside of proc"),
            }
            // ast.match_token(TokenType::SemiColon);
            r#type            
        },
        _ => panic!("Unexpected token \"{}\" found during type extraction", current_token.tok)
    }
}

fn create_if(ast: &mut Ast, parent_scope: usize) -> IfNode {
    let current_token = ast.get_current_token().unwrap().clone();
    let peek = ast.get_peek().unwrap().clone();
    let mut skip_condition = false;
    let mut is_else = false;
    if current_token.tt == TokenType::If {
        // if
        ast.advance();
    } else if current_token.tt == TokenType::Else && peek.tt == TokenType::OpenCurly {
        // else
        skip_condition = true;
        is_else = true;
        ast.advance();    
    } else if current_token.tt == TokenType::Else && peek.tt == TokenType::If {
        // else if
        is_else = true;
        ast.advance();
        ast.advance();
    } else {
        unreachable!();
    }
    let mut condition = None;
    if !skip_condition {
        condition = Some(create_expr(ast, parent_scope));
    }
    ast.match_token(TokenType::OpenCurly);
    ast.advance();
    let if_block = create_block(ast, false, Some(parent_scope));
    ast.match_token(TokenType::CloseCurly);
    let peek = ast.get_peek().unwrap();
    let mut next = None;
    if peek.tt == TokenType::Else {
        ast.advance();
        next = Some(Box::new(create_if(ast, parent_scope)));
    }
    return IfNode {
        block: if_block,
        condition,
        next,
        is_else
    };
}

fn create_for(ast: &mut Ast, parent_scope: usize) -> ForNode {
    // TODO: implement other kinds of for loops
    ast.match_token(TokenType::For);
    ast.advance();
    let current_token = ast.get_current_token().unwrap().clone();
    let peek = ast.get_peek().unwrap().clone();

    // NOTE: this won't actually be that great at differentiating for loop types but
    // it'll do for now
    if peek.tt == TokenType::Colon {
        // assume its a classic for loop
        let name = current_token.clone();
        ast.advance();
        ast.advance();
        let variable_decl = create_variable_declaration(ast, 69, name, true);
        // ast.scopes[block.scope].map.insert(variable_decl.symbol.tok.clone(), variable_decl.clone());
        ast.match_token(TokenType::SemiColon);
        ast.advance();
        let condition;
        if ast.get_current_token().unwrap().tt != TokenType::SemiColon {
            condition = create_expr(ast, parent_scope);
            ast.match_token(TokenType::SemiColon);
            ast.advance();
        } else {
            todo!("if the person skips the condition, not handled yet")
        }
        let post = create_expr(ast, parent_scope);
        ast.match_token(TokenType::OpenCurly);
        ast.advance();

        let for_block = create_block(ast, false, Some(parent_scope));
        ast.match_token(TokenType::CloseCurly);
        return ForNode {
            block: for_block,
            is_classic_for: true,
            init: Some(Box::new(Statement::Declaration(DeclNode::Var(variable_decl)))),
            condition,
            post: Some(post)
        };
    } else {
        let condition = create_expr(ast, parent_scope);
        ast.match_token(TokenType::OpenCurly);
        ast.advance();
        let for_block = create_block(ast, false, Some(parent_scope));
        ast.match_token(TokenType::CloseCurly);

        return ForNode {
            block: for_block,
            is_classic_for: false,
            init: None,
            condition,
            post: None
        };
    }
}

fn lookup_var(ast: &Ast, scope_id: usize, name: &str) -> Option<VariableDeclNode> {
    let mut scope_id = scope_id;

    loop {
        if let Some(decl) = ast.scopes[scope_id].map.get(name) {
            return Some(decl.clone());
        }

        match ast.scopes[scope_id].parent_scope {
            Some(parent) => scope_id = parent,
            None => return None,
        }
    }
}

fn get_ast_type(types: &Vec<Type>, custom_type_name: &String) -> Type {
    // let custom_type_name = custom_type.name.as_ref().unwrap();
    for r#type in types {
        match r#type {
            Type::Choice(_attributes, custom_type_cmp) => {
                let name = custom_type_cmp.name.as_ref().unwrap();
                if *custom_type_name == name.tok {
                    return r#type.clone();
                }
            },
            Type::Custom(_attributes, custom_type_cmp) => {
                let name = custom_type_cmp.name.as_ref().unwrap();
                if *custom_type_name == name.tok {
                    return r#type.clone();
                }
            },
            _ => panic!("{:?} is not a choice type", r#type)          
        };
    }
    panic!("Could not find type {}", custom_type_name);
}

fn lookup_choice_type(types: &Vec<Type>, custom_type_name: &String) -> ChoiceType {
    // let custom_type_name = custom_type.name.as_ref().unwrap();
    for r#type in types {
        match r#type {
            Type::Choice(_attributes, custom_type_cmp) => {
                let name = custom_type_cmp.name.as_ref().unwrap();
                if *custom_type_name == name.tok {
                    return custom_type_cmp.clone();
                }
            },
            _ => panic!("{:?} is not a choice type", r#type)          
        };
    }
    panic!("Could not find type {}", custom_type_name);
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
                        assert!(root, "inner functions are not supported currently\nCurrent_token: {:?}", peek);
                        let proc = create_proc(ast, ast.scopes[block.scope].id, name);
                        block.statements.push(Statement::Declaration(DeclNode::Proc(proc)));
                    } else if peek.tt == TokenType::Type {
                        ast.advance();
                        let mut new_type = extract_type(ast);
                        match &mut new_type {
                            Type::Custom(_, custom) => custom.name = Some(name),
                            Type::Choice(_, choice) => choice.name = Some(name),
                            _ => unreachable!()
                        }
                        ast.types.push(new_type);
                    } else {
                        ast.advance();
                        let variable_decl = create_variable_declaration(ast, block.scope, name, false);
                        ast.scopes[block.scope].map.insert(variable_decl.symbol.tok.clone(), variable_decl.clone());
                        block.statements.push(Statement::Declaration(DeclNode::Var(variable_decl)));
                    }
                } else if is_operator(&peek.tt) || peek.tt == TokenType::OpenParen {
                    if root {
                        // for debug purposes
                        println!("{:?}", current_token);
                    }
                    assert!(!root, "Expressions are not allowed in the top level scope");
                    // expression statement
                    let expr = create_expr(ast, ast.scopes[block.scope].id);
                    block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
                } else {
                    unreachable!("{:?}:{:?}", ast.get_current_token().unwrap(), ast.get_peek().unwrap());
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
            },
            TokenType::If => {
                assert!(!root, "if statements are not allowed in the top level scope");
                block.statements.push(Statement::ExpressionStatement(
                    ExpressionStatement::ExpressionWithBlock(
                        ExpressionStatementWithBlock::If(
                            create_if(ast, ast.scopes[block.scope].id))
                        )
                ));
            },
            TokenType::Include => {
                ast.advance();
                let filename = ast.get_current_token().unwrap().tok.clone();
                let mut dir: Vec<&str> = ast.tokens[0].filename.split('/').collect();
                if dir.len() > 1 {
                    dir.pop();
                }
                dir.push(&filename);
                let filepath = dir.join("/");
                let src = match std::fs::read_to_string(filepath.clone()) {
                    Ok(src) => src,
                    Err(e) => panic!("Could not include \"{}\": ({})", filepath, e)
                };
                ast.advance();
                ast.match_token(TokenType::SemiColon);
                ast.advance();
                let tokens = tokenize::tokenize_start(&src, &filepath);
                let mut index = ast.index;
                for token in tokens {
                    println!("{:?}", token);
                    ast.tokens.insert(index, token);
                    index += 1;
                }
                print_tokens(&ast.tokens);
                ast.index -= 1; // since we advance at the end of the loop
            },
            TokenType::For => {
                assert!(!root, "for loops are not allowed in the top level scope");
                block.statements.push(Statement::ExpressionStatement(
                    ExpressionStatement::ExpressionWithBlock(
                        ExpressionStatementWithBlock::For(
                            create_for(ast, ast.scopes[block.scope].id))
                        )
                ));
            },
            TokenType::Break => {
                ast.advance();
                ast.match_token(TokenType::SemiColon); // gets skipped at the end of the loop
                block.statements.push(Statement::Break);
            },
            TokenType::Continue => {
                ast.advance();
                ast.match_token(TokenType::SemiColon);
                block.statements.push(Statement::Continue);
            },
            TokenType::Match => {
                // currently only choice matches work TODO: implement others
                let save = current_token.clone(); // saved for debugging purposes
                ast.advance();
                let var = ast.get_current_token().unwrap().clone();
                ast.advance();
                ast.match_token(TokenType::ShortAssign);
                ast.advance();
                let subject_token = ast.get_current_token().unwrap().clone();
                ast.advance();
                ast.match_token(TokenType::OpenCurly);
                ast.advance();
                let subject = match lookup_var(ast, block.scope, &subject_token.tok) {
                    Some(var) => var,
                    None => panic!("Could not find \"{}\"", subject_token.tok)
                };
                let mut needs_deref = false;
                let fields;
                match subject.type_ {
                    Type::Choice(ref _attributes, ref choice_type) => {
                        fields = choice_type.fields.clone();
                    },
                    Type::Custom(ref _attributes, ref custom_type) => {
                        // check if actually a custom type or if its a choice type
                        let r#type = get_ast_type(&ast.types, &custom_type.name.as_ref().unwrap().tok);
                        match r#type {
                            Type::Choice(_attributes, choice_type) => {
                                fields = choice_type.fields;
                            }
                            _ => {
                                println!("{:?}", r#type);
                                panic!("Currently only support for choice types in matches")
                            }
                        }
                    },
                    Type::Pointer(ref type_) => {
                        needs_deref = true;
                        match &**type_ {
                            Type::Choice(_attributes, choice_type) => {
                                fields = choice_type.fields.clone();
                            },
                            Type::Custom(_attributes, custom_type) => {
                                // check if actually a custom type or if its a choice type
                                let r#type = get_ast_type(&ast.types, &custom_type.name.as_ref().unwrap().tok);
                                match r#type {
                                    Type::Choice(_attributes, choice_type) => {
                                        fields = choice_type.fields;
                                    }
                                    _ => {
                                        println!("{:?}", r#type);
                                        panic!("Currently only support for choice types in matches")
                                    }
                                }
                            },
                            _ => {
                                println!("{:?}", subject.type_);
                                panic!("cannot match on a non-choice type variable for the moment")
                            }
                        }
                    }
                    _ => {
                        println!("{:?}", subject.type_);
                        panic!("cannot match on a non-choice type variable for the moment")
                    }
                }
                let mut blocks = Vec::<BlockNode>::new();
                loop {
                    let current_token = ast.get_current_token().unwrap();
                    if current_token.tt == TokenType::CloseCurly {
                        break;
                    }
                    // TODO: make sure fields match whats being created
                    let field_tok = ast.get_current_token().unwrap().clone();
                    let field_type = extract_type(ast);
                    let mut index = -1;
                    for (i, field) in fields.iter().enumerate() {
                        println!("{:?}, {:?}, {}", field.1.type_id(), field_type.type_id(), fields.len());
                        println!("{:?}: {:?}", field.1, field_type);
                        match (&field.1, &field_type) {
                            (Type::VoidPtr(_), Type::VoidPtr(_)) => index = i as i32,
                            (Type::U8(_), Type::U8(_)) => index = i as i32,
                            (Type::U16(_), Type::U16(_)) => index = i as i32,
                            (Type::U32(_), Type::U32(_)) => index = i as i32,
                            (Type::U64(_), Type::U64(_)) => index = i as i32,
                            (Type::I8(_), Type::I8(_)) => index = i as i32,
                            (Type::I16(_), Type::I16(_)) => index = i as i32,
                            (Type::I32(_), Type::I32(_)) => index = i as i32,
                            (Type::I64(_), Type::I64(_)) => index = i as i32,
                            (Type::Bool(_), Type::Bool(_)) => index = i as i32,
                            (Type::String(_), Type::String(_)) => index = i as i32,
                            (Type::Proc(_, _, _), Type::Proc(_, _, _)) => index = i as i32,
                            (Type::Pointer(_), Type::Pointer(_)) => index = i as i32,
                            (Type::DynamicArray(_), Type::DynamicArray(_)) => index = i as i32,
                            (Type::Array(_), Type::Array(_)) => index = i as i32,
                            (Type::Slice(_), Type::Slice(_)) => index = i as i32,
                            // TODO: for choices with all custom structs the tags are incorrectly all set to
                            // the last index e.g. all tags of a choice type with 6 structs will have index set to 6 for
                            // all of them which is obviously not desirable. this is where it needs to be fixed!
                            // TODO: the actual names need to be compared
                            (Type::Custom(_field_type_name, fields), Type::Custom(_member_type_name, member_fields)) => {
                                if fields.name.as_ref().unwrap().tok == member_fields.name.as_ref().unwrap().tok {
                                    index = i as i32;
                                }
                            },
                            (Type::Choice(_, _), Type::Choice(_, _)) => todo!(""),
                            _ => {},
                        }
                    }
                    if index == -1 {
                        println!("{:?}", field_tok);
                        panic!("Field \"{}\" does not exist for \"{}\"", field_tok.tok, subject.symbol.tok)
                    }
                    ast.advance();
                    ast.match_token(TokenType::FatArrow);
                    ast.advance();
                    ast.match_token(TokenType::OpenCurly);
                    ast.advance();
                    let mut field_block = create_block(ast, root, Some(block.scope));
                    let field_scope = &mut ast.scopes[field_block.scope];
                    let new_var = VariableDeclNode {
                        symbol: var.clone(),
                        rhs: Some(Expression::Reference(Box::new(Expression::Binary(BinaryOpNode {
                            lhs: Box::new(Expression::Variable(VariableNode { symbol: subject.symbol.clone() })),
                            rhs: Box::new(Expression::Variable(VariableNode { symbol: create_token(-1, -1, format!("_{}", index), "generated".to_string()) })),
                            op: Operation::Access
                        })))),
                        type_: Type::Pointer(Box::new(field_type)),
                        is_arg: false
                    };
                    field_scope.map.insert(var.tok.clone(), new_var.clone());
                    field_block.statements.insert(0, Statement::Declaration(DeclNode::Var(new_var)));
                    blocks.push(field_block.to_owned());
                    ast.match_token(TokenType::CloseCurly);
                    ast.advance();
                    if ast.get_current_token().unwrap().tt == TokenType::Comma {
                        ast.advance();
                    }
                }
                // TODO: support other kinds of match_types
                block.statements.push(Statement::ExpressionStatement(
                    ExpressionStatement::ExpressionWithBlock(
                        ExpressionStatementWithBlock::Match(
                            MatchNode {
                                match_type: MatchKind::Choice,
                                fields,
                                subject: subject.clone(),
                                blocks,
                                needs_deref,
                                token: save
                                // var
                            }
                        )
                )));
            }
            _ => todo!("Unexpected Token \"{:?}\":{}", current_token, ast.index),
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
        strings: HashMap::new(),
        procs: vec!()
    };

    ast.root_block = Some(create_block(&mut ast, true, None));
    
    print_ast(&ast);
    return ast;    
}

// TODO: improve error handling
// TODO: improve error messages
