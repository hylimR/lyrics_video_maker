---
description: Fix compilation warnings
---

1. Run `cargo check --workspace` or `cargo check -p <package>` to identify warnings.
2. Review the output for `warning:` lines. Common warnings include unused imports, unused variables, and dead code.
3. Fix the warnings by editing the code.
   - For unused imports: Remove the import.
   - For unused variables: Prefix with `_` or remove if strictly not needed.
   - For dead code: Remove it or prefix with `#[allow(dead_code)]` if it's intentional.
4. Verify the fixes by running the check command again until no warnings remain.