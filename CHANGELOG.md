# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Subcommand `init` to initialize a new project
    - Creates a new project directory with default files
- Subcommand `build` to build a project
    - Compiles the project to a datapack directory
    - Allows changing the output directory and setting a assets directory
- Subcommand `clean` to clean the output directory
- Subcommand `watch` to watch for changes and run a command
- Subcommand `lang-debug` to debug the language parser
    - Allows to print the parsed tokens, AST and shulkerbox datapack representation

### Changed

### Removed
