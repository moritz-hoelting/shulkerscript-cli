# Shulkerscript cli tool

This is a cli tool for the shulkerscript language. It can be used to initialize a new project, and to compile and package a project.

## Installation
```bash	
cargo install --git https://github.com/moritz-hoelting/shulkerscript-cli.git
```

## Usage

### Initialize a new project
```bash
shulkerscript init [OPTIONS] [PATH]
```
Where [PATH] is the path of the folder to initialize in [default: `.`]

Options:
- `--name <NAME>`                The name of the project
- `--description <DESCRIPTION>`  The description of the project
- `--pack-format <PACK_FORMAT>`  The pack format version
- `--force`                      Force initialization even if the directory is not empty

### Build a project
```bash
shulkerscript build [OPTIONS] [PATH]
```
Where [PATH] is the path of the project folder to build [default: `.`]

Options:
- `--output <OUTPUT>`  The output directory, overrides the `DATAPACK_DIR` environment variable

Environment variables:
- `DATAPACK_DIR`       The output directory [default: `./dist`]

### Clean the output directory
```bash
shulkerscript clean [OPTIONS] [PATH]
```
Where [PATH] is the path of the project folder to clean [default: `.`]

Options:
- `--output <OUTPUT>`  The output directory, overrides the `DATAPACK_DIR` environment variable

Environment variables:
- `DATAPACK_DIR`       The output directory [default: `./dist`]

### Package a project
```bash
shulkerscript package [OPTIONS] [PATH]
```
Where [PATH] is the path of the project folder to package [default: `.`]

Options:
- `--output <OUTPUT>`  The output directory, overrides the `DATAPACK_DIR` environment variable

Environment variables:
- `DATAPACK_DIR`       The output directory [default: `./dist`]

### Watch for changes
```bash
shulkerscript watch [OPTIONS] [SUBCOMMAND]
```
Where [SUBCOMMAND] is either `build` or `package` [default: `build`]

Options:
- `--no-initial`                     Do not run the command initially
- `--debounce-time <DEBOUNCE_TIME>`  The time to wait in ms after the last change before running the command [default: `2000`]

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.

Please make sure to update tests as appropriate.

**Note that this repository only contains the cli tool for interfacing with the language. The language itself is located in the [shulkerscript-lang](https://github.com/moritz-hoelting/shulkerscript-lang) repository. Please indicate if pull requests for this repository require pull requests for the language repository**