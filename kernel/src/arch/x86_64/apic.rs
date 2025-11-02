//! x86_64 APIC (Advanced PIC) Implementation
//! 
//! This module provides support for the Advanced Programmable Interrupt Controller
//! for modern x86_64 systems, including Local APIC and I/O APIC.

use crate::log::{info, warn, error};
use crate::arch::interrupts::InterruptResult;

/// APIC base address (typically at 0xFEE00000)
const APIC_BASE: u64 = 0xFEE00000;

/// Local APIC register offsets
const APIC_ID_REG: u32 = 0x20;           // APIC ID Register
const APIC_VERSION_REG: u32 = 0x30;      // APIC Version Register
const APIC_TPR_REG: u32 = 0x80;          // Task Priority Register
const APIC_APR_REG: u32 = 0x90;          // Arbitration Priority Register
const APIC_PPR_REG: u32 = 0xA0;          // Processor Priority Register
const APIC_EOI_REG: u32 = 0xB0;          // End of Interrupt Register
const APIC_LDR_REG: u32 = 0xD0;          // Logical Destination Register
const APIC_DFR_REG: u32 = 0xE0;          // Destination Format Register
const APIC_SPIV_REG: u32 = 0xF0;         // Spurious Interrupt Vector Register
const APIC_ESR_REG: u32 = 0x280;         // Error Status Register
const APIC_LVT_CMCI_REG: u32 = 0x2F0;    // LVT CMCI Register
const APIC_LVT_TIMER_REG: u32 = 0x320;   // LVT Timer Register
const APIC_LVT_THERMAL_REG: u32 = 0x330; // LVT Thermal Sensor Register
const APIC_LVT_PERF_REG: u32 = 0x340;    // LVT Performance Monitor Register
const APIC_LVT_LINT0_REG: u32 = 0x350;   // LVT LINT0 Register
const APIC_LVT_LINT1_REG: u32 = 0x360;   // LVT LINT1 Register
const APIC_LVT_ERROR_REG: u32 = 0x370;   // LVT Error Register
const APIC_ICR_LOW_REG: u32 = 0x300;     // Interrupt Command Register Low
const APIC_ICR_HIGH_REG: u32 = 0x310;    // Interrupt Command Register High
const APIC_TIMER_DIV_REG: u32 = 0x3E0;   // Timer Divide Configuration Register
const APIC_TIMER_INIT_REG: u32 = 0x380;  // Timer Initial Count Register
const APIC_TIMER_CURR_REG: u32 = 0x390;  // Timer Current Count Register

/// APIC timer modes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ApicTimerMode {
    OneShot = 0,
    Periodic = 1,
    TscDeadline = 2,
}

/// APIC delivery modes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ApicDeliveryMode {
    Fixed = 0,
    LowestPriority = 1,
    Smi = 2,
    Nmi = 4,
    Init = 5,
    Startup = 6,
    ExtInt = 7,
}

/// APIC destination modes
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ApicDestinationMode {
    Physical = 0,
    Logical = 1,
}

/// APIC state
static APIC_STATE: spin::Mutex<ApicState> = spin::Mutex::new(ApicState::default());

#[derive(Debug, Clone, Default)]
struct ApicState {
    enabled: bool,
    apic_base: u64,
    apic_id: u8,
    version: u8,
    max_lvt_entries: u8,
    timer_divisor: u32,
    timer_initial_count: u32,
    timer_mode: ApicTimerMode,
}

/// Initialize Local APIC
pub fn init_apic() -> InterruptResult<()> {
    info!("Initializing Local APIC...");
    
    let mut state = APIC_STATE.lock();
    
    if state.enabled {
        warn!("Local APIC already initialized");
        return Ok(());
    }
    
    // Get APIC base address from MSR
    let apic_base = get_apic_base_msr();
    state.apic_base = apic_base;
    
    // Check if APIC is enabled
    if !is_apic_enabled() {
        error!("APIC is not enabled in MSR");
        return Err(crate::arch::interrupts::InterruptError::IdtInitializationFailed);
    }
    
    // Read APIC ID
    let apic_id = read_apic_register(APIC_ID_REG) >> 24;
    state.apic_id = apic_id as u8;
    
    // Read APIC version
    let version = read_apic_register(APIC_VERSION_REG) & 0xFF;
    state.version = version as u8;
    
    // Calculate max LVT entries from version register
    state.max_lvt_entries = ((version >> 16) & 0xFF) + 1;
    
    // Initialize Local APIC
    init_local_apic()?;
    
    // Configure timer
    configure_timer()?;
    
    state.enabled = true;
    
    info!("Local APIC initialized successfully");
    info!("APIC Base: {:#x}, APIC ID: {}, Version: {}", apic_base, apic_id, version);
    
    Ok(())
}

