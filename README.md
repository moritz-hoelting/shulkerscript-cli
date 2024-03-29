# Shulkerscript cli tool

This is a cli tool for the shulkerscript language. It can be used to initialize a new project, and to compile and package a project.

## Installation
```bash	
cargo install --git https://github.com/moritz-hoelting/shulkerscript
```

## Usage

### Initialize a new project
```bash
shulkerscript init [OPTIONS] [PATH]
```
Where [PATH] is the path of the folder to initialize in [default: .]

Options:
- `--name <NAME>`                The name of the project
- `--description <DESCRIPTION>`  The description of the project
- `--pack-format <PACK_FORMAT>`  The pack format version
- `--force`                      Force initialization even if the directory is not empty
