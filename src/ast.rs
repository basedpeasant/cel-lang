
use crate::tokenize::Token;

struct NumberNode {
    val: i64,
}

struct VariableNode {
    symbol: Token,
}

enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

struct BinaryOpNode {
    lhs: Expression,
    rhs: Expression,
    op: Operation
}

struct VariableDeclNode {
    symbol: Token,
    rhs: Option<Expression>,
}

enum Expression {
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
    Expression(Expression),
    Declaration(DeclNode)
}

struct BlockNode {
    statements: Vec<Statement>
}

pub struct Ast {
    root_block: BlockNode,
    tokens: Vec<Token>,
    index: i32
}

fn create_block(ast: &Ast, root: bool) {

    
}

fn ast_create(tokens: Vec<Token>) -> Ast {
    let ast = Ast {
        root_block: BlockNode{ statements: vec!() },
        tokens,
        index: 0,
    };

    
    
    return ast;    
}
