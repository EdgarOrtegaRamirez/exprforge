# Security Policy

## Overview

ExprForge is a mathematical expression evaluator designed with security as a foundational principle. It does **not** use `eval()`, `exec()`, or any mechanism that could execute arbitrary code. All expressions are parsed by a hand-written lexer and evaluated by a hand-written tree-walking evaluator.

## Security Properties

### No Code Injection
- The evaluator cannot execute arbitrary code
- All input is processed through a deterministic lexer → parser → evaluator pipeline
- No string interpolation, no shell execution, no file system access during evaluation

### Input Validation
- All inputs are validated during lexing (invalid characters rejected)
- All inputs are validated during parsing (syntax errors detected with position information)
- All inputs are validated during evaluation (type errors, undefined variables, incompatible units)

### Safe Arithmetic
- Division by zero returns an error (not `inf` or `NaN`)
- Modulo by zero returns an error
- Incompatible unit operations are detected and rejected (e.g., `5 m + 3 kg`)

### No External Dependencies at Runtime
- The library has zero runtime dependencies (only `clap` for the CLI binary)
- No network access, no file system access during evaluation
- No `unsafe` code

## Reporting a Vulnerability

If you discover a security vulnerability, please open an issue with the `security` label. All security issues will be addressed promptly.

## Scope

This security policy covers the ExprForge library and CLI tool. It does not cover third-party dependencies (e.g., `clap`), which have their own security policies.
