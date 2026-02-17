
use crate::ast::*;
use std::{fs, io::Write};

struct Generator {
    file: fs::File,
    scopes: Vec<Scope>
}

trait Codegen {
    fn walk(&self, g: &mut Generator);
}

impl Codegen for Statement {
    fn walk(&self, g: &mut Generator) {
        match self {
            Self::ExpressionStatement(expr) => {},
            Self::Declaration(decl) => {
                
            }
        }
    } 
}

impl Codegen for ProcNode {
    fn walk(&self, g: &mut Generator) {
        // TODO: probably should implement mangling
        if self.name.tok == "main" {
            g.file.write(b"void _cel_main()\n").unwrap();
        } else {
            if self.return_type.is_none() {
                g.file.write(b"void ").unwrap();
            } else {
                let c_type = get_c_type(self.return_type.clone().unwrap());
                assert!(c_type.1 == 0, "Array types not supported in procedures");
                g.file.write(c_type.0.as_bytes()).unwrap();
                g.file.write(b" ").unwrap();
            }
            g.file.write(self.name.tok.as_bytes()).unwrap();
            g.file.write(b"(").unwrap();
            for (i, arg) in self.args.iter().enumerate() {
                arg.walk(g);
                if i != self.args.len() - 1 {
                    g.file.write(b", ").unwrap();
                }
            }
            g.file.write(b")").unwrap();
            g.file.write(b"\n").unwrap();
        }
        g.file.write(b"{\n").unwrap();
        self.block.walk(g);
        g.file.write(b"}\n\n").unwrap();
    }
}

fn get_c_type(r#type: Type) -> (String, usize) {
    match r#type {
        Type::U8 => ("unsigned char".to_string(), 0),
        Type::U16 =>("unsigned short".to_string(), 0),
        Type::U32 =>("unsigned int".to_string(), 0),
        Type::U64 =>("unsigned long".to_string(), 0),
        Type::I8 => ("char".to_string(), 0),
        Type::I16 =>("short".to_string(), 0),
        Type::I32 =>("int".to_string(), 0),
        Type::I64 =>("long".to_string(), 0),
        Type::Array(arr) => {
            let str = get_c_type(*arr.1);
            return (str.0, arr.0)
        },
        _ => todo!("Type not implemented yet")
    }
}

impl Codegen for Expression {
    fn walk(&self, g: &mut Generator) {
        // TODO: implement constant folding
        match self {
            Expression::Binary(bin) => {
                bin.lhs.walk(g);
                let _ = match bin.op {
                    Operation::Add => g.file.write(b" + ").unwrap(),
                    Operation::Div => g.file.write(b" / ").unwrap(),
                    Operation::Sub => g.file.write(b" - ").unwrap(),
                    Operation::Mul => g.file.write(b" * ").unwrap(),
                    Operation::Assign => g.file.write(b" = ").unwrap(),
                    Operation::ArrayIndex => unreachable!("Array index should not be in a binary operation")
                };
                bin.rhs.walk(g);
            },
            Expression::Number(num) => {
                g.file.write(num.val.to_string().as_bytes()).unwrap();
            },
            Expression::Variable(var) => {
                g.file.write(var.symbol.tok.as_bytes()).unwrap();
            },
            Expression::Call(call) => {
                g.file.write(format!("{}(", call.name.tok).as_bytes()).unwrap();
                if call.args.len() > 0 {
                    for (i, arg) in call.args.iter().enumerate() {
                        arg.walk(g);
                        if i != call.args.len() - 1 {
                            g.file.write(b", ").unwrap();
                        }
                    }
                }
                g.file.write(b")").unwrap();
            },
            Expression::String(str) => {
                g.file.write(format!("\"{}\"", str.str).as_bytes()).unwrap();
            },
            Expression::Array(arr) => {
                g.file.write(b"{").unwrap();
                for i in 0..arr.size.unwrap() {
                    if i < arr.elements.len() {
                        arr.elements[i].walk(g);
                    } else {
                        g.file.write(b"0").unwrap();
                    }
                    if i != arr.size.unwrap() - 1 {
                        g.file.write(b",").unwrap();
                    }
                }
                
                g.file.write(b"}").unwrap();
            },
            Expression::Index(index) => {
                index.base.walk(g);
                g.file.write(b"[").unwrap();
                index.index.walk(g);
                g.file.write(b"]").unwrap();
            }
        } 
    }
}

