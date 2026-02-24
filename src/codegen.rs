
use crate::ast::*;
use std::{collections::HashMap, fs, io::Write};

struct Generator {
    file: fs::File,
    current_scope: usize,
    scopes: Vec<Scope>,
    procs: Vec<ProcNodeHeader>,
    types: Vec<Type>,
    indentation_level: u8,
    strings: HashMap<String, StringLiteral>,
    string_map: HashMap<String, String>
}

impl Generator {
    fn lookup_var(&self, name: &str) -> Option<&VariableDeclNode> {
        let mut scope_id = self.current_scope;

        loop {
            if let Some(decl) = self.scopes[scope_id].map.get(name) {
                return Some(decl);
            }

            match self.scopes[scope_id].parent_scope {
                Some(parent) => scope_id = parent,
                None => return None,
            }
        }
    }
}

trait Codegen {
    fn walk(&self, g: &mut Generator);
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

fn get_c_type_attribute(attribute: Attribute) -> String {
    match attribute {
        Attribute::Extern => "extern ".to_string(),
        Attribute::Static => "static ".to_string(),
        // _ => panic!("unsupported attribute: {:?}", attribute)
    }
}

fn get_c_type(r#type: Type) -> (String, usize) {
    match r#type {
        Type::VoidPtr(_) => ("void*".to_string(), 0),
        Type::U8(_) => ("unsigned char".to_string(), 0),
        Type::U16(_) =>("unsigned short".to_string(), 0),
        Type::U32(_) =>("unsigned int".to_string(), 0),
        Type::U64(_) =>("unsigned long".to_string(), 0),
        Type::I8(_) => ("char".to_string(), 0),
        Type::I16(_) =>("short".to_string(), 0),
        Type::I32(_) =>("int".to_string(), 0),
        Type::I64(_) =>("long".to_string(), 0),
        Type::String(_) =>("string".to_string(), 0),
        Type::Proc(attributes, _args, returns) => {
            assert!(attributes.len() > 0);
            let mut str = String::new();
            for attribute in attributes {
                str.push_str(&get_c_type_attribute(attribute));
            }
            if returns.len() == 0 {
                str.push_str("void");
                return (str, 0)
            }

            let ctype = get_c_type(*returns[0].clone());
            str.push_str(&ctype.0);
            return (str, ctype.1);
        }
        Type::Array(arr) => {
            let str = get_c_type(*arr.1);
            return (str.0, arr.0)
        },
        Type::Pointer(ptr) => (format!("{}{}", get_c_type(ptr.as_ref().clone()).0, "*"), 0),
        Type::Custom(_, custom) => (custom.name.unwrap().tok, 0),
        _ => todo!("Type not implemented yet")
    }
}

