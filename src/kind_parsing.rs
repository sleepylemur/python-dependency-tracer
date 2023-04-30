use rustpython_parser::ast::{self, ExprKind, StmtKind};

pub fn convert_attribute_to_name(node: &ExprKind) -> Option<String> {
    let mut name = String::new();
    let mut node = node;
    loop {
        match node {
            ExprKind::Attribute { value, attr, .. } => {
                if name.len() > 0 {
                    name = format!("{}.{}", attr, name);
                } else {
                    name = attr.to_string();
                }
                node = &value.node;
            }
            ExprKind::Name { id, .. } => {
                name = format!("{}.{}", id, name);
                return Some(name);
            }
            _ => return None,
        }
    }
}

pub fn find_calls_in_expr(node: &ExprKind) -> Vec<String> {
    let mut calls = Vec::new();
    match node {
        ExprKind::BoolOp { op: _, values } => {
            for value in values {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        ExprKind::NamedExpr { target, value } => {
            calls.append(&mut find_calls_in_expr(&target.node));
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        ExprKind::BinOp { left, op: _, right } => {
            calls.append(&mut find_calls_in_expr(&left.node));
            calls.append(&mut find_calls_in_expr(&right.node));
        }
        ExprKind::UnaryOp { op: _, operand } => {
            calls.append(&mut find_calls_in_expr(&operand.node));
        }
        ExprKind::Lambda { args: _, body } => {
            calls.append(&mut find_calls_in_expr(&body.node));
        }
        ExprKind::IfExp { test, body, orelse } => {
            calls.append(&mut find_calls_in_expr(&test.node));
            calls.append(&mut find_calls_in_expr(&body.node));
            calls.append(&mut find_calls_in_expr(&orelse.node));
        }
        ExprKind::Dict { keys, values } => {
            for key in keys {
                calls.append(&mut find_calls_in_expr(&key.node));
            }
            for value in values {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        ExprKind::Slice { lower, upper, step } => {
            if let Some(lower) = lower {
                calls.append(&mut find_calls_in_expr(&lower.node));
            }
            if let Some(upper) = upper {
                calls.append(&mut find_calls_in_expr(&upper.node));
            }
            if let Some(step) = step {
                calls.append(&mut find_calls_in_expr(&step.node));
            }
        }
        ExprKind::Set { elts } => {
            for elt in elts {
                calls.append(&mut find_calls_in_expr(&elt.node));
            }
        }
        ExprKind::ListComp { elt, generators } => {
            calls.append(&mut find_calls_in_expr(&elt.node));
            for generator in generators {
                calls.append(&mut find_calls_in_expr(&generator.iter.node));
                for if_expr in &generator.ifs {
                    calls.append(&mut find_calls_in_expr(&if_expr.node));
                }
            }
        }
        ExprKind::SetComp { elt, generators } => {
            calls.append(&mut find_calls_in_expr(&elt.node));
            for generator in generators {
                calls.append(&mut find_calls_in_expr(&generator.iter.node));
                for if_expr in &generator.ifs {
                    calls.append(&mut find_calls_in_expr(&if_expr.node));
                }
            }
        }
        ExprKind::DictComp {
            key,
            value,
            generators,
        } => {
            calls.append(&mut find_calls_in_expr(&key.node));
            calls.append(&mut find_calls_in_expr(&value.node));
            for generator in generators {
                calls.append(&mut find_calls_in_expr(&generator.iter.node));
                for if_expr in &generator.ifs {
                    calls.append(&mut find_calls_in_expr(&if_expr.node));
                }
            }
        }
        ExprKind::GeneratorExp { elt, generators } => {
            calls.append(&mut find_calls_in_expr(&elt.node));
            for generator in generators {
                calls.append(&mut find_calls_in_expr(&generator.iter.node));
                for if_expr in &generator.ifs {
                    calls.append(&mut find_calls_in_expr(&if_expr.node));
                }
            }
        }
        ExprKind::Await { value } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        ExprKind::Yield { value } => {
            if let Some(value) = value {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        ExprKind::YieldFrom { value } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        ExprKind::Compare {
            left,
            ops: _,
            comparators,
        } => {
            calls.append(&mut find_calls_in_expr(&left.node));
            for comparator in comparators {
                calls.append(&mut find_calls_in_expr(&comparator.node));
            }
        }
        ExprKind::Call {
            func,
            args,
            keywords: _,
        } => {
            if let Some(name) = match &func.node {
                ExprKind::Attribute { .. } => convert_attribute_to_name(&func.node),
                ExprKind::Name { id, ctx: _ } => Some(id.to_string()),
                _ => None,
            } {
                calls.push(name);
            } else {
                calls.append(&mut find_calls_in_expr(&func.node));
            }
            for arg in args {
                calls.append(&mut find_calls_in_expr(&arg.node));
            }
        }
        ExprKind::FormattedValue {
            value,
            conversion: _,
            format_spec,
        } => {
            calls.append(&mut find_calls_in_expr(&value.node));
            if let Some(format_spec) = format_spec {
                calls.append(&mut find_calls_in_expr(&format_spec.node));
            }
        }
        ExprKind::JoinedStr { values } => {
            for value in values {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        ExprKind::Constant { value: _, kind: _ } => {}
        ExprKind::Attribute {
            value,
            attr: _,
            ctx: _,
        } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        ExprKind::Subscript {
            value,
            slice,
            ctx: _,
        } => {
            calls.append(&mut find_calls_in_expr(&value.node));
            calls.append(&mut find_calls_in_expr(&slice.node));
        }
        ExprKind::Starred { value, ctx: _ } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        ExprKind::Name { id: _, ctx: _ } => {}
        ExprKind::List { elts, ctx: _ } => {
            for elt in elts {
                calls.append(&mut find_calls_in_expr(&elt.node));
            }
        }
        ExprKind::Tuple { elts, ctx: _ } => {
            for elt in elts {
                calls.append(&mut find_calls_in_expr(&elt.node));
            }
        }
    }
    calls
}

pub fn find_calls_in_stmt(node: &StmtKind) -> Vec<String> {
    let mut calls = Vec::new();
    match node {
        StmtKind::Match { subject, cases } => {
            calls.append(&mut find_calls_in_expr(&subject.node));
            for case in cases {
                for guard in &case.guard {
                    calls.append(&mut find_calls_in_expr(&guard.node));
                }
                // skipping patterns for now
                for stmt in &case.body {
                    calls.append(&mut find_calls_in_stmt(&stmt.node));
                }
            }
        }
        StmtKind::AsyncFor {
            target,
            iter,
            body,
            orelse,
            type_comment: _,
        } => {
            calls.append(&mut find_calls_in_expr(&target.node));
            calls.append(&mut find_calls_in_expr(&iter.node));
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for stmt in orelse {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::AsyncFunctionDef {
            name: _,
            args: _,
            body,
            decorator_list,
            returns: _,
            type_comment: _,
        } => {
            for decorator in decorator_list {
                calls.append(&mut find_calls_in_expr(&decorator.node));
            }
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::AsyncWith {
            items,
            body,
            type_comment: _,
        } => {
            for item in items {
                calls.append(&mut find_calls_in_expr(&item.context_expr.node));
            }
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::AnnAssign {
            target: _,
            annotation: _,
            value,
            simple: _,
        } => {
            if let Some(value) = value {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        StmtKind::Assert { test, msg: _ } => {
            calls.append(&mut find_calls_in_expr(&test.node));
        }
        StmtKind::Assign {
            targets: _,
            value,
            type_comment: _,
        } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        StmtKind::AugAssign {
            target: _,
            op: _,
            value,
        } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        StmtKind::Break => {}
        StmtKind::ClassDef {
            name: _,
            bases: _,
            keywords: _,
            body,
            decorator_list,
        } => {
            for decorator in decorator_list {
                calls.append(&mut find_calls_in_expr(&decorator.node));
            }
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::Continue => {}
        StmtKind::Delete { targets } => {
            for target in targets {
                calls.append(&mut find_calls_in_expr(&target.node));
            }
        }
        StmtKind::Expr { value } => {
            calls.append(&mut find_calls_in_expr(&value.node));
        }
        StmtKind::For {
            target,
            iter,
            body,
            orelse,
            type_comment: _,
        } => {
            calls.append(&mut find_calls_in_expr(&target.node));
            calls.append(&mut find_calls_in_expr(&iter.node));
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for stmt in orelse {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::FunctionDef {
            name: _,
            args: _,
            body,
            decorator_list,
            returns: _,
            type_comment: _,
        } => {
            for decorator in decorator_list {
                calls.append(&mut find_calls_in_expr(&decorator.node));
            }
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::Global { names: _ } => {}
        StmtKind::If { test, body, orelse } => {
            calls.append(&mut find_calls_in_expr(&test.node));
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for stmt in orelse {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::Import { names: _ } => {}
        StmtKind::ImportFrom {
            module: _,
            names: _,
            level: _,
        } => {}
        StmtKind::Nonlocal { names: _ } => {}
        StmtKind::Pass => {}
        StmtKind::Raise { exc, cause } => {
            if let Some(exc) = exc {
                calls.append(&mut find_calls_in_expr(&exc.node));
            }
            if let Some(cause) = cause {
                calls.append(&mut find_calls_in_expr(&cause.node));
            }
        }
        StmtKind::Return { value } => {
            if let Some(value) = value {
                calls.append(&mut find_calls_in_expr(&value.node));
            }
        }
        StmtKind::Try {
            body,
            handlers,
            orelse,
            finalbody,
        } => {
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for handler in handlers {
                let ast::ExcepthandlerKind::ExceptHandler { body, .. } = &handler.node;
                for stmt in body {
                    calls.append(&mut find_calls_in_stmt(&stmt.node));
                }
            }
            for stmt in orelse {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for stmt in finalbody {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::While {
            test: _,
            body,
            orelse,
        } => {
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
            for stmt in orelse {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
        StmtKind::With {
            items,
            body,
            type_comment: _,
        } => {
            for item in items {
                calls.append(&mut find_calls_in_expr(&item.context_expr.node));
            }
            for stmt in body {
                calls.append(&mut find_calls_in_stmt(&stmt.node));
            }
        }
    }
    calls
}
