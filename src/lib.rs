//! Address translation and access checks for the homework's six-bit MMU.
//!
//! Addresses are values in a model, never pointers into the host's memory.
//! Each of eight page-table entries encodes `--PPPUWP`: three physical-page
//! bits and the User, Writable, and Present flags. The high two bits are ignored.
//!
//! ```
//! use home_work_3::{MemoryError, PrivilegeLevel, ToyMMU};
//!
//! let mmu = ToyMMU::new([23, 43, 13, 0, 0, 0, 0, 0], PrivilegeLevel::Ring3);
//! assert_eq!(mmu.translate(3, false), Ok(19));
//! assert_eq!(mmu.translate(16, true), Err(MemoryError::WriteProtected));
//! ```

const MAX_ADDRESS: u8 = 0b0011_1111;
const PAGE_BITS: u32 = 3;
const OFFSET_MASK: u8 = 0b0000_0111;
const PHYSICAL_PAGE_MASK: u8 = 0b0011_1000;
const PRESENT: u8 = 0b0000_0001;
const WRITABLE: u8 = 0b0000_0010;
const USER: u8 = 0b0000_0100;

/// A rejected translation; no physical memory access has occurred.
#[derive(Debug, PartialEq, Eq)]
pub enum MemoryError {
    /// The selected entry does not have the Present flag.
    PageNotPresent,
    /// A write was requested without the Writable flag, including in Ring0.
    WriteProtected,
    /// Ring3 attempted to access an entry without the User flag.
    PrivilegeViolation,
    /// The virtual address exceeds the six-bit range `0..=63`.
    InvalidAddress,
}

/// The simulated access mode, not the host process's actual CPU privilege.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeLevel {
    /// Supervisor access: bypasses User, but not Present or Writable checks.
    Ring0,
    /// User access: requires the User flag for both reads and writes.
    Ring3,
}

/// An immutable, single-level page table and its simulated privilege context.
///
/// Translation performs no allocation, I/O, or host memory access through the
/// returned address. Multiple virtual pages may map to the same physical page.
#[derive(Debug)]
pub struct ToyMMU {
    page_table: [u8; 8],
    current_privilege: PrivilegeLevel,
}

impl ToyMMU {
    /// Owns a fixed table of eight raw entries and the simulated access mode.
    ///
    /// Every entry byte is accepted: bits 6 and 7 are unused by this model.
    pub fn new(page_table: [u8; 8], privilege: PrivilegeLevel) -> Self {
        Self {
            page_table,
            current_privilege: privilege,
        }
    }

    /// Translates `va` into a physical address in `0..=63` without changing state.
    ///
    /// `is_write` selects a write (`true`) or read (`false`) permission check.
    /// The low three address bits are preserved as the page offset.
    ///
    /// # Errors
    ///
    /// Checks run in this order: [`MemoryError::InvalidAddress`],
    /// [`MemoryError::PageNotPresent`], [`MemoryError::PrivilegeViolation`],
    /// then [`MemoryError::WriteProtected`]. Ring0 bypasses only the User check.
    /// No address is silently truncated into the valid range.
    pub fn translate(&self, va: u8, is_write: bool) -> Result<u8, MemoryError> {
        if va > MAX_ADDRESS {
            return Err(MemoryError::InvalidAddress);
        }

        // Validation above proves that the page index fits the eight-entry table.
        let vpn = usize::from(va >> PAGE_BITS);
        let offset = va & OFFSET_MASK;
        let entry = self.page_table[vpn];

        if entry & PRESENT == 0 {
            return Err(MemoryError::PageNotPresent);
        }
        if self.current_privilege == PrivilegeLevel::Ring3 && entry & USER == 0 {
            return Err(MemoryError::PrivilegeViolation);
        }
        if is_write && entry & WRITABLE == 0 {
            return Err(MemoryError::WriteProtected);
        }

        let physical_page = (entry & PHYSICAL_PAGE_MASK) >> PAGE_BITS;
        Ok((physical_page << PAGE_BITS) | offset)
    }
}
