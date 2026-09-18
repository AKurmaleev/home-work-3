//! Exhaustive contract checks and boundary cases for the simulated MMU.

use home_work_3::{MemoryError, PrivilegeLevel, ToyMMU};

const PRIVILEGES: [PrivilegeLevel; 2] = [PrivilegeLevel::Ring0, PrivilegeLevel::Ring3];
const OPERATIONS: [bool; 2] = [false, true];

// Arithmetic extraction and a permission truth table deliberately avoid the
// implementation's masks/shifts. Priority: absent > privilege > write protection.
fn oracle(pte: u8, va: u8, privilege: PrivilegeLevel, is_write: bool) -> Result<u8, MemoryError> {
    if va >= 64 {
        return Err(MemoryError::InvalidAddress);
    }

    match (pte % 8, privilege, is_write) {
        (0 | 2 | 4 | 6, _, _) => Err(MemoryError::PageNotPresent),
        (1 | 3, PrivilegeLevel::Ring3, _) => Err(MemoryError::PrivilegeViolation),
        (1 | 5, _, true) => Err(MemoryError::WriteProtected),
        _ => Ok(((pte / 8) % 8) * 8 + va % 8),
    }
}

fn varied_table(selected_page: u8, pte: u8) -> [u8; 8] {
    let mut table = [0; 8];
    for page in 0u8..8 {
        let distance = (page + 8 - selected_page) % 8;
        // Every other slot has a different PPN and flags; a uniform table would
        // hide wrong-index bugs. All intermediate values fit in u8.
        let ppn = (pte / 8 + distance) % 8;
        let flags = (pte % 8 + distance) % 8;
        table[usize::from(page)] = (pte / 64) * 64 + ppn * 8 + flags;
    }
    table
}

