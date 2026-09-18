//! Original tests: https://gist.github.com/kulinsky/39cee1674cbdf54432734bfe78e9ce0d
use home_work_3::{MemoryError, PrivilegeLevel, ToyMMU};

// Индекс 0: Физическая стр 2, Флаги: U=1, W=1, P=1 (0b010_111 = 23) -> Доступно всем для всего
// Индекс 1: Физическая стр 5, Флаги: U=0, W=1, P=1 (0b101_011 = 43) -> Только для Ядра (Ring0)
// Индекс 2: Физическая стр 1, Флаги: U=1, W=0, P=1 (0b001_101 = 13) -> Только для чтения
// Индекс 3: Физическая стр 0, Флаги: U=0, W=0, P=0 (0b000_000 = 0)  -> Страница отсутствует
fn setup_test_table() -> [u8; 8] {
    [23, 43, 13, 0, 0, 0, 0, 0]
}

#[test]
fn test_successful_translation() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
    // VA = 3 (0b000_011): Стр 0, Смещение 3.
    // Физическая стр в таблице = 2 (0b010). Физический адрес должен быть: 0b010_011 = 19
    assert_eq!(mmu.translate(3, false), Ok(19));
}

#[test]
fn test_invalid_address() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
    assert_eq!(mmu.translate(64, false), Err(MemoryError::InvalidAddress));
    assert_eq!(mmu.translate(100, false), Err(MemoryError::InvalidAddress));
}

#[test]
fn test_page_not_present() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
    // VA = 24 (0b011_000): Стр 3, Смещение 0. У стр 3 флаг P = 0
    assert_eq!(mmu.translate(24, false), Err(MemoryError::PageNotPresent));
}

#[test]
fn test_write_protection() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
    // VA = 16 (0b010_000): Стр 2, Смещение 0. У стр 2 флаг W = 0 (Только чтение)
    assert!(mmu.translate(16, false).is_ok()); // Читать можно
    assert_eq!(mmu.translate(16, true), Err(MemoryError::WriteProtected)); // Писать нельзя
}

#[test]
fn test_privilege_violation_in_ring3() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring3);
    // VA = 8 (0b001_000): Стр 1, Смещение 0. У стр 1 флаг U = 0 (Ядро)
    assert_eq!(
        mmu.translate(8, false),
        Err(MemoryError::PrivilegeViolation)
    );
}

#[test]
fn test_kernel_can_access_everything() {
    let mmu = ToyMMU::new(setup_test_table(), PrivilegeLevel::Ring0);
    // В Ring0 ядро должно успешно читать страницу, защищенную флагом U=0
    // VA = 11 (0b001_011): Стр 1, Смещение 3.
    // Физическая стр в таблице = 5 (0b101). Физический адрес: 0b101_011 = 43
    assert_eq!(mmu.translate(11, false), Ok(43));
}
