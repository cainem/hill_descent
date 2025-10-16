# Hill Descent - GitHub Copilot Instructions

> **Read `AGENTS.md` for comprehensive project guidance. Check `hill_descent_lib/pdd.md` for domain definitions.**

## Project Architecture
Rust genetic algorithm optimization system with workspace structure:
- `hill_descent_lib/` - Core n-dimensional optimization engine (genetic algorithm)
- `hill_descent_server/` - Web server with Actix for visualization and API endpoints
- Shared dependencies via workspace `Cargo.toml` (xxhash-rust, rand, serde)
- developed on a windows platform

## Essential Initialization Pattern
```rust
use hill_descent_lib::{GlobalConstants, setup_world, world::single_valued_function::SingleValuedFunction};

let param_range = vec![-100.0..=100.0, -100.0..=100.0];  // Problem bounds
let constants = GlobalConstants::new(1000, 100);          // Pop size, regions  
let world = setup_world(&param_range, constants, Box::new(MyFunction));
```

## Critical Code Organization
- **File naming**: `src/module/struct_name.rs` (filename = struct name)
- **Fields**: Private only - use getters/setters, never public fields
- **Function size**: Split at >40 lines into separate files
- **Tests**: `given_xxx_when_yyy_then_zzz` naming (see `global_constants.rs` tests)
- **Domain hierarchy**: `World -> Organisms -> DNA -> Phenotypes`

## Development Workflow
```powershell
cargo fmt && cargo test && cargo clippy   # Pre-commit checks
cargo run                                 # Start web server (from hill_descent_server/)
cargo bench                               # Run benchmarks
cargo test --workspace                    # Run all tests
```

## Key Domain Types
- `SingleValuedFunction` trait - fitness functions (see server's `BukinN6`, `Himmelblau`)
- `WorldFunction` trait - wrapper for batch evaluation  
- `RangeInclusive<f64>` - parameter bounds in problem space
- `GlobalConstants` - system config (pop size, regions, seed)
- `StdRng` - seeded RNG for reproducible runs

## Integration Points
- **WASM**: Build with `wasm-pack build --target web --out-dir web/pkg`
- **Web Server**: Actix endpoints at `/api/init`, `/api/step` for JS integration
- **Function Registration**: Server maps `FunctionType` enum to concrete implementations
- **Logging**: Optional tracing feature (`enable-tracing`) with structured logging

## Testing Requirements  
- Full branch/condition coverage per function (not transitive)
- Test boundary conditions, this is especially important if floating point numbers are used with the function
- Minimal mocking (only PRNG via `StdRng::from_seed`)
- Use `test_utils` module patterns for shared setup
- Integration tests in `tests/` directory for complete workflows

## Key Behaviors
- **Always read `AGENTS.md`** for comprehensive context at conversation start
- **Ask before changing** - clarify requirements vs assumptions
- **Ask before changing** - clarify any apparent contradictions in and requests and/or the combination of request and the AGENTS.md file
- **Check `pdd.md`** when domain logic changes to ensure consistency
- **Use existing patterns** - examine similar functions before creating new approaches