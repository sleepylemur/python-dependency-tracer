use clap::Parser;
use kind_parsing::find_calls_in_stmt;
use rustpython_parser::{ast, parser::parse_program};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tree::print_transitive_deps;

#[derive(Debug)]
pub struct PyModule {
    name: String,
    path: PathBuf,
    imports: Vec<String>,
    import_froms: Vec<(String, Vec<String>)>,
    functions: Vec<(String, Vec<String>)>,
    classes: Vec<PyClass>,
}

#[derive(Debug)]
pub struct PyClass {
    name: String,
    methods: Vec<(String, Vec<String>)>,
}

impl PyModule {
    fn new(name: &str, path: &Path) -> PyModule {
        PyModule {
            name: name.to_string(),
            path: path.to_path_buf(),
            imports: vec![],
            import_froms: vec![],
            functions: vec![],
            classes: vec![],
        }
    }
}

mod kind_parsing;
mod tree;

fn parse_module(name: &str, source_code: &str, path: &Path) -> PyModule {
    let mut parsed_module = PyModule::new(name, path);

    let ast = parse_program(source_code, path.to_str().unwrap()).unwrap();
    for located in ast {
        let node = located.node;
        match &node {
            ast::StmtKind::Import { names } => {
                for import_name in names {
                    let module_name = import_name.node.name.to_string();
                    parsed_module.imports.push(module_name);
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
                parsed_module.import_froms.push((
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
            }
            | ast::StmtKind::AsyncFunctionDef {
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
            ast::StmtKind::ClassDef {
                name: class_name,
                bases,
                body,
                keywords: _,
                decorator_list: _,
            } => {
                let mut methods = vec![];
                for stmt in body {
                    match &stmt.node {
                        ast::StmtKind::FunctionDef {
                            name,
                            args,
                            body,
                            decorator_list,
                            returns,
                            type_comment,
                        }
                        | ast::StmtKind::AsyncFunctionDef {
                            name,
                            args,
                            body,
                            decorator_list,
                            returns,
                            type_comment,
                        } => {
                            let mut calls = vec![];
                            for stmt in body {
                                calls.append(&mut find_calls_in_stmt(&stmt.node));
                            }
                            methods.push((name.to_string(), calls));
                        }
                        _ => {}
                    }
                }
                parsed_module.classes.push(PyClass {
                    name: class_name.to_string(),
                    methods,
                });
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The project to analyze
    #[arg(short, long)]
    project: PathBuf,

    /// The module to analyze
    #[arg(short, long)]
    module: String,

    /// Optional function to analyze
    #[arg(short, long)]
    function: Option<String>,

    #[arg(long)]
    debug: bool,
}

fn main() -> io::Result<()> {
    let args: Args = Args::parse();
    let base_path = args.project;
    let module_name = args.module;
    let function_name = args.function;
    let debug = args.debug;

    let modules_to_paths = build_module_to_paths(&base_path)?;

    let mut modules = HashMap::new();

    for (module_name, path) in modules_to_paths.iter() {
        let mut source_code = String::new();
        File::open(path)?.read_to_string(&mut source_code)?;
        let module = parse_module(&module_name, &source_code, path);
        modules.insert(module_name.to_string(), module);
    }
    if debug {
        println!("{:#?}", modules);
    }

    print_transitive_deps(&modules, &module_name, function_name.as_deref())?;

    Ok(())
}
