//! Demonstrates translations and faults using the instructor's example table.

use home_work_3::{PrivilegeLevel, ToyMMU};

fn main() {
    let table = [23, 43, 13, 0, 0, 0, 0, 0];
    for privilege in [PrivilegeLevel::Ring3, PrivilegeLevel::Ring0] {
        let mmu = ToyMMU::new(table, privilege);
        for (va, is_write) in [
            (3, false),
            (11, false),
            (16, true),
            (24, false),
            (64, false),
        ] {
            let operation = if is_write { "write" } else { "read" };
            println!(
                "{privilege:?} {operation:5} VA={va:2} -> {:?}",
                mmu.translate(va, is_write)
            );
        }
    }
}
