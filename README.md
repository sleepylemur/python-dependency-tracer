# Python dependency tracer

Long term this is intended to output dependencies and transitive dependencies for files, class, and functions in a python project.
Currently it only builds a map of modules to imports

## Install

cargo install --git https://github.com/sleepylemur/python-dependency-tracer

## Run

### module dependencies

pydep example_project modulename

### function dependencies

pydep example_project modulename functionname

## Run/install from local repo

cargo run example_project modulename

cargo install --path .
