//! x86_64 PIC (8259A) Implementation
//! 
//! This module provides support for the legacy 8259A Programmable Interrupt Controller
//! for hardware interrupt handling.

use crate::log::{info, warn, error};
use crate::arch::interrupts::InterruptResult;

/// PIC I/O port addresses
const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

/// PIC initialization control words
const ICW1_ICW4: u8 = 0x01;        // ICW4 needed
const ICW1_SINGLE: u8 = 0x02;      // Single mode
const ICW1_INTERVAL4: u8 = 0x04;   // Call address interval 8
const ICW1_LEVEL: u8 = 0x08;       // Level triggered mode
const ICW1_INIT: u8 = 0x10;        // Initialization command

const ICW4_8086: u8 = 0x01;        // 8086/88 mode
const ICW4_AUTO_EOI: u8 = 0x02;    // Auto EOI
const ICW4_BUF_SLAVE: u8 = 0x08;   // Buffered mode/slave
const ICW4_BUF_MASTER: u8 = 0x0C;  // Buffered mode/master
const ICW4_SFNM: u8 = 0x10;        // Special fully nested mode

/// PIC operational command words
const OCW2_EOI: u8 = 0x20;         // End of interrupt
const OCW2_SPECIFIC_EOI: u8 = 0x60; // Specific EOI
const OCW2_ROTATE_EOI: u8 = 0x80;  // Rotate EOI
const OCW2_PRIORITY: u8 = 0xE0;    // Priority command

const OCW3_READ_IRR: u8 = 0x0A;    // Read Interrupt Request Register
const OCW3_READ_ISR: u8 = 0x0B;    // Read In-Service Register

/// Master PIC IRQ lines
pub const IRQ_TIMER: u8 = 0;
pub const IRQ_KEYBOARD: u8 = 1;
pub const IRQ_COM1: u8 = 4;
pub const IRQ_COM2: u8 = 3;
pub const IRQ_LPT1: u8 = 7;
pub const IRQ_CMOS: u8 = 8;
pub const IRQ_FREE: u8 = 12;
pub const IRQ_PS2_MOUSE: u8 = 12;
pub const IRQ_FPU: u8 = 13;
pub const IRQ_PRIMARY_ATA: u8 = 14;
pub const IRQ_SECONDARY_ATA: u8 = 15;

/// PIC state
static PIC_STATE: spin::Mutex<PicState> = spin::Mutex::new(PicState::default());

#[derive(Debug, Clone, Copy, Default)]
struct PicState {
    master_base: u8,   // Base vector for master PIC
    slave_base: u8,    // Base vector for slave PIC
    initialized: bool,
    interrupt_counts: [u32; 16],
}

/// Initialize the PIC
pub fn init_pic() -> InterruptResult<()> {
    info!("Initializing 8259A PIC...");
    
    let mut state = PIC_STATE.lock();
    
    if state.initialized {
        warn!("PIC already initialized");
        return Ok(());
    }
    
    // Save interrupt masks
    let master_mask = inb(PIC1_DATA);
    let slave_mask = inb(PIC2_DATA);
    
    // Initialize both PICs
    init_pic_master()?;
    init_pic_slave()?;
    
    // Restore interrupt masks
    outb(PIC1_DATA, master_mask);
    outb(PIC2_DATA, slave_mask);
    
    // Configure initial state
    state.master_base = 0x20; // IRQ 0-7 maps to vectors 0x20-0x27
    state.slave_base = 0x28; // IRQ 8-15 maps to vectors 0x28-0x2F
    state.initialized = true;
    
    info!("8259A PIC initialized successfully");
    info!("Master PIC base: {:#x}, Slave PIC base: {:#x}", state.master_base, state.slave_base);
    
    Ok(())
}

/// Initialize master PIC
fn init_pic_master() -> InterruptResult<()> {
    // Start initialization sequence
    outb(PIC1_COMMAND, ICW1_INIT | ICW1_ICW4);
    
    // ICW2: Vector offset for master
    outb(PIC1_DATA, 0x20);
    
    // ICW3: Master has slave on IRQ2
    outb(PIC1_DATA, 0x04);
    
    // ICW4: 8086 mode
    outb(PIC1_DATA, ICW4_8086);
    
    Ok(())
}

/// Initialize slave PIC
fn init_pic_slave() -> InterruptResult<()> {
    // Start initialization sequence
    outb(PIC2_COMMAND, ICW1_INIT | ICW1_ICW4);
    
    // ICW2: Vector offset for slave
    outb(PIC2_DATA, 0x28);
    
    // ICW3: Slave ID is IRQ2
    outb(PIC2_DATA, 0x02);
    
    // ICW4: 8086 mode
    outb(PIC2_DATA, ICW4_8086);
    
    Ok(())
}

/// Send End of Interrupt (EOI) command
pub fn send_eoi(irq: u8) {
    let mut state = PIC_STATE.lock();
    state.interrupt_counts[irq as usize] += 1;
    
    if irq >= 8 {
        // Slave interrupt - send EOI to both PICs
        outb(PIC2_COMMAND, OCW2_EOI);
    }
    
    // Always send EOI to master
    outb(PIC1_COMMAND, OCW2_EOI);
}

