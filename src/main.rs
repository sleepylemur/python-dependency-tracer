use rustpython_parser::{ast, parser::parse_program};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::{env, fs};

fn parse_imports(source_code: &str, path: &Path) -> Vec<String> {
    let mut imported_modules = vec![];
    let ast = parse_program(source_code, path.to_str().unwrap()).unwrap();
    for located in ast {
        let node = located.node;
        match &node {
            ast::StmtKind::Import { names } => {
                for import_name in names {
                    let module_name = import_name.node.name.to_string();
                    imported_modules.push(module_name);
                }
            }
            ast::StmtKind::ImportFrom {
                level,
                module,
                names: _,
            } => {
                let module_name = match &module {
                    Some(module) => module.to_string(),
                    None => ".".repeat(level.unwrap_or(0) as usize),
                };
                imported_modules.push(module_name);
            }
            _ => {}
        }
    }
    imported_modules
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

fn main() -> io::Result<()> {
    let args: Vec<_> = env::args().collect();
    let base_path = Path::new(&args[1]);

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
    println!("modules_to_paths {:?}", modules_to_paths);

    let mut module_imports = HashMap::new();

    for (module, path) in modules_to_paths.iter() {
        println!("Parsing {} from {:?}", module, path);
        let mut source_code = String::new();
        File::open(path)?.read_to_string(&mut source_code)?;
        let imports = parse_imports(&source_code, path);
        module_imports.insert(module.to_string(), imports);
    }

    println!("module_imports {:?}", module_imports);

    Ok(())
}
