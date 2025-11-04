//! SD Card Driver
//! 
//! Support for SD and SDHC/SDXC cards with multiple interfaces:
//! SPI mode and SD mode (4-bit and 8-bit).

use crate::log::{info, warn, error};
use super::block::{BlockDeviceId, BlockOperation, BlockDeviceError};

use spin::{Mutex, RwLock};
use alloc::{vec::Vec, collections::BTreeMap};
use core::time::Duration;

/// SD card types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdCardType {
    SdSc = 0,      // Standard Capacity SD (up to 2GB)
    SdHc = 1,      // High Capacity SD (4GB-32GB)
    SdXc = 2,      // Extended Capacity SD (64GB-2TB)
    SdUc = 3,      // Ultra Capacity SD (future)
}

/// SD card interface mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SdInterfaceMode {
    Spi = 0,       // SPI mode (slower, simpler)
    Sd1Bit = 1,    // SD mode 1-bit
    Sd4Bit = 2,    // SD mode 4-bit
    Sd8Bit = 3,    // SD mode 8-bit (MMC)
}

/// SD card command types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum SdCommand {
    GoIdleState = 0,
    SendOpCond = 1,
    AllSendCid = 2,
    SendRelativeAddr = 3,
    SelectCard = 7,
    SendCsd = 9,
    SendCid = 10,
    StopTransmission = 12,
    SendStatus = 13,
    SetBlockLen = 16,
    ReadSingleBlock = 17,
    ReadMultipleBlocks = 18,
    WriteBlock = 24,
    WriteMultipleBlocks = 25,
    ProgramCsd = 27,
    SetWriteProtect = 28,
    ClearWriteProtect = 29,
    SendWriteProtect = 30,
    EraseWriteBlockStart = 32,
    EraseWriteBlockEnd = 33,
    Erase = 38,
    IoSendCommand = 39,
    AppCommand = 55,
    ReadOcr = 58,
    CrcOnOff = 59,
}

/// SD card response types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum SdResponseType {
    None = 0,
    R1 = 1,        // 48-bit response
    R1b = 2,       // R1 with busy
    R2 = 3,        // 136-bit response
    R3 = 4,        // 48-bit response (OCR)
    R6 = 5,        // 48-bit response (RCA)
}

/// SD card register structure
#[derive(Debug, Clone)]
struct SdRegisters {
    pub cid: [u8; 16],      // Card Identification register
    pub csd: [u8; 16],      // Card Specific Data register
    pub scr: [u8; 8],       // SD Configuration register
    pub ocr: [u4; 8],       // Operation Conditions register
    pub rca: u16,           // Relative Card Address
    pub dsr: u16,           // Driver Stage register
}

/// SD card information
#[derive(Debug, Clone)]
pub struct SdCardInfo {
    pub card_type: SdCardType,
    pub interface_mode: SdInterfaceMode,
    pub total_size: u64,           // in bytes
    pub sector_size: u32,          // typically 512 bytes
    pub total_sectors: u64,
    pub max_transfer_size: u32,    // maximum bytes per transfer
    pub high_capacity: bool,       // true for SDHC/SDXC
    pub supports_write_protection: bool,
    pub supports_erase: bool,
    pub speed_class: u8,           // speed class (0, 2, 4, 6, 10)
    pub manufacturer_id: u8,
    pub oem_id: [u8; 2],
    pub product_name: [u8; 6],
    pub product_revision: u8,
    pub manufacture_date: [u8; 2],
    pub serial_number: u32,
    pub crc: u8,
}

/// SD card driver state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum SdCardState {
    Idle = 0,
    Ready = 1,
    Identification = 2,
    Standby = 3,
    Transfer = 4,
    SendingData = 5,
    ReceivingData = 6,
    Programming = 7,
    Disconnect = 8,
    Reserved = 9,
    Error = 10,
}

