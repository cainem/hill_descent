# Hill Descent - GitHub Copilot Instructions

> **See `AGENTS.md` for comprehensive project guidance**

## Quick Context
Rust genetic algorithm optimization system with two components:
- `hill_descent_lib/` - Core n-dimensional hill descent optimization 
- `hill_descent_server/` - Web visualization with WASM integration

## Critical Patterns
- **File structure**: `src/module/struct_name.rs` (filename = struct name)
- **Fields**: Private only, use getters/setters
- **Tests**: `given_xxx_when_yyy_then_zzz` naming pattern
- **Functions**: Split at >40 lines
- **Domain**: `World -> Organisms -> DNA -> Phenotypes`

## Key Commands
```powershell
cargo fmt && cargo test && cargo clippy  # Pre-commit essentials
cargo run                                # Start web server (hill_descent_server/)
```

## Domain Types
- `SingleValuedFunction` trait for fitness functions
- `RangeInclusive<f64>` for parameter bounds  
- `GlobalConstants` for system configuration
- `StdRng` with consistent seeding patterns

## Key behaviour
At the start of a new conversation always read AGENTS.md to get the full context.
When presented with a question always prompt before assuming to change code.
Always try to resolve any ambiguity in requests by asking questions rather than making assumptions.