#[test]
fn every_valid_address_pte_privilege_and_operation_matches_oracle() {
    let mut cases = 0usize;
    for va in 0u8..64 {
        for pte in 0u8..=u8::MAX {
            let table = varied_table(va / 8, pte);
            for privilege in PRIVILEGES {
                let mmu = ToyMMU::new(table, privilege);
                for is_write in OPERATIONS {
                    assert_eq!(
                        mmu.translate(va, is_write),
                        oracle(pte, va, privilege, is_write),
                        "va={va}, pte={pte:#010b}, privilege={privilege:?}, write={is_write}"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 65_536);
}

#[test]
fn every_invalid_address_is_rejected_before_indexing_or_permissions() {
    let mut cases = 0usize;
    for pte in 0u8..=u8::MAX {
        for privilege in PRIVILEGES {
            // Uniform entries expose any erroneous address truncation for every
            // possible PTE, including conflicting permission failures.
            let mmu = ToyMMU::new([pte; 8], privilege);
            for va in 64u8..=u8::MAX {
                for is_write in OPERATIONS {
                    assert_eq!(
                        mmu.translate(va, is_write),
                        Err(MemoryError::InvalidAddress),
                        "va={va}, pte={pte:#010b}, privilege={privilege:?}, write={is_write}"
                    );
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 196_608);
}

#[test]
fn address_and_page_boundaries_preserve_offsets() {
    let table = [63, 55, 47, 39, 31, 23, 15, 7];
    let cases = [
        (0, 56),
        (7, 63),
        (8, 48),
        (15, 55),
        (55, 15),
        (56, 0),
        (62, 6),
        (63, 7),
    ];
    for privilege in PRIVILEGES {
        let mmu = ToyMMU::new(table, privilege);
        for (va, pa) in cases {
            for is_write in OPERATIONS {
                assert_eq!(
                    mmu.translate(va, is_write),
                    Ok(pa),
                    "va={va}, privilege={privilege:?}, write={is_write}"
                );
            }
        }
    }
}

#[test]
fn conflicting_errors_follow_priority_without_kernel_presence_or_write_bypass() {
    use MemoryError::{PageNotPresent, PrivilegeViolation, WriteProtected};
    use PrivilegeLevel::{Ring0, Ring3};

    let cases = [
        (0, Ring0, false, Err(PageNotPresent)),
        (0, Ring0, true, Err(PageNotPresent)),
        (0, Ring3, false, Err(PageNotPresent)),
        (0, Ring3, true, Err(PageNotPresent)),
        (1, Ring3, false, Err(PrivilegeViolation)),
        (1, Ring3, true, Err(PrivilegeViolation)),
        (1, Ring0, false, Ok(0)),
        (1, Ring0, true, Err(WriteProtected)),
        (5, Ring0, true, Err(WriteProtected)),
        (5, Ring3, true, Err(WriteProtected)),
        (3, Ring0, true, Ok(0)),
        (3, Ring3, true, Err(PrivilegeViolation)),
    ];
    for (pte, privilege, is_write, expected) in cases {
        let mmu = ToyMMU::new([pte; 8], privilege);
        assert_eq!(
            mmu.translate(0, is_write),
            expected,
            "pte={pte:#010b}, privilege={privilege:?}, write={is_write}"
        );
    }
}

#[test]
fn ignored_high_bits_change_neither_physical_address_nor_permissions() {
    for flags in 0u8..8 {
        let base_pte = 48 + flags;
        for high_bits in [0u8, 64, 128, 192] {
            for privilege in PRIVILEGES {
                let mut table = [0; 8];
                table[5] = base_pte + high_bits;
                let mmu = ToyMMU::new(table, privilege);
                for is_write in OPERATIONS {
                    assert_eq!(
                        mmu.translate(45, is_write),
                        oracle(base_pte, 45, privilege, is_write),
                        "flags={flags}, high_bits={high_bits}, privilege={privilege:?}, write={is_write}"
                    );
                }
            }
        }
    }
}

#[test]
fn distinct_page_mappings_select_the_virtual_page_not_the_offset() {
    let physical_pages = [3u8, 0, 7, 2, 5, 1, 6, 4];
    let table = physical_pages.map(|ppn| ppn * 8 + 7);
    for privilege in PRIVILEGES {
        let mmu = ToyMMU::new(table, privilege);
        for va in 0u8..64 {
            let pa = physical_pages[usize::from(va / 8)] * 8 + va % 8;
            for is_write in OPERATIONS {
                assert_eq!(
                    mmu.translate(va, is_write),
                    Ok(pa),
                    "va={va}, privilege={privilege:?}, write={is_write}"
                );
            }
        }
    }
}

#[test]
fn aliased_physical_pages_keep_independent_virtual_page_permissions() {
    for privilege in PRIVILEGES {
        let mmu = ToyMMU::new([31, 29, 0, 0, 0, 0, 0, 0], privilege);
        for offset in 0u8..8 {
            let pa = 24 + offset;
            assert_eq!(mmu.translate(offset, false), Ok(pa));
            assert_eq!(mmu.translate(offset, true), Ok(pa));
            assert_eq!(mmu.translate(8 + offset, false), Ok(pa));
            assert_eq!(
                mmu.translate(8 + offset, true),
                Err(MemoryError::WriteProtected)
            );
        }
    }
}

#[test]
fn interleaved_mmu_contexts_keep_mappings_and_privileges_isolated() {
    let first = ToyMMU::new([15; 8], PrivilegeLevel::Ring3);
    let second = ToyMMU::new([55; 8], PrivilegeLevel::Ring3);
    let restricted = ToyMMU::new([51; 8], PrivilegeLevel::Ring3);
    let kernel = ToyMMU::new([51; 8], PrivilegeLevel::Ring0);
    for offset in 0u8..8 {
        for is_write in OPERATIONS {
            let va = 32 + offset;
            assert_eq!(first.translate(va, is_write), Ok(8 + offset));
            assert_eq!(second.translate(va, is_write), Ok(48 + offset));
            assert_eq!(
                restricted.translate(va, is_write),
                Err(MemoryError::PrivilegeViolation)
            );
            assert_eq!(kernel.translate(va, is_write), Ok(48 + offset));
            assert_eq!(first.translate(va, is_write), Ok(8 + offset));
        }
    }
}