/// SD Card Driver
pub struct SdCardDriver {
    card_id: BlockDeviceId,
    spi_cs_pin: u8,
    spi_mosi_pin: u8,
    spi_miso_pin: u8,
    spi_clk_pin: u8,
    voltage_supply: f32,        // Supply voltage (1.8V or 3.3V)
    clock_frequency: u32,       // Current clock frequency in Hz
    max_clock_frequency: u32,   // Maximum supported frequency
    state: SdCardState,
    registers: SdRegisters,
    current_block_len: u32,
    card_info: Option<SdCardInfo>,
    initialized: bool,
    write_protected: bool,
    erase_enabled: bool,
    multiple_block_mode: bool,
}

/// SPI communication interface for SD cards
pub struct SpiInterface {
    pub cs_pin: u8,
    pub mosi_pin: u8,
    pub miso_pin: u8,
    pub clk_pin: u8,
    pub current_frequency: u32,
}

impl SpiInterface {
    /// Create new SPI interface
    pub fn new(cs_pin: u8, mosi_pin: u8, miso_pin: u8, clk_pin: u8) -> Self {
        Self {
            cs_pin,
            mosi_pin,
            miso_pin,
            clk_pin,
            current_frequency: 400_000, // Start with 400kHz
        }
    }

    /// Send SPI byte
    pub fn send_byte(&self, data: u8) -> Result<u8, BlockDeviceError> {
        // In real implementation, this would use hardware SPI
        info!("SPI Send: 0x{:02X}", data);
        Ok(0) // Return received byte
    }

    /// Receive SPI byte
    pub fn receive_byte(&self) -> Result<u8, BlockDeviceError> {
        // In real implementation, this would use hardware SPI
        info!("SPI Receive");
        Ok(0xFF) // Return received byte (0xFF for no data)
    }

    /// Send command to SD card
    pub fn send_command(&self, command: SdCommand, argument: u32, response_type: SdResponseType) -> Result<Vec<u8>, BlockDeviceError> {
        info!("Sending SD command {} with argument 0x{:08X}", command as u8, argument);
        
        // Build command packet
        let mut packet = [0u8; 6];
        packet[0] = 0x40 | (command as u8); // Command + start bit
        packet[1] = (argument >> 24) as u8;
        packet[2] = (argument >> 16) as u8;
        packet[3] = (argument >> 8) as u8;
        packet[4] = argument as u8;
        
        // Calculate CRC (simplified)
        let crc = self.calculate_crc(&packet[0..5]);
        packet[5] = crc;
        
        // Send command
        let _ = self.send_byte(packet[0]);
        let _ = self.send_byte(packet[1]);
        let _ = self.send_byte(packet[2]);
        let _ = self.send_byte(packet[3]);
        let _ = self.send_byte(packet[4]);
        let _ = self.send_byte(packet[5]);
        
        // Wait for response
        self.wait_for_response(response_type)
    }

    /// Wait for SD card response
    fn wait_for_response(&self, response_type: SdResponseType) -> Result<Vec<u8>, BlockDeviceError> {
        let mut response = Vec::new();
        
        match response_type {
            SdResponseType::None => {
                return Ok(response);
            }
            SdResponseType::R1 => {
                // Wait for non-0xFF response
                let mut timeout = 1000;
                let mut resp = 0xFF;
                while resp == 0xFF && timeout > 0 {
                    resp = self.receive_byte()?;
                    timeout -= 1;
                }
                response.push(resp);
            }
            SdResponseType::R1b => {
                // R1 with busy signal
                let resp = self.receive_byte()?;
                response.push(resp);
                
                // Wait for busy to clear
                let mut timeout = 10000;
                while self.receive_byte()? != 0xFF && timeout > 0 {
                    timeout -= 1;
                }
            }
            SdResponseType::R2 => {
                // 136-bit response (17 bytes)
                for _ in 0..17 {
                    let byte = self.receive_byte()?;
                    response.push(byte);
                }
            }
            SdResponseType::R3 => {
                // OCR response (5 bytes)
                let resp = self.receive_byte()?;
                response.push(resp);
                for _ in 0..4 {
                    let byte = self.receive_byte()?;
                    response.push(byte);
                }
            }
            SdResponseType::R6 => {
                // RCA response (6 bytes)
                let resp = self.receive_byte()?;
                response.push(resp);
                for _ in 0..5 {
                    let byte = self.receive_byte()?;
                    response.push(byte);
                }
            }
        }
        
        Ok(response)
    }

