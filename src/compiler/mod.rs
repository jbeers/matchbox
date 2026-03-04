use anyhow::{Result, bail};
use crate::ast::{Expression, Literal, Statement, StringPart, FunctionBody, ClassMember};
use crate::types::{BxValue, BxCompiledFunction, BxClass};
use crate::vm::chunk::Chunk;
use crate::vm::opcode::OpCode;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: i32,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: i32,
    is_class: bool,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            is_class: false,
        }
    }

    pub fn compile(mut self, ast: &[Statement]) -> Result<Chunk> {
        for stmt in ast {
            self.compile_statement(stmt)?;
        }
        self.chunk.write(OpCode::OpReturn);
        Ok(self.chunk)
    }

    fn compile_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::ClassDecl { name, members } => {
                let mut constructor_compiler = Compiler::new();
                constructor_compiler.is_class = true;
                constructor_compiler.scope_depth = 1;
                
                let mut methods = HashMap::new();
                
                for member in members {
                    match member {
                        ClassMember::Property(prop_name) => {
                            let null_idx = constructor_compiler.chunk.add_constant(BxValue::Null);
                            constructor_compiler.chunk.write(OpCode::OpConstant(null_idx));
                            let name_idx = constructor_compiler.chunk.add_constant(BxValue::String(prop_name.clone()));
                            constructor_compiler.chunk.write(OpCode::OpSetPrivate(name_idx));
                            constructor_compiler.chunk.write(OpCode::OpPop);
                        }
                        ClassMember::Statement(inner_stmt) => {
                            match inner_stmt {
                                Statement::FunctionDecl { name: func_name, params, body } => {
                                    let mut method_compiler = Compiler::new();
                                    method_compiler.is_class = true;
                                    let func = method_compiler.compile_function(func_name, params, body)?;
                                    methods.insert(func_name.to_lowercase(), Rc::new(func));
                                }
                                _ => {
                                    constructor_compiler.compile_statement(inner_stmt)?;
                                }
                            }
                        }
                    }
                }
                
                constructor_compiler.chunk.write(OpCode::OpReturn);
                
                let class = BxClass {
                    name: name.clone(),
                    constructor: constructor_compiler.chunk,
                    methods,
                };
                
                let class_idx = self.chunk.add_constant(BxValue::Class(Rc::new(RefCell::new(class))));
                self.chunk.write(OpCode::OpConstant(class_idx));
                let name_idx = self.chunk.add_constant(BxValue::String(name.clone()));
                self.chunk.write(OpCode::OpDefineGlobal(name_idx));
                Ok(())
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.chunk.write(OpCode::OpPop);
                Ok(())
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.compile_expression(e)?;
                } else {
                    let null_idx = self.chunk.add_constant(BxValue::Null);
                    self.chunk.write(OpCode::OpConstant(null_idx));
                }
                self.chunk.write(OpCode::OpReturn);
                Ok(())
            }
            Statement::Throw(expr) => {
                if let Some(e) = expr {
                    self.compile_expression(e)?;
                } else {
                    let null_idx = self.chunk.add_constant(BxValue::Null);
                    self.chunk.write(OpCode::OpConstant(null_idx));
                }
                self.chunk.write(OpCode::OpThrow);
                Ok(())
            }
            Statement::TryCatch { try_branch, catches, finally_branch } => {
                // 1. Push Handler
                let push_handler_idx = self.chunk.code.len();
                self.chunk.write(OpCode::OpPushHandler(0));

                // 2. Try block
                self.begin_scope();
                for s in try_branch {
                    self.compile_statement(s)?;
                }
                self.end_scope();

                // 3. Pop Handler (if try finished successfully)
                self.chunk.write(OpCode::OpPopHandler);

                // 4. Jump to finally/end
                let jump_to_finally_idx = self.chunk.code.len();
                self.chunk.write(OpCode::OpJump(0));

                // 5. Catch targets
                let catch_target = self.chunk.code.len();
                let offset = catch_target - push_handler_idx - 1;
                self.chunk.code[push_handler_idx] = OpCode::OpPushHandler(offset);

                // For simplicity, we handle the first catch block only in this POC
                // In a real VM, OpThrow might push the exception value
                if !catches.is_empty() {
                    // For simplicity, we handle the first catch block only in this POC
                    let first_catch = &catches[0];
                    self.begin_scope();
                    // Exception value is on top of stack
                    self.add_local(first_catch.exception_var.clone());
                    for s in &first_catch.body {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();
                } else {
                    // No catch? Rethrow so outer try/catch can handle it
                    self.chunk.write(OpCode::OpThrow);
                }

                // 6. Finally block (simplified: just run it at the end)
                let finally_target = self.chunk.code.len();
                let jump_offset = finally_target - jump_to_finally_idx - 1;
                self.chunk.code[jump_to_finally_idx] = OpCode::OpJump(jump_offset);

                if let Some(finally_stmts) = finally_branch {
                    self.begin_scope();
                    for s in finally_stmts {
                        self.compile_statement(s)?;
                    }
                    self.end_scope();
                }

                Ok(())
            }
            Statement::VariableDecl { name, value } => {
                self.compile_expression(value)?;
                if self.scope_depth > 0 {
                    self.add_local(name.clone());
                } else {
                    let name_idx = self.chunk.add_constant(BxValue::String(name.clone()));
                    self.chunk.write(OpCode::OpDefineGlobal(name_idx));
                }
                Ok(())
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;
                
                let jump_if_false_idx = self.chunk.code.len();
                self.chunk.write(OpCode::OpJumpIfFalse(0));
                self.chunk.write(OpCode::OpPop);

                self.begin_scope();
                for stmt in then_branch {
                    self.compile_statement(stmt)?;
                }
                self.end_scope();

                let jump_idx = self.chunk.code.len();
                self.chunk.write(OpCode::OpJump(0));

                let false_target = self.chunk.code.len();
                let offset = false_target - jump_if_false_idx - 1;
                self.chunk.code[jump_if_false_idx] = OpCode::OpJumpIfFalse(offset);
                
                self.chunk.write(OpCode::OpPop);

                if let Some(else_stmts) = else_branch {
                    self.begin_scope();
                    for stmt in else_stmts {
                        self.compile_statement(stmt)?;
                    }
                    self.end_scope();
                }

                let end_target = self.chunk.code.len();
                let jump_offset = end_target - jump_idx - 1;
                self.chunk.code[jump_idx] = OpCode::OpJump(jump_offset);

                Ok(())
            }
            Statement::ForClassic { init, condition, update, body } => {
                self.begin_scope();
                if let Some(init_stmt) = init {
                    self.compile_statement(init_stmt)?;
                }

                let loop_start = self.chunk.code.len();

                let mut exit_jump = None;
                if let Some(cond_expr) = condition {
                    self.compile_expression(cond_expr)?;
                    let jump_idx = self.chunk.code.len();
                    self.chunk.write(OpCode::OpJumpIfFalse(0));
                    self.chunk.write(OpCode::OpPop);
                    exit_jump = Some(jump_idx);
                }

                for stmt in body {
                    self.compile_statement(stmt)?;
                }

                if let Some(update_expr) = update {
                    self.compile_expression(update_expr)?;
                    self.chunk.write(OpCode::OpPop);
                }

                let loop_end = self.chunk.code.len();
                let offset = loop_end - loop_start + 1;
                self.chunk.write(OpCode::OpLoop(offset));

                if let Some(idx) = exit_jump {
                    let exit_target = self.chunk.code.len();
                    let offset = exit_target - idx - 1;
                    self.chunk.code[idx] = OpCode::OpJumpIfFalse(offset);
                    self.chunk.write(OpCode::OpPop);
                }
                self.end_scope();

                Ok(())
            }
            Statement::FunctionDecl { name, params, body } => {
                let func = self.compile_function(name, params, body)?;
                let func_idx = self.chunk.add_constant(BxValue::CompiledFunction(Rc::new(func)));
                self.chunk.write(OpCode::OpConstant(func_idx));
                let name_idx = self.chunk.add_constant(BxValue::String(name.clone()));
                self.chunk.write(OpCode::OpDefineGlobal(name_idx));
                Ok(())
            }
            Statement::ForLoop { item, index, collection, body } => {
                self.begin_scope();
                
                self.compile_expression(collection)?;
                let collection_slot = self.locals.len();
                self.locals.push(Local { name: "$collection".to_string(), depth: self.scope_depth });

                let zero_idx = self.chunk.add_constant(BxValue::Number(0.0));
                self.chunk.write(OpCode::OpConstant(zero_idx));
                let cursor_slot = self.locals.len();
                self.locals.push(Local { name: "$cursor".to_string(), depth: self.scope_depth });

                let loop_start = self.chunk.code.len();

                let has_index = index.is_some();
                let iter_next_idx = self.chunk.code.len();
                self.chunk.write(OpCode::OpIterNext(collection_slot, cursor_slot, 0, has_index));

                self.add_local(item.clone());
                if let Some(index_name) = index {
                    self.add_local(index_name.clone());
                }

                for stmt in body {
                    self.compile_statement(stmt)?;
                }

                if index.is_some() {
                    self.chunk.write(OpCode::OpPop);
                    self.locals.pop();
                }
                self.chunk.write(OpCode::OpPop);
                self.locals.pop();

                let loop_end = self.chunk.code.len();
                let offset = loop_end - loop_start + 1;
                self.chunk.write(OpCode::OpLoop(offset));

                let exit_target = self.chunk.code.len();
                let offset = exit_target - iter_next_idx - 1;
                self.chunk.code[iter_next_idx] = OpCode::OpIterNext(collection_slot, cursor_slot, offset, has_index);

                self.end_scope();
                Ok(())
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expression) -> Result<()> {
        match expr {
            Expression::New { class_name, args } => {
                let class_idx = self.chunk.add_constant(BxValue::String(class_name.clone()));
                self.chunk.write(OpCode::OpGetGlobal(class_idx));
                
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.chunk.write(OpCode::OpNew(args.len()));
                Ok(())
            }
            Expression::Literal(lit) => match lit {
                Literal::Number(n) => {
                    let idx = self.chunk.add_constant(BxValue::Number(*n));
                    self.chunk.write(OpCode::OpConstant(idx));
                    Ok(())
                }
                Literal::String(parts) => {
                    if parts.is_empty() {
                        let idx = self.chunk.add_constant(BxValue::String("".to_string()));
                        self.chunk.write(OpCode::OpConstant(idx));
                        return Ok(());
                    }
                    self.compile_string_part(&parts[0])?;
                    for i in 1..parts.len() {
                        self.compile_string_part(&parts[i])?;
                        self.chunk.write(OpCode::OpStringConcat);
                    }
                    Ok(())
                }
                Literal::Boolean(b) => {
                    let idx = self.chunk.add_constant(BxValue::Boolean(*b));
                    self.chunk.write(OpCode::OpConstant(idx));
                    Ok(())
                }
                Literal::Null => {
                    let idx = self.chunk.add_constant(BxValue::Null);
                    self.chunk.write(OpCode::OpConstant(idx));
                    Ok(())
                }
                Literal::Array(items) => {
                    for item in items {
                        self.compile_expression(item)?;
                    }
                    self.chunk.write(OpCode::OpArray(items.len()));
                    Ok(())
                }
                Literal::Struct(members) => {
                    for (key_expr, val_expr) in members {
                        match key_expr {
                            Expression::Identifier(name) => {
                                let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                                self.chunk.write(OpCode::OpConstant(idx));
                            }
                            _ => self.compile_expression(key_expr)?,
                        }
                        self.compile_expression(val_expr)?;
                    }
                    self.chunk.write(OpCode::OpStruct(members.len()));
                    Ok(())
                }
                Literal::Function { params, body } => {
                    let func = self.compile_function("anonymous", params, body)?;
                    let func_idx = self.chunk.add_constant(BxValue::CompiledFunction(Rc::new(func)));
                    self.chunk.write(OpCode::OpConstant(func_idx));
                    Ok(())
                }
            },
            Expression::Binary { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                match operator.as_str() {
                    "+" => self.chunk.write(OpCode::OpAdd),
                    "-" => self.chunk.write(OpCode::OpSubtract),
                    "*" => self.chunk.write(OpCode::OpMultiply),
                    "/" => self.chunk.write(OpCode::OpDivide),
                    "&" => self.chunk.write(OpCode::OpStringConcat),
                    "==" => self.chunk.write(OpCode::OpEqual),
                    "!=" => self.chunk.write(OpCode::OpNotEqual),
                    "<" => self.chunk.write(OpCode::OpLess),
                    "<=" => self.chunk.write(OpCode::OpLessEqual),
                    ">" => self.chunk.write(OpCode::OpGreater),
                    ">=" => self.chunk.write(OpCode::OpGreaterEqual),
                    _ => bail!("Unknown operator: {}", operator),
                }
                Ok(())
            }
            Expression::Identifier(name) => {
                let lower_name = name.to_lowercase();
                if lower_name == "this" {
                    let idx = self.chunk.add_constant(BxValue::String("this".to_string()));
                    self.chunk.write(OpCode::OpGetPrivate(idx));
                } else if lower_name == "variables" {
                    let idx = self.chunk.add_constant(BxValue::String("variables".to_string()));
                    self.chunk.write(OpCode::OpGetPrivate(idx));
                } else if let Some(slot) = self.resolve_local(name) {
                    self.chunk.write(OpCode::OpGetLocal(slot));
                } else if self.is_class {
                    let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                    self.chunk.write(OpCode::OpGetPrivate(idx));
                } else {
                    let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                    self.chunk.write(OpCode::OpGetGlobal(idx));
                }
                Ok(())
            }
            Expression::Assignment { target, value } => {
                match target {
                    crate::ast::AssignmentTarget::Identifier(name) => {
                        let lower_name = name.to_lowercase();
                        if lower_name == "variables" {
                            bail!("Cannot assign to the 'variables' scope directly");
                        }
                        self.compile_expression(value)?;
                        if let Some(slot) = self.resolve_local(name) {
                            self.chunk.write(OpCode::OpSetLocal(slot));
                        } else if self.is_class {
                            let name_idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetPrivate(name_idx));
                        } else {
                            let name_idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetGlobal(name_idx));
                        }
                    }
                    crate::ast::AssignmentTarget::Member { base, member } => {
                        self.compile_expression(base)?;
                        self.compile_expression(value)?;
                        let name_idx = self.chunk.add_constant(BxValue::String(member.clone()));
                        self.chunk.write(OpCode::OpSetMember(name_idx));
                    }
                    crate::ast::AssignmentTarget::Index { base, index } => {
                        self.compile_expression(base)?;
                        self.compile_expression(index)?;
                        self.compile_expression(value)?;
                        self.chunk.write(OpCode::OpSetIndex);
                    }
                }
                Ok(())
            }
            Expression::FunctionCall { base, args } => {
                if let Expression::Identifier(name) = base.as_ref() {
                    let lower_name = name.to_lowercase();
                    if lower_name == "println" || lower_name == "echo" {
                        for arg in args {
                            self.compile_expression(arg)?;
                        }
                        self.chunk.write(OpCode::OpPrintln(args.len()));
                        let null_idx = self.chunk.add_constant(BxValue::Null);
                        self.chunk.write(OpCode::OpConstant(null_idx));
                        return Ok(());
                    }
                    if lower_name == "print" {
                        for arg in args {
                            self.compile_expression(arg)?;
                        }
                        self.chunk.write(OpCode::OpPrint(args.len()));
                        let null_idx = self.chunk.add_constant(BxValue::Null);
                        self.chunk.write(OpCode::OpConstant(null_idx));
                        return Ok(());
                    }
                }
                
                if let Expression::MemberAccess { base: member_base, member } = base.as_ref() {
                    self.compile_expression(member_base)?;
                    for arg in args {
                        self.compile_expression(arg)?;
                    }
                    let name_idx = self.chunk.add_constant(BxValue::String(member.clone()));
                    self.chunk.write(OpCode::OpInvoke(name_idx, args.len()));
                    return Ok(());
                }

                self.compile_expression(base)?;
                for arg in args {
                    self.compile_expression(arg)?;
                }
                self.chunk.write(OpCode::OpCall(args.len()));
                Ok(())
            }
            Expression::ArrayAccess { base, index } => {
                self.compile_expression(base)?;
                self.compile_expression(index)?;
                self.chunk.write(OpCode::OpIndex);
                Ok(())
            }
            Expression::MemberAccess { base, member } => {
                self.compile_expression(base)?;
                let name_idx = self.chunk.add_constant(BxValue::String(member.clone()));
                self.chunk.write(OpCode::OpMember(name_idx));
                Ok(())
            }
            Expression::Prefix { operator, target } => {
                match target {
                    crate::ast::AssignmentTarget::Identifier(name) => {
                        self.compile_expression(&Expression::Identifier(name.clone()))?;
                        if operator == "++" {
                            self.chunk.write(OpCode::OpInc);
                        } else {
                            self.chunk.write(OpCode::OpDec);
                        }
                        
                        // Set back
                        if let Some(slot) = self.resolve_local(name) {
                            self.chunk.write(OpCode::OpSetLocal(slot));
                        } else if self.is_class {
                            let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetPrivate(idx));
                        } else {
                            let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetGlobal(idx));
                        }
                    }
                    crate::ast::AssignmentTarget::Member { base, member } => {
                        self.compile_expression(base)?;
                        self.chunk.write(OpCode::OpDup);
                        let name_idx = self.chunk.add_constant(BxValue::String(member.clone()));
                        self.chunk.write(OpCode::OpMember(name_idx));
                        if operator == "++" {
                            self.chunk.write(OpCode::OpInc);
                        } else {
                            self.chunk.write(OpCode::OpDec);
                        }
                        self.chunk.write(OpCode::OpSetMember(name_idx));
                    }
                    crate::ast::AssignmentTarget::Index { base, index } => {
                        self.compile_expression(base)?;
                        self.compile_expression(index)?;
                        self.chunk.write(OpCode::OpDup);
                        // Wait, I need base AND index.
                        // [base, index] -> [base, index, base, index] would be better.
                        // Let's just implement for Identifier and Member for now if too complex.
                        // Actually, I can just use temporary locals or more DUPs.
                        // For now, let's bail on Index prefix/postfix if not easy.
                        bail!("Prefix ops on indexed targets not yet implemented");
                    }
                }
                Ok(())
            }
            Expression::Postfix { base, operator } => {
                // Postfix is more complex because we need to return original value
                match base.as_ref() {
                    Expression::Identifier(name) => {
                        self.compile_expression(base)?; // [val]
                        self.chunk.write(OpCode::OpDup); // [val, val]
                        if operator == "++" {
                            self.chunk.write(OpCode::OpInc); // [val, val+1]
                        } else {
                            self.chunk.write(OpCode::OpDec); // [val, val-1]
                        }
                        // Set back
                        if let Some(slot) = self.resolve_local(name) {
                            self.chunk.write(OpCode::OpSetLocal(slot)); // [val, val+1]
                        } else if self.is_class {
                            let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetPrivate(idx));
                        } else {
                            let idx = self.chunk.add_constant(BxValue::String(name.clone()));
                            self.chunk.write(OpCode::OpSetGlobal(idx));
                        }
                        self.chunk.write(OpCode::OpPop); // [val]
                    }
                    Expression::MemberAccess { base: member_base, member } => {
                        self.compile_expression(member_base)?; // [obj]
                        self.chunk.write(OpCode::OpDup); // [obj, obj]
                        let name_idx = self.chunk.add_constant(BxValue::String(member.clone()));
                        self.chunk.write(OpCode::OpMember(name_idx)); // [obj, val]
                        self.chunk.write(OpCode::OpSwap); // [val, obj]
                        self.chunk.write(OpCode::OpOver); // [val, obj, val]
                        if operator == "++" {
                            self.chunk.write(OpCode::OpInc); // [val, obj, val+1]
                        } else {
                            self.chunk.write(OpCode::OpDec); // [val, obj, val-1]
                        }
                        self.chunk.write(OpCode::OpSetMember(name_idx)); // [val, val+1]
                        self.chunk.write(OpCode::OpPop); // [val]
                    }
                    _ => bail!("Postfix ops only supported on identifiers and members currently"),
                }
                Ok(())
            }
        }
    }

    fn compile_string_part(&mut self, part: &StringPart) -> Result<()> {
        match part {
            StringPart::Text(t) => {
                let idx = self.chunk.add_constant(BxValue::String(t.clone()));
                self.chunk.write(OpCode::OpConstant(idx));
                Ok(())
            }
            StringPart::Expression(expr) => self.compile_expression(expr),
        }
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &FunctionBody) -> Result<BxCompiledFunction> {
        let mut sub_compiler = Compiler::new();
        sub_compiler.scope_depth = 1;
        sub_compiler.is_class = self.is_class;
        
        for param in params {
            sub_compiler.locals.push(Local {
                name: param.clone(),
                depth: 1,
            });
        }

        match body {
            FunctionBody::Block(stmts) => {
                for stmt in stmts {
                    sub_compiler.compile_statement(stmt)?;
                }
                let null_idx = sub_compiler.chunk.add_constant(BxValue::Null);
                sub_compiler.chunk.write(OpCode::OpConstant(null_idx));
                sub_compiler.chunk.write(OpCode::OpReturn);
            }
            FunctionBody::Expression(expr) => {
                sub_compiler.compile_expression(expr)?;
                sub_compiler.chunk.write(OpCode::OpReturn);
            }
        }

        Ok(BxCompiledFunction {
            name: name.to_string(),
            arity: params.len(),
            chunk: sub_compiler.chunk,
        })
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name.to_lowercase() == name.to_lowercase() {
                return Some(i);
            }
        }
        None
    }

    fn add_local(&mut self, name: String) {
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.last() {
            if local.depth > self.scope_depth {
                self.locals.pop();
                self.chunk.write(OpCode::OpPop);
            } else {
                break;
            }
        }
    }
}
