use rustpython_parser::{ast, parser::parse_program};
use std::env;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn read_files(paths: &[&Path]) -> io::Result<String> {
    let mut contents = String::new();

    for path in paths {
        let mut file = File::open(path)?;
        file.read_to_string(&mut contents)?;
    }

    Ok(contents)
}
fn parse(source_code: &str) {
    let ast = parse_program(source_code, "fake.py").unwrap();
    for statement in ast {
        let node = statement.node;
        match &node {
            ast::StmtKind::Import { names } => {
                for import_name in names {
                    let module_name = &import_name.node;
                    let alias = match &module_name.asname {
                        Some(alias) => format!(" as {}", alias),
                        None => "".to_string(),
                    };
                    println!("Import: {}{}", module_name.name, alias);
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
                for import_name in names {
                    let import_item = &import_name.node;
                    let alias = match &import_item.asname {
                        Some(alias) => format!(" as {}", alias),
                        None => "".to_string(),
                    };
                    println!("Import: {} from {}{}", import_item.name, module_name, alias);
                }
            }
            ast::StmtKind::FunctionDef { name, .. } => {
                println!("FunctionDef: {}", name);
            }
            ast::StmtKind::ClassDef { name, .. } => {
                println!("ClassDef: {}", name);
            }
            _ => {
                // Skipping other statements
            }
        }
    }
}

fn visit_dirs(dir: &Path, cb: &dyn Fn(&Path)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else if let Some(extension) = path.extension() {
                if extension == "py" {
                    cb(&path);
                }
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <dir_path>", args[0]);
        return;
    }

    let dir_path = Path::new(&args[1]);

    if let Err(e) = visit_dirs(dir_path, &|path: &Path| {
        println!("Parsing file: {:?}", path);
        match read_files(&[path]) {
            Ok(contents) => {
                parse(&contents);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }) {
        eprintln!("Error: {}", e);
    }
}

// #[derive(Debug)]
// struct LookupItem {
//     name: String,
//     line_number: usize,
// }