    /// Calculate CRC7 for command
    fn calculate_crc(&self, data: &[u8]) -> u8 {
        let mut crc = 0u8;
        for &byte in data {
            let mut val = byte;
            for _ in 0..8 {
                let tmp = crc ^ val;
                crc <<= 1;
                if tmp & 0x80 != 0 {
                    crc ^= 0x09;
                }
                val <<= 1;
            }
        }
        ((crc << 1) | 1) & 0x7F
    }
}

impl SdCardDriver {
    /// Create new SD card driver
    pub fn new(card_id: BlockDeviceId, cs_pin: u8, mosi_pin: u8, miso_pin: u8, clk_pin: u8) -> Self {
        info!("Creating SD card driver for device {:?} with SPI pins CS:{}, MOSI:{}, MISO:{}, CLK:{}", 
              card_id, cs_pin, mosi_pin, miso_pin, clk_pin);
        
        Self {
            card_id,
            spi_cs_pin: cs_pin,
            spi_mosi_pin: mosi_pin,
            spi_miso_pin: miso_pin,
            spi_clk_pin: clk_pin,
            voltage_supply: 3.3,
            clock_frequency: 400_000,
            max_clock_frequency: 50_000_000, // 50MHz maximum
            state: SdCardState::Idle,
            registers: SdRegisters {
                cid: [0; 16],
                csd: [0; 16],
                scr: [0; 8],
                ocr: [0; 8],
                rca: 0,
                dsr: 0,
            },
            current_block_len: 512,
            card_info: None,
            initialized: false,
            write_protected: false,
            erase_enabled: false,
            multiple_block_mode: false,
        }
    }

    /// Initialize SD card
    pub fn init(&mut self) -> Result<(), BlockDeviceError> {
        info!("Initializing SD card device {:?}", self.card_id);
        
        // Create SPI interface
        let spi = SpiInterface::new(self.spi_cs_pin, self.spi_mosi_pin, self.spi_miso_pin, self.spi_clk_pin);
        
        // Step 1: Send 80 clock pulses to wake up the card
        self.send_clock_pulses(&spi, 80)?;
        
        // Step 2: Send GO_IDLE_STATE command
        let response = spi.send_command(SdCommand::GoIdleState, 0, SdResponseType::R1)?;
        if response[0] != 0x01 {
            return Err(BlockDeviceError::HardwareError);
        }
        
        self.state = SdCardState::Ready;
        
        // Step 3: Check card voltage
        if !self.check_voltage_range(&spi)? {
            return Err(BlockDeviceError::UnsupportedOperation);
        }
        
        // Step 4: Initialize card
        if self.initialize_card(&spi)? {
            info!("SD card initialized successfully");
            self.initialized = true;
            Ok(())
        } else {
            error!("SD card initialization failed");
            self.state = SdCardState::Error;
            Err(BlockDeviceError::HardwareError)
        }
    }

    /// Send clock pulses (CS high)
    fn send_clock_pulses(&self, spi: &SpiInterface, count: usize) -> Result<(), BlockDeviceError> {
        info!("Sending {} clock pulses", count);
        
        for _ in 0..count {
            let _ = spi.send_byte(0xFF);
        }
        
        Ok(())
    }

    /// Check voltage range compatibility
    fn check_voltage_range(&self, spi: &SpiInterface) -> Result<bool, BlockDeviceError> {
        info!("Checking voltage range");
        
        // Send ACMD41 to check voltage range
        let response = spi.send_command(SdCommand::AppCommand, 0, SdResponseType::R1)?;
        if response[0] != 0x00 && response[0] != 0x01 {
            return Ok(false);
        }
        
        let response = spi.send_command(SdCommand::SendOpCond, 0x00100000, SdResponseType::R3)?;
        
        // Check OCR register bit 7 (Card power up status bit)
        if response.len() >= 4 {
            let ocr_byte3 = response[1];
            return Ok((ocr_byte3 & 0x80) != 0);
        }
        
        Ok(false)
    }

