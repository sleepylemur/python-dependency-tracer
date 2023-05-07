# Python dependency tracer

Long term this is intended to output dependencies and transitive dependencies for files, class, and functions in a python project.
Currently it only builds a map of modules to imports

## Install

cargo install --git https://github.com/sleepylemur/python-dependency-tracer

## Run

### module dependencies

pydep -p example_project -m modulename

### function dependencies

pydep -p example_project -m modulename -f functionname

## Run/install from local repo

cargo run --release -- -p example_project -m modulename

cargo install --path .gi
