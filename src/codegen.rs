
use crate::ast::*;
use std::{collections::HashMap, fmt::Formatter, fs, io::Write};

struct Generator {
    file: fs::File,
    current_scope: usize,
    scopes: Vec<Scope>,
    procs: Vec<ProcNodeHeader>,
    types: Vec<Type>,
    indentation_level: u8,
    strings: HashMap<String, StringLiteral>,
    string_map: HashMap<String, String>,
    main_declaration: String
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
            g.file.write(g.main_declaration.as_bytes()).unwrap();
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
        Type::Bool(_) =>("bool".to_string(), 0),
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
        Type::DynamicArray(arr) => {
            let mut str = get_c_type(*arr);
            str.0.push_str("*");
            return (str.0, 0)
        },
        Type::Pointer(ptr) => (format!("{}{}", get_c_type(ptr.as_ref().clone()).0, "*"), 0),
        Type::Custom(_, custom) => (custom.name.unwrap().tok, 0),
        _ => todo!("Type not implemented yet")
    }
}

fn lookup_custom_type(types: &Vec<Type>, custom_type_name: &String) -> Option<CustomType> {
    // let custom_type_name = custom_type.name.as_ref().unwrap();
    for r#type in types {
        match r#type {
            Type::Custom(_attributes, custom_type_cmp) => {
                let name = custom_type_cmp.name.as_ref().unwrap();
                if *custom_type_name == name.tok {
                    return Some(custom_type_cmp.clone());
                }
            },
            _ => return None,
            // _ => panic!("{:?} is not a custom type", r#type)          
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
            _ => {}
        };
    }
    panic!("Could not find type {}", custom_type_name);
}

fn get_type(types: &Vec<Type>, custom_type_name: &String) -> Type {
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

fn handle_member_access(g: &mut Generator, var_decl: VariableDeclNode, bin: &BinaryOpNode, start: bool) {
    if bin.op != Operation::Access {
        return;
    }
    if start {
        g.file.write(var_decl.symbol.tok.as_bytes()).unwrap();
        match &var_decl.type_ {
            Type::Pointer(_) => g.file.write(b"->").unwrap(),
            _ => g.file.write(b".").unwrap(),
        };
    } else {
        let member_symbol;
        // Member of a variable/struct
        match bin.lhs.as_ref() {
            Expression::Variable(var) => {
                member_symbol = var.symbol.clone();
                bin.lhs.walk(g);
            },
            _ => unreachable!("This should not happen")
        }
        // TODO: needs to check if the member is a pointer or not
        // this doesn't do that
        match &var_decl.type_ {
            Type::Custom(_attributes, custom_type) => {
                println!("{:?}", var_decl);
                let var_type = lookup_custom_type(&g.types, &custom_type.name.as_ref().unwrap().tok);
                let mut success = false;
                if var_type.is_some() {
                    for r#type in &var_type.unwrap().fields {
                        if member_symbol.tok == r#type.0.tok {
                            match r#type.1 {
                                Type::Pointer(_) => g.file.write(b"->").unwrap(),
                                _ => g.file.write(b".").unwrap()
                            };
                            success = true;
                        }
                    }
                } else {
                    panic!("Could not find type for {}", &custom_type.name.as_ref().unwrap().tok);
                }
                if !success {
                    panic!("Member \"{}\" is not found in variable \"{}\"", member_symbol.tok, var_decl.symbol.tok);
                }
            },
            _ => todo!()
        };
    }

    match bin.rhs.as_ref() {
        Expression::Binary(bin_rhs) => {
            if bin.op == Operation::Access {
                handle_member_access(g, var_decl, bin_rhs, false);
            } else {
                bin.rhs.walk(g); // TODO: might be wrong
            }
        },
        _ => bin.rhs.walk(g)
    }
}

