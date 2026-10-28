# Homework 3 — Page Tables

An MMU emulator in Rust for an educational 6-bit architecture. Implements virtual-to-physical address translation with page presence and access permission checks based on lectures 5–6.

## Running and verification

Requires stable Rust with edition 2024 support (MSRV: 1.85). No external dependencies.

```sh
cargo test
cargo run
```

`cargo run` compares successful translations and errors in Ring3 and Ring0; all four error types are demonstrated in Ring3, while Ring0 bypasses the `User` check. This is an ordinary user-space program: no administrator privileges, Linux, changes to the host's page tables, or special CPU are required.

Full verification:

```sh
cargo fmt --all --check
cargo check --frozen --all-targets
cargo clippy --frozen --all-targets -- -D warnings -D unused_must_use -D clippy::let_underscore_must_use -D clippy::unused_result_ok
cargo test --frozen
cargo test --frozen --release
```

## Reviewer navigation

| File | Contents |
|---|---|
| [`src/lib.rs`](src/lib.rs) | `ToyMMU::translate`, error and privilege level types, API example |
| [`src/main.rs`](src/main.rs) | Demonstration using the table from the instructor's template |
| [`tests/instructor.rs`](tests/instructor.rs) | 6 tests from the original template |
| [`tests/exhaustive.rs`](tests/exhaustive.rs) | 8 additional tests: exhaustive enumeration, boundaries, permissions, independent contexts |
| [`REPORT.md`](REPORT.md) | Algorithm, translation example, connections to the lectures, and verification results |
| [`Cargo.toml`](Cargo.toml) | Rust version, package settings, and prohibition of `unsafe` |

## Model

- Addresses: `0..=63`; page: 8 bytes; table: 8 entries.
- Virtual address: `VPN[5:3] | OFFSET[2:0]`.
- PTE entry: `--PPPUWP`, where `PPP` is the physical page number, `U` is User, `W` is Writable, and `P` is Present.
- PTE bits 6–7 are ignored; a virtual address exceeding 6 bits is rejected rather than truncated.
- Ring3 requires `U=1`. Ring0 bypasses only this check; `P` and `W` apply in both modes.

Example from the template: `VA=3`, `PTE=23` → physical page 2, offset 3 → `PA=19`.

The API names and signatures from the [instructor's template](https://gist.github.com/kulinsky/39cee1674cbdf54432734bfe78e9ce0d) are preserved. The logic is in a library, with tests in separate files.

## Results

**14 integration tests and 1 doctest** pass in both debug and release. Exhaustive enumeration covers 65 536 valid combinations and 196 608 combinations with invalid addresses. Translation time and auxiliary space are `Θ(1)` for this model. No benchmarks or hardware optimizations were added: the assignment verifies the correctness of translation and access protection.
