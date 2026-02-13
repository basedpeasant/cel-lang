
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
            g.file.write(self.name.tok.as_bytes()).unwrap();
            g.file.write(b"\n").unwrap();
        }
        g.file.write(b"{\n").unwrap();
        self.block.walk(g);
        g.file.write(b"}\n").unwrap();
    }
}

fn get_c_type(type_: Type) -> String {
    match type_ {
        Type::U8 => "unsigned char".to_string(),
        Type::U16 => "unsigned short".to_string(),
        Type::U32 => "unsigned int".to_string(),
        Type::U64 => "unsigned long".to_string(),
        Type::I8 => "char".to_string(),
        Type::I16 => "short".to_string(),
        Type::I32 => "int".to_string(),
        Type::I64 => "long".to_string(),
        _ => todo!("Type not implemented yet")
    }
}

impl Codegen for VariableDeclNode {
    fn walk(&self, g: &mut Generator) {
        g.file.write(b"\t").unwrap();
        g.file.write(get_c_type(self.type_).as_bytes()).unwrap();
        g.file.write(b" ").unwrap();
        g.file.write(self.symbol.tok.as_bytes()).unwrap();
        if self.rhs.is_some() {
            todo!("handle rhs");
        }
        g.file.write(b";\n").unwrap();
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
            match n {
                Statement::Declaration(decl) => decl.walk(g),
                _ => todo!("not implemented yet")
            }
            n.walk(g);
        }        
    }
}

fn write_start(g: &mut Generator) {
    g.file.write(b"void exit(long code);\n").unwrap();
    g.file.write(b"void _start();\n").unwrap();
    g.file.write(b"void _cel_main();\n").unwrap();
    g.file.write(b"\n").unwrap();
    g.file.write(b"void exit(long code)\n").unwrap();
    g.file.write(b"{\n").unwrap();
    g.file.write(b"\t__asm__ (\n").unwrap();
    g.file.write(b"\t\t\"mov $60, %%rax\\n\"\n").unwrap();
    g.file.write(b"\t\t\"mov %0, %%rdi\\n\"\n").unwrap();
    g.file.write(b"\t\t\"syscall\\n\"\n").unwrap();
    g.file.write(b"\t\t:\n").unwrap();
    g.file.write(b"\t\t: \"r\"(code)\n").unwrap();
    g.file.write(b"\t\t:\"%rax\", \"%rdi\", \"%rcx\", \"%r11\"\n").unwrap();
    g.file.write(b"\t);\n").unwrap();
    g.file.write(b"}\n").unwrap();
    g.file.write(b"\n").unwrap();
    g.file.write(b"void _start()\n").unwrap();
    g.file.write(b"{\n").unwrap();
    g.file.write(b"\t_cel_main();\n").unwrap();
    g.file.write(b"\texit(0);\n").unwrap();
    g.file.write(b"}\n").unwrap();
    g.file.write(b"\n").unwrap();
}

pub fn codegen_start(ast: &Ast) {
    let mut g = Generator {
        file: std::fs::File::options()
            .create(true)
            .append(false)
            .write(true)
            .open("out.c")
            .unwrap(),
        scopes: ast.scopes.clone()
    };
    write_start(&mut g);
    ast.root_block.as_ref().unwrap().walk(&mut g);
    std::process::Command::new("gcc")
        .arg("-nostdlib")
        .arg("out.c")
        .arg("-o")
        .arg("out")
        .output()
        .unwrap();
}