    /// Initialize SD card
    fn initialize_card(&mut self, spi: &SpiInterface) -> Result<bool, BlockDeviceError> {
        info!("Initializing SD card");
        
        // Get CID register
        let cid_response = spi.send_command(SdCommand::AllSendCid, 0, SdResponseType::R2)?;
        if cid_response.len() == 17 {
            self.registers.cid.copy_from_slice(&cid_response[1..]);
        }
        
        // Get RCA
        let rca_response = spi.send_command(SdCommand::SendRelativeAddr, 0, SdResponseType::R6)?;
        if rca_response.len() >= 6 {
            self.registers.rca = ((rca_response[1] as u16) << 8) | rca_response[2] as u16;
        }
        
        // Select card
        let _ = spi.send_command(SdCommand::SelectCard, (self.registers.rca as u32) << 16, SdResponseType::R1b)?;
        
        self.state = SdCardState::Standby;
        
        // Get CSD register
        let csd_response = spi.send_command(SdCommand::SendCsd, (self.registers.rca as u32) << 16, SdResponseType::R2)?;
        if csd_response.len() == 17 {
            self.registers.csd.copy_from_slice(&csd_response[1..]);
        }
        
        // Parse card information
        self.parse_card_information()?;
        
        // Set block length
        let _ = spi.send_command(SdCommand::SetBlockLen, 512, SdResponseType::R1)?;
        self.current_block_len = 512;
        
        // Get SCR register
        let scr_response = spi.send_command(SdCommand::AppCommand, (self.registers.rca as u32) << 16, SdResponseType::R1)?;
        if scr_response[0] == 0x00 {
            let _ = spi.send_command(SdCommand::SendStatus, 0, SdResponseType::R1)?;
            // In real implementation, would read SCR register
        }
        
        self.state = SdCardState::Transfer;
        
        Ok(true)
    }

    /// Parse card information from registers
    fn parse_card_information(&mut self) -> Result<(), BlockDeviceError> {
        let csd = &self.registers.csd;
        
        // Determine card type based on CSD structure
        let csd_structure = (csd[0] >> 6) & 0x03;
        self.card_info = match csd_structure {
            0 => Some(SdCardInfo {
                card_type: SdCardType::SdSc,
                interface_mode: SdInterfaceMode::Spi,
                total_size: self.calculate_total_size_sdsc(csd),
                sector_size: 512,
                total_sectors: self.calculate_total_size_sdsc(csd) / 512,
                max_transfer_size: 512,
                high_capacity: false,
                supports_write_protection: (csd[14] & 0x10) != 0,
                supports_erase: (csd[14] & 0x40) != 0,
                speed_class: 0,
                manufacturer_id: self.registers.cid[0],
                oem_id: [self.registers.cid[1], self.registers.cid[2]],
                product_name: self.registers.cid[3..9].try_into().unwrap_or([0; 6]),
                product_revision: self.registers.cid[9],
                manufacture_date: [self.registers.cid[10], self.registers.cid[11]],
                serial_number: ((self.registers.cid[12] as u32) << 16) | 
                              ((self.registers.cid[13] as u32) << 8) | 
                              (self.registers.cid[14] as u32),
                crc: self.registers.cid[15],
            }),
            1 => Some(SdCardInfo {
                card_type: SdCardType::SdHc,
                interface_mode: SdInterfaceMode::Spi,
                total_size: self.calculate_total_size_sdhs(csd),
                sector_size: 512,
                total_sectors: self.calculate_total_size_sdhs(csd) / 512,
                max_transfer_size: 4096,
                high_capacity: true,
                supports_write_protection: (csd[14] & 0x10) != 0,
                supports_erase: (csd[14] & 0x40) != 0,
                speed_class: self.calculate_speed_class(csd),
                manufacturer_id: self.registers.cid[0],
                oem_id: [self.registers.cid[1], self.registers.cid[2]],
                product_name: self.registers.cid[3..9].try_into().unwrap_or([0; 6]),
                product_revision: self.registers.cid[9],
                manufacture_date: [self.registers.cid[10], self.registers.cid[11]],
                serial_number: ((self.registers.cid[12] as u32) << 16) | 
                              ((self.registers.cid[13] as u32) << 8) | 
                              (self.registers.cid[14] as u32),
                crc: self.registers.cid[15],
            }),
            _ => Some(SdCardInfo {
                card_type: SdCardType::SdXc,
                interface_mode: SdInterfaceMode::Spi,
                total_size: self.calculate_total_size_sdxc(csd),
                sector_size: 512,
                total_sectors: self.calculate_total_size_sdxc(csd) / 512,
                max_transfer_size: 8192,
                high_capacity: true,
                supports_write_protection: (csd[14] & 0x10) != 0,
                supports_erase: (csd[14] & 0x40) != 0,
                speed_class: self.calculate_speed_class(csd),
                manufacturer_id: self.registers.cid[0],
                oem_id: [self.registers.cid[1], self.registers.cid[2]],
                product_name: self.registers.cid[3..9].try_into().unwrap_or([0; 6]),
                product_revision: self.registers.cid[9],
                manufacture_date: [self.registers.cid[10], self.registers.cid[11]],
                serial_number: ((self.registers.cid[12] as u32) << 16) | 
                              ((self.registers.cid[13] as u32) << 8) | 
                              (self.registers.cid[14] as u32),
                crc: self.registers.cid[15],
            }),
        };
        
        info!("SD card info parsed: {:?}", self.card_info.as_ref().unwrap());
        
        Ok(())
    }