impl Codegen for VariableDeclNode {
    fn walk(&self, g: &mut Generator) {
        let c_type = get_c_type(self.type_.clone());
        g.file.write(c_type.0.as_bytes()).unwrap();
        g.file.write(b" ").unwrap();
        g.file.write(self.symbol.tok.as_bytes()).unwrap();
        if c_type.1 > 0 {
            g.file.write(format!("[{}]", c_type.1).as_bytes()).unwrap();
        }
        if self.rhs.is_some() {
            g.file.write(b" = ").unwrap();
            self.rhs.as_ref().unwrap().walk(g);
        } else {
            match &self.type_ {
                Type::Array((_, _)) => {
                    g.file.write(b" = ").unwrap();
                    g.file.write(b"{").unwrap();
                    for i in 0..c_type.1 {
                        g.file.write(b"0").unwrap();
                        if i != c_type.1 - 1 {
                            g.file.write(b",").unwrap();
                        }
                    }
                    g.file.write(b"}").unwrap();
                }
                _ => {}
            }
        }
    }
}

impl Codegen for DeclNode {
    fn walk(&self, g: &mut Generator) {
        match self {
            DeclNode::Proc(proc) => proc.walk(g),
            DeclNode::Var(var) => var.walk(g)
        }
    }
}

impl Codegen for BlockNode {
    fn walk(&self, g: &mut Generator) {
        for n in &self.statements {
            if self.scope != 0 {
                g.file.write(b"\t").unwrap();
            }
            match n {
                Statement::Declaration(decl) => {
                    decl.walk(g);
                    match decl {
                        DeclNode::Proc(_) => {},
                        DeclNode::Var(_) => {
                            g.file.write(b";\n").unwrap();
                        }
                    }
                },
                Statement::ExpressionStatement(expr) => {
                    match expr {
                        ExpressionStatement::Expression(expr) => {
                             expr.walk(g);
                             g.file.write(b";\n").unwrap();
                        },
                        ExpressionStatement::ExpressionWithBlock(expr) => todo!(),
                        ExpressionStatement::Return(ret) => {
                             g.file.write(b"return ").unwrap();
                             ret.expr.walk(g);
                             g.file.write(b";\n").unwrap();
                        },
                        ExpressionStatement::Defer(expr) => {
                            expr.walk(g);
                            g.file.write(b";\n").unwrap();
                        }
                    }
                },
                _ => todo!("not implemented yet")
            }
            n.walk(g);
        }        
    }
}

fn write_start(g: &mut Generator) {
    g.file.write(r#"
extern void exit(int code);
extern int write(int fd, const char* buf, int count);
extern int printf(const char* fmt, ...);
void _start();
void _cel_main();
int writei(int fd, int number); // TODO: implement in Cel

// TODO: implement this in Cel; this is just for debugging/convenience
int writei(int fd, int number) {
    char rev_buf[20] = {0};
    char buf[20] = {0};

    int tmp = number;
    int count = 0;
    while (tmp != 0) {
        rev_buf[count] = '0' + tmp % 10;
        tmp /= 10;
        count++;
    }

    for (int i = count; i >= 0; i--) {
        buf[i] = rev_buf[count - i - 1];
    }

    write(1, buf, count);
    return count;
}

int main()
{
    _cel_main();
    exit(0);
}

"#.as_bytes()).unwrap();
}

pub fn codegen_start(ast: &Ast) {
    let mut g = Generator {
        file: std::fs::File::options()
            .write(true)
            .create(true)
            .open("out.c")
            .unwrap(),
        scopes: ast.scopes.clone()
    };
    write_start(&mut g);
    ast.root_block.as_ref().unwrap().walk(&mut g);
    let output = std::process::Command::new("cc")
        // .arg("-ffreestanding")
        // .arg("-nostdlib")
        .arg("out.c")
        .arg("-o")
        .arg("out")
        .output()
        .unwrap();
    println!("{}", str::from_utf8(&output.stderr).unwrap());
}
