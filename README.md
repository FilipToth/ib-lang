# IB Lang

While some tools exist for executing and debugging the IB "Pseudocode" standard, they often fail to provide a robust interface with clear error handling, which is exactly what students need when learning the basics of programming through the IB curriculum. This project provides a Rust crate containing ib-lang's code analysis features and a limited execution VM. This analyzer then gets exposed through an API as a bare-bones language server. The server then interfaces with the custom IDE shipped as part of this project to provide features such as code completion, error highlighting, and more.

### Core Features

-   Tokenizer / lexer pipeline
-   Parser that synthesizes an Abstract Syntax Tree
-   Binder that performs type-checking
-   Control Flow Analyzer that ensures all non-void user-defined functions return a value
-   Minimal execution runtime (this is a WIP, and the current implementation is experimental)
-   Concurrent RESTful API server that exposes the analyzer as a bare-bones Language Server Protocol-like API surface (with plans to migrate to full LSP later)
-   Web-based IDE written in React using the Codemirror text editor
-   Context and scope-aware code completion in the IDE leveraging the language server for obtaining information about symbols in the current scope to provide smarter code completions
-   Multiple tabs
-   Code execution on the backend through the IB Lang Core VM
-   User account management for Web IDE through Firebase

### Adoption of Microsoft's Language Server Protocol

Migration to Microsoft's Language Server Protocol is planned, although due to time constraints, a more flexible solution was chosen. This allowed us to ship code completion features sooner without having to implement the very heavy LSP standard yet. Migration to the LSP standard will allow IB Lang to be used in a multitude of text editors and IDEs, such as a VSCode extension. Initially, LSP was planned to be implemented, but we failed to get the communication between Microsoft's Monaco React Client (the editor backend of VSCode) to communicate with any LSP backend, thus due to time constraints, we've chosen to build our own API surface.
