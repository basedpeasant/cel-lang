
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
}

enum Statement {
    ExpressionStatement(ExpressionStatement),
    Declaration(DeclNode)
}

struct Scope {
    
}

struct BlockNode {
    statements: Vec<Statement>,
    scope: Scope,
}

pub struct Ast {
    root_block: Option<BlockNode>,
    tokens: Vec<Token>,
    index: usize
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

    fn advance(&mut self) {
        self.index += 1;
    }
}

fn ast_parse_primary(ast: &mut Ast, scope: &Scope) -> Expression {
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

fn get_prec(op: Operation) -> i32 {
    match op {
        Operation::Mul | Operation::Div => 5,
        Operation::Add | Operation::Sub => 4,
        _ => -1
    }
}

fn ast_create_binary(op: Operation, lhs: Expression, rhs: Expression) -> BinaryOpNode {
    return BinaryOpNode { lhs: Box::new(lhs), rhs: Box::new(rhs), op };
}

fn create_expr_with_prec(ast: &mut Ast, min_prec: i32, scope: &Scope) -> Expression {
    let mut lhs = ast_parse_primary(ast, scope);
    loop {
        let op_token = ast.get_current_token();
        if op_token.is_none() {
            return lhs;
        }
        let op = get_op(op_token.unwrap());
        let prec = get_prec(op);
        if prec == -1 {
            break;
        }

        // TODO: check if this actually succeeded
        ast.advance();
        // if success { // TODO: something like this
        //     return lhs
        // }

        let rhs = create_expr_with_prec(ast, prec, scope);

        lhs = Expression::Binary(ast_create_binary(op, lhs, rhs));
    }
    return lhs;
}

fn create_expr(ast: &mut Ast, scope: &Scope) -> Expression {
    return create_expr_with_prec(ast, 0, scope);
}

fn create_block(ast: &mut Ast, root: bool) -> BlockNode {
    let mut block = BlockNode{
        statements: vec!(),
        scope: Scope {}
    };
    
    loop {
        let current_token = ast.get_current_token().unwrap();
        match current_token.tt {
            TokenType::Number | TokenType :: Word => {
                // expression or Decl
                let peek = ast.get_peek().unwrap(); // TODO: handle errors for this later
                if peek.tt == TokenType::Word ||peek.tt == TokenType::ShortAssign || peek.tt == TokenType::DoubleColon {
                    // declaration
                    todo!("handle errors");
                } else if is_operator(&peek.tt) {
                    // expression statement
                    let expr = create_expr(ast, &block.scope);
                    block.statements.push(Statement::ExpressionStatement(ExpressionStatement::Expression(expr)));
                }
            },
            _ => todo!("handle errors"),
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
    };

    ast.root_block = Some(create_block(&mut ast, true));
    
    
    return ast;    
}

// TODO: improve error handling
// TODO: improve error messages
