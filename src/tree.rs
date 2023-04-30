use std::{
    collections::{HashMap, HashSet},
    io,
};

use ptree::TreeBuilder;

use crate::PyModule;

pub fn print_transitive_deps(
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
pub fn add_function_dependencies_to_tree(
    tree_builder: &mut TreeBuilder,
    modules: &HashMap<String, PyModule>,
    module_name: &str,
    function_name: &str,
) {
    if let Some(PyModule {
        imports,
        import_froms,
        functions,
        ..
    }) = modules.get(module_name)
    {
        // build reverse map of imported names to imported modules
        let mut rev_imports = HashMap::new();
        for (module, names) in import_froms {
            for name in names {
                rev_imports.insert(name, module);
            }
        }

        let mut module_imports = HashSet::new();
        for module in imports {
            module_imports.insert(module);
        }

        if let Some((_, calls)) = functions
            .iter()
            .find(|(function, _)| function == function_name)
        {
            // Add the dependencies of the function to the tree
            for call in calls {
                // Check if the call references an import
                if let Some(module) = rev_imports.get(call) {
                    // Add the dependency to the tree
                    let child_builder = tree_builder.begin_child(format!("{}::{}", module, call));

                    // Recursively add the dependencies of the dependency to the tree
                    add_function_dependencies_to_tree(child_builder, modules, module, call);

                    tree_builder.end_child();
                } else {
                    // Check if the call references a module
                    // Split the call into parts
                    let split: Vec<_> = call.split(".").collect();
                    if split.len() > 1 {
                        let function = split.last().unwrap();
                        let module = split[..split.len() - 1].join(".");
                        if module_imports.contains(&module) {
                            // Add the dependency to the tree
                            let child_builder =
                                tree_builder.begin_child(format!("{}::{}", module, function));

                            // Recursively add the dependencies of the dependency to the tree
                            add_function_dependencies_to_tree(
                                child_builder,
                                modules,
                                &module,
                                function,
                            );

                            tree_builder.end_child();
                        }
                    }
                }
            }
        }
    }
}

pub fn add_module_dependencies_to_tree(
    tree_builder: &mut TreeBuilder,
    modules: &HashMap<String, PyModule>,
    module_name: &str,
) {
    if let Some(PyModule {
        imports,
        import_froms,
        ..
    }) = modules.get(module_name)
    {
        for module in import_froms.iter().map(|(m, _)| m).chain(imports.iter()) {
            // Add the dependency to the tree
            let child_builder = tree_builder.begin_child(module.to_string());

            // Recursively add the dependencies of the dependency to the tree
            add_module_dependencies_to_tree(child_builder, modules, module);

            tree_builder.end_child();
        }
    }
}