impl Codegen for Expression {
    fn walk(&self, g: &mut Generator) {
        // TODO: implement constant folding
        match self {
            Expression::Binary(bin) => {
                bin.lhs.walk(g);
                match bin.op {
                    Operation::Add => g.file.write(b" + ").unwrap(),
                    Operation::Div => g.file.write(b" / ").unwrap(),
                    Operation::Sub => g.file.write(b" - ").unwrap(),
                    Operation::Mul => g.file.write(b" * ").unwrap(),
                    Operation::Assign => g.file.write(b" = ").unwrap(),
                    Operation::Or => g.file.write(b" || ").unwrap(),
                    Operation::NotEqual => g.file.write(b" != ").unwrap(),
                    Operation::Equal => g.file.write(b" == ").unwrap(),
                    Operation::Gte => g.file.write(b" >= ").unwrap(),
                    Operation::Gt => g.file.write(b" > ").unwrap(),
                    Operation::Lte => g.file.write(b" <= ").unwrap(),
                    Operation::Lt => g.file.write(b" < ").unwrap(),
                    Operation::Access => {
                        let mut symbol: Option<String> = None;
                        let mut skip = false;
                        match bin.lhs.as_ref() {
                            Expression::Variable(var) => symbol = Some(var.symbol.tok.clone()),
                            Expression::String(_) => {
                                g.file.write(b".").unwrap();
                                skip = true;
                            },
                            _ => panic!("Can only apply member access to variables")
                        }
                        if !skip {
                            let var_decl = g.lookup_var(&symbol.as_ref().unwrap())
                                .unwrap_or_else(|| panic!("symbol \"{}\" could not be found", symbol.unwrap()));

                            match var_decl.type_ {
                                Type::Pointer(_) => g.file.write(b"->").unwrap(),
                                _ => g.file.write(b".").unwrap(),
                            }
                        } else {
                            0 // added just to satisfy rust error lol
                        }
                    },
                    Operation::Reference => g.file.write(b"&").unwrap(),
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
                g.file.write(format!("{}", g.string_map.get(&str.str.to_string()).unwrap()).as_bytes()).unwrap();
            },
            Expression::Char(char) => {
                g.file.write(format!("'{}'", char.c).as_bytes()).unwrap();
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
            },
            Expression::Struct(struct_declaration) => {
                g.file.write(b"{").unwrap();
                for (i, expr) in struct_declaration.exprs.iter().enumerate() {
                    expr.walk(g);
                    if i != struct_declaration.exprs.len() - 1 {
                        g.file.write(b", ").unwrap();
                    }
                }
                
                g.file.write(b"}").unwrap();
            },
            Expression::Reference(expr) => {
                g.file.write(b"&").unwrap();
                expr.walk(g);
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
                },
                Type::Proc(_attributes, args, _returns) => {
                    g.file.write(b"(").unwrap();
                    for (i, arg) in args.iter().enumerate() {
                        g.file.write(get_c_type(arg.type_.clone()).0.as_bytes()).unwrap();
                        if i != args.len() - 1 {
                            g.file.write(b", ").unwrap();
                        }
                    }
                    g.file.write(b")").unwrap();
                },
                Type::Custom(_attributes, _type) => {
                    g.file.write(b" = {0}").unwrap();
                },
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

impl Codegen for IfNode {
    fn walk(&self, g: &mut Generator) {
        if self.is_else {
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(b"else ").unwrap();
        }
        if self.condition.is_some() {
            g.file.write(b"if (").unwrap();
            self.condition.as_ref().unwrap().walk(g);
            g.file.write(b") {\n").unwrap();
            self.block.walk(g);
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(b"}\n").unwrap();
        } else {
            g.file.write(b"{\n").unwrap();
            self.block.walk(g);
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(b"}\n").unwrap();
        }
        if self.next.is_some() {
            self.next.as_ref().unwrap().walk(g);
        }
    }
}

impl Codegen for ForNode {
    fn walk(&self, g: &mut Generator) {
        if !self.is_classic_for {
            g.file.write(b"while (").unwrap();
            self.condition.walk(g);
            g.file.write(b") {\n").unwrap();
        } else {
            g.file.write(b"for (").unwrap();
            match (*self.init.clone().unwrap()).clone() {
                Statement::Declaration(decl) => {
                    match decl {
                        DeclNode::Var(var) => {
                            var.walk(g);
                        }
                        _ => panic!("You can only declare variables in the pre condition")
                    }
                }
                _ => panic!("Unexpected")
            }
            g.file.write(b";").unwrap();
            self.condition.walk(g);
            g.file.write(b";").unwrap();
            self.post.clone().unwrap().walk(g);
            g.file.write(b") {\n").unwrap();
        }
        self.block.walk(g);
        print_indentations(&mut g.file, g.indentation_level);
        g.file.write(b"}\n").unwrap();
    }
}

impl Codegen for ExpressionStatementWithBlock {
    fn walk(&self, g: &mut Generator) {
        match self {
            ExpressionStatementWithBlock::If(if_node) => if_node.walk(g),
            ExpressionStatementWithBlock::For(for_node) => for_node.walk(g),
        }
    }
}

impl Codegen for BlockNode {
    fn walk(&self, g: &mut Generator) {
        let save = g.current_scope;
        g.current_scope = self.scope;
        g.indentation_level += 1;
        for n in &self.statements {
            if self.scope != 0 {
                print_indentations(&mut g.file, g.indentation_level);
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
                        ExpressionStatement::ExpressionWithBlock(expr) => {
                             expr.walk(g);
                        },
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
        }
        g.indentation_level -= 1;
        g.current_scope = save;      
    }
}

fn print_indentations(g: &mut fs::File, indentation_level: u8) {
    for _ in 0..indentation_level {
        g.write(b"    ").unwrap();
    }
}

fn write_start(g: &mut Generator) {
    g.file.write(r#"
typedef enum {
	false = 0,
	true = 1
} __Bool;
#define bool __Bool
#define	false	false
#define	true	true
extern int printf(const char* fmt, ...);
void _start();
void _cel_main();
typedef struct {
    unsigned int length;
    const char* ptr;
} string;

"#.as_bytes()).unwrap();

    for r#type in &g.types {
        match r#type {
            Type::Custom(attributes, custom) => {
                let attribute_str = String::new();
                for _ in attributes {
                    todo!("Need to think more about how attributes work for structs")
                }
                g.file.write(attribute_str.as_bytes()).unwrap();
                g.file.write(b"typedef struct {\n").unwrap();
                for field in &custom.fields {
                    print_indentations(&mut g.file, g.indentation_level + 1);
                    let c_type = get_c_type(field.1.clone());
                    g.file.write(format!("{} {};\n", c_type.0, field.0.tok).as_bytes()).unwrap();
                }
                g.file.write(format!("}} {};\n\n", custom.name.as_ref().unwrap().tok).as_bytes()).unwrap();
            },
            _ => panic!("unexpected type in code generation, only custom types should be here")
        }
    }

    for i in 0..g.procs.len() {
        let proc = g.procs[i].clone();
        if proc.name.tok == "main" {
            continue;
        }

        if proc.return_type.is_none() {
            g.file.write("void ".as_bytes()).unwrap();
        } else {
            let c_type = get_c_type(proc.return_type.as_ref().unwrap().clone()).0;
            g.file.write(format!("{} ", c_type).as_bytes()).unwrap();
        }
        g.file.write(proc.name.tok.as_bytes()).unwrap();
        g.file.write(b"(").unwrap();

        for (i, arg) in proc.args.iter().enumerate() {
            arg.walk(g);
            if i != proc.args.len() - 1 {
                g.file.write(b", ").unwrap();
            }
        }
        g.file.write(b");\n").unwrap();
    }

    let mut count = 0;
    for str in &g.strings {
        let str = str.0;
        let len = str.len();
        let mapped_name = format!("s{}", count);
        g.file.write(format!("static string {} = {{ {}, \"{}\" }};\n", mapped_name, len, str).as_bytes()).unwrap();
        g.string_map.insert(str.clone(), mapped_name);
        count += 1;
    }

    g.file.write(r#"

int main()
{
    _cel_main();
    return 0;
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
        current_scope: ast.root_block.as_ref().unwrap().scope,
        scopes: ast.scopes.clone(),
        types: ast.types.clone(),
        procs: ast.procs.clone(),
        indentation_level: 0,
        strings: ast.strings.clone(),
        string_map: HashMap::new()
    };
    write_start(&mut g);
    ast.root_block.as_ref().unwrap().walk(&mut g);
    let output = std::process::Command::new("cc")
        // .arg("-ffreestanding")
        // .arg("-nostdlib")
        .arg("-ggdb")
        .arg("out.c")
        .arg("-o")
        .arg("out")
        .output()
        .unwrap();
    println!("{}", str::from_utf8(&output.stderr).unwrap());
}
