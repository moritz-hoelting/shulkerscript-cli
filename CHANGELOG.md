# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

### Changed

### Removed

## [0.1.0] - 2024-10-01

### Added

- Subcommand `init` to initialize a new project
    - Creates a new project directory with default files
- Subcommand `build` to build a project
    - Compiles the project to a datapack directory
    - Allows changing the output directory and setting a assets directory
- Subcommand `clean` to clean the output directory
- Subcommand `watch` to watch for changes and run a command
- Subcommand `migrate` to migrate a datapack to a shulkerscript project
- Subcommand `lang-debug` to debug the language parser
    - Allows to print the parsed tokens, AST and shulkerbox datapack representation

[unreleased]: https://github.com/moritz-hoelting/shulkerscript-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/moritz-hoelting/shulkerscript-cli/releases/tag/v0.1.0