    /// Calculate total size for SD Standard Capacity
    fn calculate_total_size_sdsc(&self, csd: &[u8; 16]) -> u64 {
        let c_size = ((csd[6] & 0x03) as u64) << 10 | 
                    ((csd[7] as u64) << 2) | 
                    ((csd[8] & 0xC0) >> 6);
        let c_size_mult = ((csd[9] & 0x03) << 1) | ((csd[10] & 0x80) >> 7);
        let read_bl_len = csd[5] & 0x0F;
        
        let mult = 1u64 << (c_size_mult + 2);
        let block_len = 1u64 << read_bl_len;
        
        (c_size + 1) * mult * block_len
    }

    /// Calculate total size for SD High Capacity
    fn calculate_total_size_sdhs(&self, csd: &[u8; 16]) -> u64 {
        let c_size = ((csd[7] as u64) & 0x3F) << 16 | 
                    ((csd[8] as u64) << 8) | 
                    (csd[9] as u64);
        
        (c_size + 1) * 512 * 1024 // 512KB blocks
    }

    /// Calculate total size for SD Extended Capacity
    fn calculate_total_size_sdxc(&self, csd: &[u8; 16]) -> u64 {
        // Similar to SDHC but with larger capacity calculation
        self.calculate_total_size_sdhs(csd)
    }

    /// Calculate speed class
    fn calculate_speed_class(&self, csd: &[u8; 16]) -> u8 {
        // Simplified speed class calculation
        // Real implementation would parse more complex fields
        if (csd[3] & 0x80) != 0 {
            10 // UHS-I Class 10
        } else if (csd[3] & 0x40) != 0 {
            6  // UHS-I Class 6
        } else if (csd[3] & 0x20) != 0 {
            4  // Class 4
        } else {
            2  // Class 2
        }
    }