/// Enable specific IRQ line
pub fn enable_irq(irq: u8) {
    let port = if irq >= 8 { PIC2_DATA } else { PIC1_DATA };
    let mask = inb(port);
    
    outb(port, mask & !(1 << (irq % 8)));
    
    debug!("Enabled IRQ {}", irq);
}

/// Disable specific IRQ line
pub fn disable_irq(irq: u8) {
    let port = if irq >= 8 { PIC2_DATA } else { PIC1_DATA };
    let mask = inb(port);
    
    outb(port, mask | (1 << (irq % 8)));
    
    debug!("Disabled IRQ {}", irq);
}

/// Get current interrupt mask
pub fn get_irq_mask() -> u16 {
    let master_mask = inb(PIC1_DATA) as u16;
    let slave_mask = inb(PIC2_DATA) as u16;
    
    (master_mask | (slave_mask << 8)) & 0xFFFE // Ignore IRQ2
}

/// Set interrupt mask
pub fn set_irq_mask(mask: u16) {
    outb(PIC1_DATA, (mask & 0xFF) as u8);
    outb(PIC2_DATA, ((mask >> 8) & 0xFF) as u8);
}

/// Set specific IRQ mask bits
pub fn set_irq_mask_bits(bits: u16) {
    let mut mask = get_irq_mask();
    mask |= bits;
    set_irq_mask(mask);
}

/// Clear specific IRQ mask bits
pub fn clear_irq_mask_bits(bits: u16) {
    let mut mask = get_irq_mask();
    mask &= !bits;
    set_irq_mask(mask);
}

/// Read Interrupt Request Register (IRR)
pub fn read_irr() -> u16 {
    let master_irr = read_pic_register(PIC1_COMMAND, OCW3_READ_IRR);
    let slave_irr = read_pic_register(PIC2_COMMAND, OCW3_READ_IRR);
    
    (master_irr as u16) | ((slave_irr as u16) << 8)
}

/// Read In-Service Register (ISR)
pub fn read_isr() -> u16 {
    let master_isr = read_pic_register(PIC1_COMMAND, OCW3_READ_ISR);
    let slave_isr = read_pic_register(PIC2_COMMAND, OCW3_READ_ISR);
    
    (master_isr as u16) | ((slave_isr as u16) << 8)
}

/// Read PIC register
fn read_pic_register(port: u16, command: u8) -> u8 {
    outb(port, command);
    inb(port)
}

/// Read byte from I/O port
fn inb(port: u16) -> u8 {
    unsafe {
        let result: u8;
        core::arch::asm!(
            "inb {}, {}",
            out(reg) result,
            in(reg) port as u32,
            options(nostack)
        );
        result
    }
}

/// Write byte to I/O port
fn outb(port: u16, value: u8) {
    unsafe {
        core::arch::asm!(
            "outb {}, {}",
            in(reg) value as u32,
            in(reg) port as u32,
            options(nostack)
        );
    }
}

/// Get PIC interrupt statistics
pub fn get_pic_stats() -> PicStats {
    let state = PIC_STATE.lock();
    
    PicStats {
        master_base: state.master_base,
        slave_base: state.slave_base,
        initialized: state.initialized,
        interrupt_counts: state.interrupt_counts,
        current_mask: get_irq_mask(),
    }
}

/// PIC statistics structure
#[derive(Debug, Clone)]
pub struct PicStats {
    pub master_base: u8,
    pub slave_base: u8,
    pub initialized: bool,
    pub interrupt_counts: [u32; 16],
    pub current_mask: u16,
}

/// Set up standard interrupt handlers for PIC
pub fn setup_pic_handlers() -> InterruptResult<()> {
    info!("Setting up PIC interrupt handlers...");
    
    // Enable necessary IRQ lines
    enable_irq(IRQ_TIMER);       // Enable timer interrupt
    enable_irq(IRQ_KEYBOARD);    // Enable keyboard interrupt
    enable_irq(IRQ_CMOS);        // Enable CMOS clock
    enable_irq(IRQ_PS2_MOUSE);   // Enable PS/2 mouse if present
    
    info!("PIC interrupt handlers configured");
    Ok(())
}

/// Configure specific device IRQ
pub fn configure_device_irq(irq: u8, edge_triggered: bool, level_triggered: bool) -> InterruptResult<()> {
    if edge_triggered && level_triggered {
        return Err(crate::arch::interrupts::InterruptError::InvalidHandler);
    }
    
    if edge_triggered {
        info!("Configuring IRQ {} as edge-triggered", irq);
    } else if level_triggered {
        warn!("Level-triggered mode not supported on legacy PIC");
    }
    
    Ok(())
}

/// Handle PIC-specific interrupt acknowledgment
pub fn pic_interrupt_acknowledge(irq: u8) -> InterruptResult<usize> {
    // Map IRQ to interrupt vector number
    let vector = if irq < 8 {
        0x20 + irq
    } else {
        0x28 + (irq - 8)
    };
    
    debug!("PIC interrupt acknowledged: IRQ {} -> Vector {:#x}", irq, vector);
    Ok(vector)
}

/// Get IRQ line from interrupt vector
pub fn get_irq_from_vector(vector: usize) -> Option<u8> {
    if vector >= 0x20 && vector < 0x30 {
        Some(vector as u8 - 0x20)
    } else {
        None
    }
}