impl Codegen for Expression {
    fn walk(&self, g: &mut Generator) {
        // TODO: implement constant folding
        match self {
            Expression::Binary(bin) => {
                if bin.op == Operation::Access {
                    let mut symbol: Option<String> = None;
                    let mut skip = false;
                    match bin.lhs.as_ref() {
                        Expression::Variable(var) => symbol = Some(var.symbol.tok.clone()),
                        Expression::String(_) => {
                            bin.lhs.walk(g);
                            skip = true;
                            g.file.write(b".").unwrap();
                        },
                        other => {
                            println!("{:?}", other);
                            panic!("Can only apply member access to variables")  
                        }
                    }
                    if !skip {
                        let var_decl = g.lookup_var(&symbol.as_ref().unwrap())
                            .unwrap_or_else(|| panic!("symbol \"{}\" could not be found", symbol.unwrap()));
                        handle_member_access(g, var_decl.clone(), bin, true);
                    } else {
                        bin.rhs.walk(g);
                    }
                    // bin.rhs.walk(g); // NOTE: might need it if at the end of ^ it is a non Access binary operation
                    return;
                }
                g.file.write(b"(").unwrap();
                bin.lhs.walk(g);
                match bin.op {
                    Operation::Add => g.file.write(b" + ").unwrap(),
                    Operation::Div => g.file.write(b" / ").unwrap(),
                    Operation::Sub => g.file.write(b" - ").unwrap(),
                    Operation::Mul => g.file.write(b" * ").unwrap(),
                    Operation::Assign => g.file.write(b" = ").unwrap(),
                    Operation::Or => g.file.write(b" | ").unwrap(),
                    Operation::LogicalOr => g.file.write(b" || ").unwrap(),
                    Operation::LogicalAnd => g.file.write(b" && ").unwrap(),
                    Operation::NotEqual => g.file.write(b" != ").unwrap(),
                    Operation::Equal => g.file.write(b" == ").unwrap(),
                    Operation::Gte => g.file.write(b" >= ").unwrap(),
                    Operation::Gt => g.file.write(b" > ").unwrap(),
                    Operation::Lte => g.file.write(b" <= ").unwrap(),
                    Operation::Lt => g.file.write(b" < ").unwrap(),
                    Operation::Access => unreachable!("Acccess operation is handled somewhere else"),
                    Operation::Reference => g.file.write(b"&").unwrap(),
                    Operation::And => g.file.write(b"&").unwrap(),
                    Operation::Not => g.file.write(b"!").unwrap(),
                    Operation::Mod => g.file.write(b"%").unwrap(),
                    Operation::ArrayIndex => unreachable!("Array index should not be in a binary operation")
                };
                bin.rhs.walk(g);
                g.file.write(b")").unwrap();
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
                let var_type = get_type(&g.types, &struct_declaration.name.as_ref().unwrap().tok);
                // check if it is a choice or a struct;
                match var_type {
                    Type::Custom(_attributes, _custom_type) => {
                        g.file.write(b"{").unwrap();
                        for (i, expr) in struct_declaration.exprs.iter().enumerate() {
                            expr.walk(g);
                            if i != struct_declaration.exprs.len() - 1 {
                                g.file.write(b", ").unwrap();
                            }
                        }
                
                        g.file.write(b"}").unwrap();
                    },
                    Type::Choice(_attributes, choice_type) => {
                        assert!(struct_declaration.exprs.len() == 1, "Too many fields in choice construct");
                        let expr = struct_declaration.exprs[0].clone();
                        let choice = lookup_choice_type(&g.types, &choice_type.name.as_ref().unwrap().tok);
                        g.file.write(b"{").unwrap();
                        let mut tag: i32 = -1;
                        for (i, field) in choice.fields.iter().enumerate() {
                            match &expr {
                                Expression::Number(_) => {
                                    match field.1 {
                                        Type::U8(_) =>  tag = i as i32,
                                        Type::U16(_) => tag = i as i32,
                                        Type::U32(_) => tag = i as i32,
                                        Type::U64(_) => tag = i as i32,
                                        Type::I8(_) =>  tag = i as i32,
                                        Type::I16(_) => tag = i as i32,
                                        Type::I32(_) => tag = i as i32,
                                        Type::I64(_) => tag = i as i32,
                                        _ => {},
                                    }
                                },
                                Expression::String(_) => {
                                    match field.1 {
                                        Type::String(_) => tag = i as i32,
                                        _ => {},
                                    }
                                },
                                Expression::Variable(var) => {
                                    // let var_type = lookup_custom_type(&g.types, &var.symbol.tok);
                                    let var_decl = g.lookup_var(&var.symbol.tok)
                                        .unwrap_or_else(|| panic!("symbol \"{}\" could not be found", var.symbol.tok));
                                    match (&field.1, &var_decl.type_) {
                                        (Type::U8(_), Type::U8(_)) =>  tag = i as i32,
                                        (Type::U16(_), Type::U16(_)) => tag = i as i32,
                                        (Type::U32(_), Type::U32(_)) => tag = i as i32,
                                        (Type::U64(_), Type::U64(_)) => tag = i as i32,
                                        (Type::I8(_), Type::I8(_)) =>  tag = i as i32,
                                        (Type::I16(_), Type::I16(_)) => tag = i as i32,
                                        (Type::I32(_), Type::I32(_)) => tag = i as i32,
                                        (Type::I64(_), Type::I64(_)) => tag = i as i32,
                                        (Type::VoidPtr(_), Type::VoidPtr(_)) => tag = i as i32,
                                        (Type::String(_), Type::String(_)) => tag = i as i32,
                                        (Type::Bool(_), Type::Bool(_)) => tag = i as i32,
                                        (Type::Array(_), Type::Array(_)) => tag = i as i32,
                                        (Type::Slice(_), Type::Slice(_)) => tag = i as i32,
                                        (Type::Custom(_attributes, field_custom_type), Type::Custom(_attributes2, var_decl_custom_type)) => {
                                            if field_custom_type.name.as_ref().unwrap().tok == var_decl_custom_type.name.as_ref().unwrap().tok {
                                                tag = i as i32;
                                            }
                                        },
                                        (Type::Pointer(_), Type::Pointer(_)) => todo!("implement pointer in this"),
                                        (Type::Choice(..), Type::Choice(..)) => todo!("implement choice in this"),
                                        (Type::DynamicArray(_), Type::DynamicArray(_)) => todo!("implement dynamic arrays in this"),
                                        _ => {},
                                    }

                                }
                                _ => todo!(),
                            }
                        }
                        assert!(tag != -1, "Failed to match expression with choice field");
                        
                        g.file.write(format!("._tag = {}, ._{} = ", tag, tag).as_bytes()).unwrap();
                        expr.walk(g);
                        g.file.write(b"}").unwrap();
                    },
                    _ => panic!("Invalid type for struct declaration rhs"),
                }
            },
            Expression::Reference(expr) => {
                g.file.write(b"&").unwrap();
                expr.walk(g);
            },
            Expression::Not(expr) => {
                g.file.write(b"!").unwrap();
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
                Type::DynamicArray(arr_type) => {
                    if !self.is_arg {
                        g.file.write(b" = ").unwrap();
                        g.file.write(b"array_new(").unwrap();
                        g.file.write(get_c_type(*arr_type.clone()).0.as_bytes()).unwrap();
                        g.file.write(b")").unwrap();
                    }
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
                    if !self.is_arg {
                        g.file.write(b" = {0}").unwrap();
                    }
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

impl Codegen for MatchNode {
    fn walk(&self, g: &mut Generator) {
        //TODO: support other kinds of match
        assert!(self.match_type == MatchKind::Choice);
        if self.needs_deref {
            g.file.write(format!("switch({}->_tag)\n", self.subject.symbol.tok).as_bytes()).unwrap();
        } else {
            g.file.write(format!("switch({}._tag)\n", self.subject.symbol.tok).as_bytes()).unwrap();
        }
        print_indentations(&mut g.file, g.indentation_level);
        g.file.write(b"{\n").unwrap();
        assert!(self.blocks.len() == self.fields.len(), "Number of items must match the number of fields in the choice node for this match: {:?}", self.token);
        for (i, field) in self.fields.iter().enumerate() {
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(format!("case {}: {{\n", i).as_bytes()).unwrap();
            self.blocks[i].walk(g);
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(b"break;\n").unwrap();
            print_indentations(&mut g.file, g.indentation_level);
            g.file.write(b"}\n").unwrap();
        }
        
        print_indentations(&mut g.file, g.indentation_level);
        g.file.write(b"}\n").unwrap();
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
            ExpressionStatementWithBlock::Match(match_node) => match_node.walk(g),
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
                Statement::Break => {
                    g.file.write(b"break;\n").unwrap();
                },
                Statement::Continue => {
                    g.file.write(b"continue;\n").unwrap();
                }
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
#define NULL 0
#define bool __Bool
#define	false	false
#define	true	true
extern int printf(const char* fmt, ...);
extern int sprintf(char* str, const char* format, ...);
void _start();
//NOTE: should be removed later because this is just for debugging
#define TODO(msg) \
        printf("%s:%d: TODO: %s\n", __FILE__, __LINE__, msg.ptr); \
        exit(101);

"#.as_bytes()).unwrap();

    let mut cel_main = "";
    let mut cel_main_call = "";

    for proc in &g.procs {
        if proc.name.tok == "main" {
            if proc.args.len() == 0 {
                cel_main = "void _cel_main();";
                cel_main_call = "    _cel_main();"; 
                g.file.write(cel_main.as_bytes()).unwrap();
            } else if proc.args.len() == 1 {
                // TODO: make sure the types are good, but I don't think is necesssary tbh
                if proc.args[0].symbol.tok == "argc" {
                    cel_main = "void _cel_main(int argc);";
                    cel_main_call = "    _cel_main(argc);";
                    g.file.write(cel_main.as_bytes()).unwrap();
                } else if proc.args[0].symbol.tok == "argv" {
                    cel_main = "void _cel_main(char** argv);"; 
                    cel_main_call = "    _cel_main(argv);";
                    g.file.write(cel_main.as_bytes()).unwrap();
                }
            } else if proc.args.len() == 2 {
                assert!(proc.args[0].symbol.tok == "argc");
                assert!(proc.args[1].symbol.tok == "argv");
                cel_main = "void _cel_main(int argc, char** argv);";
                cel_main_call = "    _cel_main(argc, argv);";
                g.file.write(cel_main.as_bytes()).unwrap();
            }
            break;
        }
    }

    assert!(cel_main != "");
    assert!(cel_main_call != "");
    let cel_main = cel_main.replace(";", "");
    g.main_declaration = cel_main.to_string();
    
    for r#type in &g.types {
        match r#type {
            Type::Custom(attributes, custom) => {
                let attribute_str = String::new();
                for _ in attributes {
                    todo!("Need to think more about how attributes work for structs")
                }
                g.file.write(attribute_str.as_bytes()).unwrap();
                g.file.write(format!("typedef struct {} {};\n", custom.name.as_ref().unwrap().tok, custom.name.as_ref().unwrap().tok).as_bytes()).unwrap();
            },
            Type::Choice(attributes, choice) => {
                let attribute_str = String::new();
                for _ in attributes {
                    todo!("Need to think more about how attributes work for choices")
                }
                g.file.write(attribute_str.as_bytes()).unwrap();
                g.file.write(format!("typedef struct {} {};\n", choice.name.as_ref().unwrap().tok, choice.name.as_ref().unwrap().tok).as_bytes()).unwrap();
            },
            _ => panic!("unexpected type in code generation, only custom types and choice types should be here")
        }
    }

    for r#type in &g.types {
        match r#type {
            Type::Custom(attributes, custom) => {
                let attribute_str = String::new();
                for _ in attributes {
                    todo!("Need to think more about how attributes work for structs")
                }
                g.file.write(attribute_str.as_bytes()).unwrap();
                g.file.write(format!("struct {} {{\n", custom.name.as_ref().unwrap().tok).as_bytes()).unwrap();
                for field in &custom.fields {
                    print_indentations(&mut g.file, g.indentation_level + 1);
                    let c_type = get_c_type(field.1.clone());
                    g.file.write(format!("{} {};\n", c_type.0, field.0.tok).as_bytes()).unwrap();
                }
                g.file.write(b"};\n\n").unwrap();
            },
            Type::Choice(attributes, choice) => {
                let attribute_str = String::new();
                for _ in attributes {
                    todo!("Need to think more about how attributes work for structs")
                }
                g.file.write(attribute_str.as_bytes()).unwrap();
                g.file.write(format!("struct {} {{\n", choice.name.as_ref().unwrap().tok).as_bytes()).unwrap();
                print_indentations(&mut g.file, g.indentation_level + 1);
                g.file.write(b"int _tag;\n").unwrap();
                print_indentations(&mut g.file, g.indentation_level + 1);
                g.file.write(b"union {\n").unwrap();
                let mut index = 0;
                for field in &choice.fields {
                    print_indentations(&mut g.file, g.indentation_level + 2);
                    let c_type = get_c_type(field.1.clone());
                    g.file.write(format!("{} _{};\n", c_type.0, index).as_bytes()).unwrap();
                    index += 1;
                }
                print_indentations(&mut g.file, g.indentation_level + 1);
                g.file.write(b"};\n").unwrap();
                g.file.write(b"};\n\n").unwrap();
            }
            _ => panic!("unexpected type in code generation, only custom types should be here")
        }
    }

    for i in 0..g.procs.len() {
        let proc = g.procs[i].clone();
        if proc.name.tok == "main" {
            continue;
        }
        for attribute in proc.attributes {
            g.file.write(get_c_type_attribute(attribute).as_bytes()).unwrap();
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
            // TODO: we have to fix custom types in arg decalrations so they don't
            // do array initialization
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

    g.file.write(r#"int main(int argc, char** argv)
{
"#.as_bytes()).unwrap();
    g.file.write(cel_main_call.as_bytes()).unwrap();

    g.file.write(r#"
    return 0;
}

#define ARRAY_INITIAL_CAPACITY 10

typedef struct {
    unsigned long type_size;
    unsigned long size;
    unsigned long capacity;
    char data[];
} Array_Metadata;

#define array_append(DA, ITEM) \
    do { \
      Array_Metadata* array_meta = get_meta(DA); \
      if (array_meta->size >= array_meta->capacity) { \
          unsigned long new_capacity = (array_meta->capacity == 0) ? ARRAY_INITIAL_CAPACITY : array_meta->capacity * 2; \
          array_meta = (Array_Metadata*) realloc(array_meta, sizeof(Array_Metadata) + new_capacity * array_meta->type_size); \
          array_meta->capacity = new_capacity; \
          void** tmp = (void**)&DA; \
          *tmp = array_meta->data; \
      } \
      memcpy((char*)DA + array_meta->size * array_meta->type_size, &ITEM, array_meta->type_size); \
      array_meta->size++; \
    } while(0);

#define array_type_size(DA) get_meta(DA)->type_size

#define array_foreach(TYPE, ITEM, DA) \
    for (unsigned long array_i = 0, next = 1; array_i < array_size((void**)&DA); next = !next, array_i++) \
        for(TYPE ITEM = DA[array_i]; next; next = !next)

#define array_new(TYPE) (TYPE*)(_array_new(sizeof(TYPE)))

#define array_size(DA) _array_size((void**)&DA)

// #define array_free(DA) _array_free((void**)&DA)

#define array_free(DA) \
    do { \
        Array_Metadata* array_meta = get_meta(DA); \
        free(array_meta); \
    } while(0);

void* _array_new(unsigned long type_size)
{
    Array_Metadata* meta = (Array_Metadata*) calloc(1, sizeof(Array_Metadata) + ARRAY_INITIAL_CAPACITY * type_size);
    meta->type_size = type_size;
    meta->size = 0;
    meta->capacity = 0;
    return meta->data;
}

Array_Metadata* get_meta(void* da)
{
    return (Array_Metadata*)((char*)da - sizeof(Array_Metadata));
}

unsigned long _array_size(void** da)
{
    Array_Metadata* array_meta = get_meta(*da);
    return array_meta->size;
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
        string_map: HashMap::new(),
        main_declaration: String::new()
    };
    // let string_type = Type::Custom(vec!(), CustomType { name: "string", fields: vec!() }) ;
    // g.types.push();
    write_start(&mut g);
    ast.root_block.as_ref().unwrap().walk(&mut g);
    let output = std::process::Command::new("cc")
        // .arg("-ffreestanding")
        // .arg("-nostdlib")
        .arg("-std=c99")
        .arg("-fno-builtin")
        .arg("-ggdb")
        .arg("out.c")
        .arg("-o")
        .arg("out")
        .output()
        .unwrap();
    println!("{}", str::from_utf8(&output.stderr).unwrap());
}
