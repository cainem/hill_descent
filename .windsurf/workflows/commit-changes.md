---
description: The standard process of checks to use on commit
---

run `cargo check` and make sure the code compiles. Fix it if it does not, repeating the `cargo check` to ensure any fixes
run `cargo test --quiet` to make sure all of the tests pass. Fix broken tests and lint errors, repeating the `cargo test --quiet` to ensure any fixes
make sure all edited structs and functions have accurate comments that are up to date an relevant.
run `cargo clippy` and make sure there are no lint errors. Fix lint errors found, repeating the `cargo clippy` to ensure any fixes
run `cargo clippy --tests` and make sure there are no lint errors. Fix lint errors found, repeating the `cargo clippy` to ensure any fixes
run `cargo fmt` to format the code correctlys
use git to stage all unstaged files
use git to commit the changes with an appropriate summary of the work carried out try and restrict the commit message to 200 characters if possible