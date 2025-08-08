---
description: The standard process of checks to use on commit
---

1) make sure all edited structs and functions have accurate comments that are up to date and relevant.
2) run `cargo fmt` to format the code correctly.
3) run `cargo test --quiet` to make sure all of the tests pass. Fix broken tests and lint errors, repeating the `cargo test --quiet` to ensure any fixes
4) run `cargo check` and make sure the code compiles without warnings. Fix it if it does not, repeating the `cargo check` to ensure any fixes
repeat this step.
5) Repeat steps 4 but with the feature flag enable-tracing.
7) run `cargo clippy` and make sure there are no lint errors. Fix lint errors found, repeating the `cargo clippy` to ensure any fixes
8) run `cargo clippy --tests` and make sure there are no lint errors. Fix lint errors found, repeating the `cargo clippy` to ensure any fixes
9) use git to stage all unstaged files
10) use git to commit the changes with an appropriate summary of the work carried out try and restrict the commit message to 200 characters if possibles