    /// Read sectors from SD card
    pub fn read_sectors(&mut self, sector: u64, count: u32, buffer: &mut [u8]) -> Result<usize, BlockDeviceError> {
        if !self.initialized {
            return Err(BlockDeviceError::HardwareError);
        }
        
        let required_size = (count as usize) * (self.current_block_len as usize);
        if buffer.len() < required_size {
            return Err(BlockDeviceError::BufferTooSmall);
        }
        
        if let Some(card_info) = &self.card_info {
            if sector + count as u64 > card_info.total_sectors {
                return Err(BlockDeviceError::InvalidSector);
            }
        }
        
        info!("Reading {} sectors from SD card starting at sector {}", count, sector);
        
        let spi = SpiInterface::new(self.spi_cs_pin, self.spi_mosi_pin, self.spi_miso_pin, self.spi_clk_pin);
        
        // Calculate byte address (for SDHC/SDXC, sector is the byte address/512)
        let address = if self.card_info.as_ref().map_or(false, |info| info.high_capacity) {
            sector * 512
        } else {
            sector
        };
        
        // Send READ_SINGLE_BLOCK command
        let response = spi.send_command(SdCommand::ReadSingleBlock, address, SdResponseType::R1)?;
        if response[0] != 0x00 {
            return Err(BlockDeviceError::HardwareError);
        }
        
        // Wait for data start token
        let mut token = 0xFF;
        let mut timeout = 10000;
        while token == 0xFF && timeout > 0 {
            token = spi.receive_byte()?;
            timeout -= 1;
        }
        
        if token != 0xFE {
            return Err(BlockDeviceError::HardwareError);
        }
        
        // Read data
        let mut bytes_read = 0;
        for i in 0..count {
            // Read one sector (512 bytes)
            for j in 0..self.current_block_len {
                let byte = spi.receive_byte()?;
                if (i as usize * self.current_block_len as usize + j as usize) < buffer.len() {
                    buffer[i as usize * self.current_block_len as usize + j as usize] = byte;
                    bytes_read += 1;
                }
            }
            
            // Read CRC16 (2 bytes)
            let _ = spi.receive_byte()?;
            let _ = spi.receive_byte()?;
        }
        
        info!("Successfully read {} bytes from SD card", bytes_read);
        
        Ok(bytes_read)
    }

    /// Write sectors to SD card
    pub fn write_sectors(&mut self, sector: u64, count: u32, buffer: &[u8]) -> Result<usize, BlockDeviceError> {
        if !self.initialized {
            return Err(BlockDeviceError::HardwareError);
        }
        
        if self.write_protected {
            return Err(BlockDeviceError::PermissionDenied);
        }
        
        let required_size = (count as usize) * (self.current_block_len as usize);
        if buffer.len() < required_size {
            return Err(BlockDeviceError::BufferTooSmall);
        }
        
        if let Some(card_info) = &self.card_info {
            if sector + count as u64 > card_info.total_sectors {
                return Err(BlockDeviceError::InvalidSector);
            }
        }
        
        info!("Writing {} sectors to SD card starting at sector {}", count, sector);
        
        let spi = SpiInterface::new(self.spi_cs_pin, self.spi_mosi_pin, self.spi_miso_pin, self.spi_clk_pin);
        
        // Calculate byte address
        let address = if self.card_info.as_ref().map_or(false, |info| info.high_capacity) {
            sector * 512
        } else {
            sector
        };
        
        // Send WRITE_BLOCK command
        let response = spi.send_command(SdCommand::WriteBlock, address, SdResponseType::R1)?;
        if response[0] != 0x00 {
            return Err(BlockDeviceError::HardwareError);
        }
        
        // Send data start token
        let _ = spi.send_byte(0xFE);
        
        // Write data
        for i in 0..count {
            for j in 0..self.current_block_len {
                let byte_index = i as usize * self.current_block_len as usize + j as usize;
                if byte_index < buffer.len() {
                    let _ = spi.send_byte(buffer[byte_index]);
                } else {
                    let _ = spi.send_byte(0x00);
                }
            }
            
            // Send CRC16 (simplified - not calculating real CRC)
            let _ = spi.send_byte(0x00);
            let _ = spi.send_byte(0x00);
            
            // Wait for response
            let mut response_byte = 0xFF;
            let mut timeout = 1000;
            while response_byte == 0xFF && timeout > 0 {
                response_byte = spi.receive_byte()?;
                timeout -= 1;
            }
            
            if (response_byte & 0x1F) != 0x05 {
                return Err(BlockDeviceError::HardwareError);
            }
            
            // Wait for write completion
            let mut busy = 0xFF;
            timeout = 10000;
            while busy != 0xFF && timeout > 0 {
                busy = spi.receive_byte()?;
                timeout -= 1;
            }
        }
        
        let bytes_written = (count as usize) * (self.current_block_len as usize);
        
        info!("Successfully wrote {} bytes to SD card", bytes_written);
        
        Ok(bytes_written)
    }

