# Report: 6-bit MMU

## Implementation

As specified in `HW_Page_Tables.pdf`, `ToyMMU::translate(&self, va: u8, is_write: bool) -> Result<u8, MemoryError>` is implemented. The API and six checks from the [instructor's template](https://gist.github.com/kulinsky/39cee1674cbdf54432734bfe78e9ce0d) are preserved. The code is in [`src/lib.rs`](src/lib.rs); run the examples with `cargo run`.

The MMU instance owns the `[u8; 8]` table and access mode. The method does not modify state, allocate memory, or dereference host addresses. The returned `u8` is an address in the educational model, not a Rust pointer.

## Algorithm

1. Check `va <= 63` before accessing the table. Otherwise, return `InvalidAddress`.
2. Compute `VPN = va >> 3`, `offset = va & 0b111`.
3. Read `entry = page_table[VPN]`. After address validation, the index is guaranteed to be in `0..8`.
4. If `P=0`, return `PageNotPresent`.
5. If the mode is Ring3 and `U=0`, return `PrivilegeViolation`.
6. If a write is requested and `W=0`, return `WriteProtected`.
7. Extract `PPN = (entry & 0b0011_1000) >> 3`, excluding flags and unused bits.
8. Return `PA = (PPN << 3) | offset`.

The result has at most six significant bits: `PPN <= 7`, `offset <= 7`. Shifts are fixed, with no overflow. For the fixed table, translation time and auxiliary space are `Θ(1)`; the table's payload occupies 8 bytes, with no claim about the full layout of the Rust struct.

### Permissions and error precedence

| Condition | Read | Write |
|---|---|---|
| `P=0`, any mode | `PageNotPresent` | `PageNotPresent` |
| `P=1`, Ring3, `U=0` | `PrivilegeViolation` | `PrivilegeViolation` |
| `P=1`, permitted mode, `W=0` | Allowed | `WriteProtected` |
| `P=1`, permitted mode, `W=1` | Allowed | Allowed |

The assignment does not specify precedence for simultaneous `U` and `W` violations. The implementation defines it explicitly: `InvalidAddress → PageNotPresent → PrivilegeViolation → WriteProtected`. Ring0 does not bypass page absence or write protection. In its specific example, the original test name `test_kernel_can_access_everything` means access to a supervisor page, not removal of all checks.

PTE bits 6–7 are unused according to the assignment and are ignored when extracting the PPN. This does not permit ignoring reserved bits in arbitrary real architectures.

## Step-by-step example

In the template's table, `page_table[0] = 23 = 0b0001_0111`:

```text
VA = 3 = 0b000_011
VPN = 0, offset = 3
PTE = 23: PPN = 2, U = 1, W = 1, P = 1
PA = (2 << 3) | 3 = 19 = 0b010_011
```

The offset is unchanged. `VA=11` uses `page_table[1]=43` instead: Ring3 gets `PrivilegeViolation`, while Ring0 gets `PA=43`. Reading `VA=16` is allowed; writing is prohibited. For `VA=24`, the page is absent; `VA=64` is invalid even before the PTE lookup.

## Connections to lectures 5–6

- **Privileges and isolation.** The mode and `User` are checked separately from address conversion itself. Different MMU instances can translate the same VA to different PAs; tests verify that there is no shared hidden state.
- **Page tables.** An entry contains both a page number and permissions. Simply replacing the high address bits is not enough: access must be checked first.
- **Paging.** Replacing the VPN with the PPN preserves the offset. Multiple virtual pages can refer to the same physical page while retaining different permissions in their PTEs.
- **Faults as access results.** An error is neither replaced with zero nor turned into `panic!`; the caller receives a specific `MemoryError`.

The model deliberately does not implement a scheduler, a CPU transition to Ring0, a multilevel page walk, a TLB, swap, copy-on-write, demand paging, or a hardware page fault handler. In a real OS, a page fault can be handled successfully rather than necessarily terminating the process. Specific page sizes, numbers of levels, and TLB latencies depend on the platform. `malloc` is an allocator function, not a separate system call; calling it does not necessarily enter the kernel.

These clarifications were checked against the Linux documentation on [page tables](https://docs.kernel.org/mm/page_tables.html), [five-level x86 paging](https://docs.kernel.org/arch/x86/x86_64/5level-paging.html), and [`malloc(3)`](https://man7.org/linux/man-pages/man3/malloc.3.html).

## Verification

On native Windows with Rust 1.95.0, the following passed:

- `cargo fmt --all --check`;
- `cargo check --frozen --all-targets`;
- `cargo clippy --frozen --all-targets -- -D warnings`;
- `cargo test --frozen`;
- `cargo test --frozen --release`;
- `cargo run --frozen --quiet` — demonstration of all four errors and successful translations.

Cargo was run with `CARGO_INCREMENTAL=0` because of an access error affecting the local disk's incremental cache. This is a build setting, not an algorithm change.

In each test profile, 6 instructor tests, 8 additional tests, and 1 doctest passed:

| Check | Coverage |
|---|---|
| Exhaustive enumeration of valid inputs | 64 VAs × 256 values of the selected PTE × 2 modes × 2 operations = 65 536 cases |
| Exhaustive enumeration of invalid addresses | 192 VAs × 256 PTEs × 2 modes × 2 operations = 196 608 cases |
| Boundaries | Addresses 0 and 63, page transitions, offset preservation |
| Permissions | All flags, conflicting errors, no P/W bypass in Ring0 |
| Unused bits | All four combinations of the high PTE bits |
| Mappings | Different PPNs, a shared physical page with different permissions, independent MMUs |

The reference calculation in [`tests/exhaustive.rs`](tests/exhaustive.rs) uses division, remainder, and a permission table instead of the implementation's masks and shifts. Distinct adjacent table entries help detect incorrect index selection. Enumeration exhausts the values of the selected PTE, but not all possible eight-byte tables.
