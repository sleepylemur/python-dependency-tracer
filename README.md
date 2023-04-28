# Python dependency tracer

Long term this is intended to output dependencies and transitive dependencies for files, class, and functions in a python project.
Currently it only builds a map of modules to imports

## Install

cargo install --git https://github.com/sleepylemur/python-dependency-tracer

## Run

pydep example_project packagename

## Run/install from local repo

cargo run example_project packagename

cargo install --path .