/// Get APIC base address from MSR
fn get_apic_base_msr() -> u64 {
    let mut apic_base: u64;
    unsafe {
        core::arch::asm!(
            "mov $0x1B, %ecx",      // MSR_APIC_BASE
            "rdmsr",
            "mov %rax, {}",
            out(reg) apic_base
        );
    }
    
    // Mask off reserved bits to get base address
    apic_base & 0xFFFFF000
}

/// Check if APIC is enabled
fn is_apic_enabled() -> bool {
    let mut apic_base: u64;
    unsafe {
        core::arch::asm!(
            "mov $0x1B, %ecx",      // MSR_APIC_BASE
            "rdmsr",
            "mov %rax, {}",
            out(reg) apic_base
        );
    }
    
    (apic_base & (1 << 11)) != 0
}

/// Initialize Local APIC
fn init_local_apic() -> InterruptResult<()> {
    // Configure spurious interrupt vector register
    // Enable APIC and set spurious vector to 0xFF
    write_apic_register(APIC_SPIV_REG, 0x1FF);
    
    // Set task priority to allow all interrupts
    write_apic_register(APIC_TPR_REG, 0);
    
    // Configure logical destination mode
    write_apic_register(APIC_LDR_REG, 0x01000000); // Logical APIC ID = 1
    
    // Configure destination format register (flat mode)
    write_apic_register(APIC_DFR_REG, 0xFFFFFFFF);
    
    Ok(())
}

/// Configure Local APIC timer
fn configure_timer() -> InterruptResult<()> {
    let mut state = APIC_STATE.lock();
    
    // Set timer divide configuration to divide by 16 (default)
    write_apic_register(APIC_TIMER_DIV_REG, 0b1011); // Divide by 16
    state.timer_divisor = 16;
    
    // Set initial timer count for periodic mode (e.g., 1ms intervals)
    let timer_tick_rate = 1000; // 1000 Hz = 1ms intervals
    let timer_count = 100000;   // Assume 100 MHz bus clock / 16 = ~6.25 MHz
    
    write_apic_register(APIC_TIMER_INIT_REG, timer_count);
    state.timer_initial_count = timer_count;
    
    // Configure timer LVT register for periodic mode
    let timer_lvt = (0 << 16) |        // Mask bit = 0 (enabled)
                   (0 << 15) |        // Trigger mode = 0 (edge)
                   (0 << 14) |        // Remote IRR = 0
                   (0 << 12) |        // Delivery status = 0
                   (0x20) |           // Vector = 0x20 (IRQ 0)
                   (ApicTimerMode::Periodic as u32);
    
    write_apic_register(APIC_LVT_TIMER_REG, timer_lvt);
    state.timer_mode = ApicTimerMode::Periodic;
    
    Ok(())
}

/// Read Local APIC register
fn read_apic_register(offset: u32) -> u32 {
    let addr = APIC_BASE + offset;
    
    unsafe {
        core::arch::asm!(
            "mov {}, %eax",
            in(reg) addr as usize,
            options(nostack, readonly)
        );
        let result: u32;
        core::arch::asm!(
            "mov %eax, {}",
            out(reg) result
        );
        result
    }
}

/// Write Local APIC register
fn write_apic_register(offset: u32, value: u32) {
    let addr = APIC_BASE + offset;
    
    unsafe {
        core::arch::asm!(
            "mov {}, %eax",
            in(reg) value
        );
        core::arch::asm!(
            "mov {}, %edx",      // High 32 bits are zero
            in(reg) 0u32
        );
        core::arch::asm!(
            "mov {}, %ebx",
            in(reg) addr as usize,
            options(nostack)
        );
        core::arch::asm!(
            "mov %ebx, %ecx",
            "wrmsr",
            options(nostack)
        );
    }
}

/// Send End of Interrupt
pub fn send_eoi() {
    write_apic_register(APIC_EOI_REG, 0);
}

/// Send interrupt to local processor
pub fn send_ipi(destination: u8, vector: u8, delivery_mode: ApicDeliveryMode) -> InterruptResult<()> {
    let mut icr_low = (vector as u32) |
                      ((delivery_mode as u32) << 8) |
                      ((ApicDestinationMode::Physical as u32) << 11) |
                      (1 << 14); // Level bit
    
    let mut icr_high = ((destination as u32) << 24);
    
    // Write ICR high then low (order matters)
    write_apic_register(APIC_ICR_HIGH_REG, icr_high);
    write_apic_register(APIC_ICR_LOW_REG, icr_low);
    
    Ok(())
}

