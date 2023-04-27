# Python dependency tracer
Long term this is intended to output dependencies and transitive dependencies for files, class, and functions in a python project.
Currently it only builds a map of modules to imports

## Run
cargo run --bin trace3 "$(pwd)/example_project"