    /// Erase sectors (TRIM operation for SD cards)
    pub fn erase_sectors(&mut self, sector: u64, count: u32) -> Result<(), BlockDeviceError> {
        if !self.initialized || !self.erase_enabled {
            return Err(BlockDeviceError::UnsupportedOperation);
        }
        
        info!("Erasing {} sectors from SD card starting at sector {}", count, sector);
        
        let spi = SpiInterface::new(self.spi_cs_pin, self.spi_mosi_pin, self.spi_miso_pin, self.spi_clk_pin);
        
        // Send ERASE_WR_BLOCK_START command
        let address = if self.card_info.as_ref().map_or(false, |info| info.high_capacity) {
            sector * 512
        } else {
            sector
        };
        let _ = spi.send_command(SdCommand::EraseWriteBlockStart, address, SdResponseType::R1)?;
        
        // Send ERASE_WR_BLOCK_END command
        let end_address = if self.card_info.as_ref().map_or(false, |info| info.high_capacity) {
            (sector + count as u64 - 1) * 512
        } else {
            sector + count as u64 - 1
        };
        let _ = spi.send_command(SdCommand::EraseWriteBlockEnd, end_address, SdResponseType::R1)?;
        
        // Send ERASE command
        let response = spi.send_command(SdCommand::Erase, 0, SdResponseType::R1b)?;
        if response[0] != 0x00 {
            return Err(BlockDeviceError::HardwareError);
        }
        
        info!("Sector erase completed");
        
        Ok(())
    }

    /// Get SD card information
    pub fn get_card_info(&self) -> Result<SdCardInfo, BlockDeviceError> {
        match &self.card_info {
            Some(info) => Ok(info.clone()),
            None => Err(BlockDeviceError::DeviceNotFound),
        }
    }

    /// Check if SD card is ready
    pub fn is_ready(&self) -> bool {
        self.initialized && self.state == SdCardState::Transfer
    }

    /// Get card state
    pub fn get_state(&self) -> SdCardState {
        self.state
    }

    /// Set write protection
    pub fn set_write_protection(&mut self, protected: bool) {
        self.write_protected = protected;
        info!("SD card write protection: {}", if protected { "enabled" } else { "disabled" });
    }

    /// Enable/disable multiple block mode
    pub fn set_multiple_block_mode(&mut self, enabled: bool) {
        self.multiple_block_mode = enabled;
        info!("SD card multiple block mode: {}", if enabled { "enabled" } else { "disabled" });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sd_card_driver_creation() {
        let driver = SdCardDriver::new(BlockDeviceId(1), 10, 11, 12, 13);
        assert_eq!(driver.card_id, BlockDeviceId(1));
        assert!(!driver.is_ready());
    }

    #[test]
    fn test_spi_interface_creation() {
        let spi = SpiInterface::new(10, 11, 12, 13);
        assert_eq!(spi.cs_pin, 10);
        assert_eq!(spi.mosi_pin, 11);
        assert_eq!(spi.current_frequency, 400_000);
    }

    #[test]
    fn test_sd_card_state() {
        let state = SdCardState::Transfer;
        assert_eq!(state as u8, 4);
    }

    #[test]
    fn test_sd_card_type() {
        let card_type = SdCardType::SdHc;
        assert_eq!(card_type as u8, 1);
    }

    #[test]
    fn test_interface_mode() {
        let mode = SdInterfaceMode::Spi;
        assert_eq!(mode as u8, 0);
    }
}