/// Initialize specific processor
pub fn send_init_ipi(target_cpu: u8) -> InterruptResult<()> {
    send_ipi(target_cpu, 0, ApicDeliveryMode::Init)?;
    
    // Wait for delivery
    for _ in 0..10 {
        let icr_low = read_apic_register(APIC_ICR_LOW_REG);
        if (icr_low & (1 << 12)) == 0 {
            break; // Delivery complete
        }
        // Small delay
        for _ in 0..1000 {
            unsafe { core::arch::asm!("pause"); }
        }
    }
    
    Ok(())
}

/// Send startup IPI to target processor
pub fn send_startup_ipi(target_cpu: u8, start_addr: usize) -> InterruptResult<()> {
    // Startup IPI uses 4KB page addresses
    let page_addr = start_addr >> 12;
    let vector = (page_addr & 0xFF) as u8;
    
    send_ipi(target_cpu, vector, ApicDeliveryMode::Startup)?;
    
    Ok(())
}

/// Get timer current count
pub fn get_timer_count() -> u32 {
    read_apic_register(APIC_TIMER_CURR_REG)
}

/// Set timer count
pub fn set_timer_count(count: u32) {
    write_apic_register(APIC_TIMER_INIT_REG, count);
}

/// Configure timer for one-shot mode
pub fn configure_timer_one_shot() -> InterruptResult<()> {
    let mut state = APIC_STATE.lock();
    state.timer_mode = ApicTimerMode::OneShot;
    
    let timer_lvt = read_apic_register(APIC_LVT_TIMER_REG);
    let new_timer_lvt = (timer_lvt & !((1 << 17) | 0x1F)) | (ApicTimerMode::OneShot as u32);
    write_apic_register(APIC_LVT_TIMER_REG, new_timer_lvt);
    
    Ok(())
}

/// Configure timer for periodic mode
pub fn configure_timer_periodic() -> InterruptResult<()> {
    let mut state = APIC_STATE.lock();
    state.timer_mode = ApicTimerMode::Periodic;
    
    let timer_lvt = read_apic_register(APIC_LVT_TIMER_REG);
    let new_timer_lvt = (timer_lvt & !((1 << 17) | 0x1F)) | (ApicTimerMode::Periodic as u32);
    write_apic_register(APIC_LVT_TIMER_REG, new_timer_lvt);
    
    Ok(())
}

/// Get APIC statistics
pub fn get_apic_stats() -> ApicStats {
    let state = APIC_STATE.lock();
    
    ApicStats {
        enabled: state.enabled,
        apic_base: state.apic_base,
        apic_id: state.apic_id,
        version: state.version,
        max_lvt_entries: state.max_lvt_entries,
        timer_divisor: state.timer_divisor,
        timer_initial_count: state.timer_initial_count,
        timer_mode: state.timer_mode,
        timer_current_count: if state.enabled { get_timer_count() } else { 0 },
    }
}

/// APIC statistics structure
#[derive(Debug, Clone)]
pub struct ApicStats {
    pub enabled: bool,
    pub apic_base: u64,
    pub apic_id: u8,
    pub version: u8,
    pub max_lvt_entries: u8,
    pub timer_divisor: u32,
    pub timer_initial_count: u32,
    pub timer_mode: ApicTimerMode,
    pub timer_current_count: u32,
}

/// Configure interrupt for local delivery
pub fn configure_local_interrupt(vector: u8, delivery_mode: ApicDeliveryMode, 
                                trigger_mode: bool, level: bool) -> InterruptResult<()> {
    let reg_offset = match vector {
        0 => APIC_LVT_CMCI_REG,
        1 => APIC_LVT_TIMER_REG,
        2 => APIC_LVT_THERMAL_REG,
        3 => APIC_LVT_PERF_REG,
        4 => APIC_LVT_LINT0_REG,
        5 => APIC_LVT_LINT1_REG,
        6 => APIC_LVT_ERROR_REG,
        _ => return Err(crate::arch::interrupts::InterruptError::InvalidInterruptNumber),
    };
    
    let mut lvt_value = (vector as u32) |
                       ((delivery_mode as u32) << 8) |
                       (if trigger_mode { 1 << 15 } else { 0 }) |
                       (if level { 1 << 14 } else { 0 });
    
    write_apic_register(reg_offset, lvt_value);
    
    Ok(())
}