use ptree::TreeBuilder;
use rustpython_parser::ast::{Excepthandler, ExprKind, StmtKind};
use rustpython_parser::{ast, parser::parse_program};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Debug)]
struct PyModule {
    name: String,
    path: PathBuf,
    imports: Vec<(String, Vec<String>)>,
    functions: Vec<(String, Vec<String>)>,
}

fn find_calls_in_expr(node: &ExprKind) -> Vec<String> {
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
            if let ExprKind::Name { id, ctx: _ } = &func.node {
                calls.push(id.to_string());
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
        ExprKind::Name { id, ctx: _ } => {
            println!("name id: {:?}", id);
        }
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

fn find_calls_in_stmt(node: &StmtKind) -> Vec<String> {
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

fn parse_module(name: &str, source_code: &str, path: &Path) -> PyModule {
    let mut parsed_module = PyModule {
        name: name.to_string(),
        path: path.to_path_buf(),
        imports: vec![],
        functions: vec![],
    };

    let ast = parse_program(source_code, path.to_str().unwrap()).unwrap();
    for located in ast {
        let node = located.node;
        match &node {
            ast::StmtKind::Import { names } => {
                for import_name in names {
                    let module_name = import_name.node.name.to_string();
                    parsed_module.imports.push((module_name, vec![]));
                }
            }
            ast::StmtKind::ImportFrom {
                level,
                module,
                names,
            } => {
                let module_name = match &module {
                    Some(module) => module.to_string(),
                    None => ".".repeat(level.unwrap_or(0) as usize),
                };
                parsed_module.imports.push((
                    module_name,
                    names.iter().map(|n| n.node.name.to_string()).collect(),
                ));
            }
            ast::StmtKind::FunctionDef {
                name,
                args: _,
                body,
                decorator_list: _,
                returns: _,
                type_comment: _,
            } => {
                let mut calls = vec![];
                for stmt in body {
                    calls.append(&mut find_calls_in_stmt(&stmt.node));
                }
                parsed_module.functions.push((name.to_string(), calls));
            }
            _ => {}
        }
    }
    parsed_module
}

fn get_python_paths(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                paths.extend(get_python_paths(&path)?);
            } else if let Some(extension) = path.extension() {
                if extension == "py" {
                    paths.push(path);
                }
            }
        }
    }
    Ok(paths)
}

fn get_module_name(path: &Path) -> String {
    let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
    let parent = path.parent().unwrap().to_str().unwrap().replace("/", ".");
    format!("{}.{}", parent, stem)
}

fn print_transitive_deps(
    modules: &HashMap<String, PyModule>,
    module_name: &str,
    function_name: Option<&str>,
) -> io::Result<()> {
    // Build a tree representing the transitive dependencies of the module
    let mut tree_builder;
    match function_name {
        Some(function_name) => {
            tree_builder = TreeBuilder::new(format!("{}::{}", module_name, function_name));
            add_function_dependencies_to_tree(
                &mut tree_builder,
                modules,
                module_name,
                function_name,
            )
        }
        None => {
            tree_builder = TreeBuilder::new(module_name.to_string());
            add_module_dependencies_to_tree(&mut tree_builder, modules, module_name)
        }
    }

    // Print the tree
    ptree::print_tree(&tree_builder.build())?;
    Ok(())
}

// Recursively add the dependencies of a module to a tree
fn add_function_dependencies_to_tree(
    tree_builder: &mut TreeBuilder,
    modules: &HashMap<String, PyModule>,
    module_name: &str,
    function_name: &str,
) {
    if let Some(PyModule {
        imports, functions, ..
    }) = modules.get(module_name)
    {
        // build reverse map of imported names to imported modules
        let mut rev_imports = HashMap::new();
        for (module, names) in imports {
            for name in names {
                rev_imports.insert(name, module);
            }
        }

        for (function, calls) in functions {
            if function == function_name {
                // Add the dependencies of the function to the tree
                for call in calls {
                    // Check if the call references an import
                    if let Some(module) = rev_imports.get(call) {
                        // Add the dependency to the tree
                        let child_builder =
                            tree_builder.begin_child(format!("{}::{}", module, call));

                        // Recursively add the dependencies of the dependency to the tree
                        add_function_dependencies_to_tree(child_builder, modules, module, call);

                        tree_builder.end_child();
                    }
                }
            }
        }
    }
}

fn add_module_dependencies_to_tree(
    tree_builder: &mut TreeBuilder,
    modules: &HashMap<String, PyModule>,
    module_name: &str,
) {
    if let Some(PyModule { imports, .. }) = modules.get(module_name) {
        for (module, _) in imports {
            // Add the dependency to the tree
            let child_builder = tree_builder.begin_child(module.to_string());

            // Recursively add the dependencies of the dependency to the tree
            add_module_dependencies_to_tree(child_builder, modules, module);

            tree_builder.end_child();
        }
    }
}

fn build_module_to_paths(base_path: &Path) -> io::Result<HashMap<String, PathBuf>> {
    let python_paths = get_python_paths(base_path)?;

    // build lookup table from python modules to paths
    let mut modules_to_paths = HashMap::new();
    for path in python_paths {
        let relative_path = path.strip_prefix(base_path).unwrap();
        if relative_path.file_name().unwrap() == "__init__.py" {
            modules_to_paths.insert(
                relative_path
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("/", "."),
                path,
            );
        } else {
            modules_to_paths.insert(get_module_name(&relative_path), path);
        }
    }
    Ok(modules_to_paths)
}

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    let base_path = Path::new(&args[1]);
    let module_name = &args[2];
    let function_name = args.get(3).map(|s| s.as_str());

    let modules_to_paths = build_module_to_paths(base_path)?;

    let mut modules = HashMap::new();

    for (module_name, path) in modules_to_paths.iter() {
        let mut source_code = String::new();
        File::open(path)?.read_to_string(&mut source_code)?;
        let module = parse_module(&module_name, &source_code, path);
        modules.insert(module_name.to_string(), module);
    }
    print_transitive_deps(&modules, &module_name, function_name)?;

    Ok(())
}
