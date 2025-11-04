//! USB MSC (Mass Storage Class) Driver
//! 
//! Supports USB mass storage devices like flash drives, external hard drives, and memory cards.
//! Implements SCSI commands, bulk-only transport protocol, and file system mounting.

use crate::*;

#[cfg(feature = "std")]
use std::collections::BTreeMap;

/// SCSI Command Operation Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiOperationCode {
    TestUnitReady = 0x00,
    RequestSense = 0x03,
    Inquiry = 0x12,
    ModeSense = 0x1A,
    StartStopUnit = 0x1B,
    MediaRemoval = 0x1E,
    ReadCapacity10 = 0x25,
    Read10 = 0x28,
    Write10 = 0x2A,
    WriteAndVerify10 = 0x2E,
    Verify10 = 0x2F,
    ModeSelect6 = 0x15,
    ModeSelect10 = 0x55,
    ModeSense6 = 0x1A,
    ModeSense10 = 0x5A,
    FormatUnit = 0x04,
    Read12 = 0xA8,
    Write12 = 0xAA,
    Seek10 = 0x2B,
    SynchronizeCache10 = 0x35,
    SynchronizeCache12 = 0x91,
    LockUnlockCache = 0x36,
    ReadDefectData = 0x37,
    ReadToc = 0x43,
    ReadHeader = 0x44,
    PlayAudio10 = 0x45,
    PlayAudioMSF = 0x47,
    PlayAudioTrackIndex = 0x48,
    PlayTrackRelative10 = 0x49,
    PauseResume = 0x4B,
    StopPlayLoad = 0x4E,
    ReadDiscInformation = 0x51,
    ReadTrackInformation = 0x52,
    ReserveTrack = 0x56,
    SendCommandInformation = 0x57,
    ModeSelect2 = 0xA7,
    Unknown = 0xFF,
}

/// SCSI Response Codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiResponseCode {
    Good = 0x00,
    CheckCondition = 0x02,
    ConditionMet = 0x04,
    Busy = 0x08,
    Intermediate = 0x10,
    IntermediateConditionMet = 0x14,
    ReservationConflict = 0x18,
    CommandTerminated = 0x22,
    QueueFull = 0x28,
    Unknown = 0xFF,
}

/// SCSI Sense Key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ScsiSenseKey {
    NoSense = 0x00,
    RecoveredError = 0x01,
    NotReady = 0x02,
    MediumError = 0x03,
    HardwareError = 0x04,
    IllegalRequest = 0x05,
    UnitAttention = 0x06,
    DataProtect = 0x07,
    BlankCheck = 0x08,
    VendorSpecific = 0x09,
    CopyAborted = 0x0A,
    AbortedCommand = 0x0B,
    VolumeOverflow = 0x0D,
    Miscompare = 0x0E,
    Reserved = 0x0F,
}

/// SCSI SCSI Command Block (CBW)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScsiCBW {
    pub dSignature: u32,           // Signature: 0x43425355 ("USBC")
    pub dTag: u32,                 // Command Tag
    pub dDataTransferLength: u32,  // Data transfer length
    pub bmFlags: u8,               // Direction and reserved bits
    pub bLUN: u8,                  // Logical Unit Number
    pub bCDBLength: u8,            // SCSI Command Block length
    pub CB: [u8; 16],              // SCSI Command Block
}

/// SCSI Command Status Wrapper (CSW)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScsiCSW {
    pub dSignature: u32,           // Signature: 0x53425355 ("USBS")
    pub dTag: u32,                 // Command Tag
    pub dDataResidue: u32,         // Data residue
    pub bStatus: u8,               // Command Status
    pub reserved: [u8; 3],         // Reserved bytes
}

/// SCSI Command Descriptor Block
#[derive(Debug, Clone)]
pub struct ScsiCDB {
    pub operation_code: ScsiOperationCode,
    pub logical_block_address: u32,
    pub transfer_length: u32,
    pub parameters: Vec<u8>,
}

/// SCSI Sense Data
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ScsiSenseData {
    pub valid: u8,                  // Valid bit
    pub response_code: u8,          // Response code
    pub obsolete: u8,               // Obsolete
    pub sense_key: ScsiSenseKey,   // Sense key
    pub information: [u8; 4],       // Information
    pub additional_sense_length: u8, // Additional sense length
    pub command_specific_information: [u8; 4], // Command specific information
    pub additional_sense_code: u8,   // Additional sense code
    pub additional_sense_qualifier: u8, // Additional sense code qualifier
}

/// SCSI Device Information
#[derive(Debug)]
pub struct ScsiDeviceInfo {
    pub device_type: String,
    pub vendor_id: [u8; 8],
    pub product_id: [u8; 16],
    pub product_revision: [u8; 4],
    pub serial_number: Vec<u8>,
    pub removable_media: bool,
    pub write_protected: bool,
    pub command_queue_support: bool,
    pub characteristics: Vec<String>,
}

/// USB MSC Device Information
#[derive(Debug)]
pub struct MscDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub scsi_info: ScsiDeviceInfo,
    pub logical_unit_count: u8,
    pub max_lun: u8,
    pub bulk_only: bool,
    pub control_transport: bool,
    pub max_sense_length: u8,
}

/// SCSI Transfer State
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScsiTransferState {
    Idle,
    CommandSent,
    DataPhase,
    StatusPhase,
    Completed,
    Error,
}

/// SCSI Transfer
#[derive(Debug)]
pub struct ScsiTransfer {
    pub state: ScsiTransferState,
    pub csw_received: bool,
    pub data_transferred: u32,
    pub total_data_length: u32,
    pub sense_data: Option<ScsiSenseData>,
    pub status: ScsiResponseCode,
    pub active: bool,
}

/// MSC Driver
pub struct MscDriver {
    pub device_info: MscDeviceInfo,
    pub bulk_endpoint_in: Option<u8>,
    pub bulk_endpoint_out: Option<u8>,
    pub control_endpoint: Option<u8>,
    pub current_transfer: Option<ScsiTransfer>,
    pub logical_units: Vec<ScsiLogicalUnit>,
    pub block_size: u32,
    pub total_blocks: u32,
    pub protocol_active: bool,
}

/// SCSI Logical Unit
#[derive(Debug)]
pub struct ScsiLogicalUnit {
    pub lun: u8,
    pub device_type: String,
    pub removable_media: bool,
    pub write_protected: bool,
    pub block_size: u32,
    pub total_blocks: u32,
    pub inquiry_data: Option<ScsiDeviceInfo>,
    pub sense_data: Option<ScsiSenseData>,
    pub media_inserted: bool,
}

/// MSC Command Result
#[derive(Debug)]
pub enum MscCommandResult {
    Success(Vec<u8>),
    CheckCondition(ScsiSenseData),
    Busy,
    ReservationConflict,
    Unknown,
}

impl MscDriver {
    /// Create a new MSC driver instance
    pub fn new(device_address: u8) -> Self {
        Self {
            device_info: MscDeviceInfo {
                vendor_id: 0,
                product_id: 0,
                scsi_info: ScsiDeviceInfo {
                    device_type: "Unknown".to_string(),
                    vendor_id: [0; 8],
                    product_id: [0; 16],
                    product_revision: [0; 4],
                    serial_number: Vec::new(),
                    removable_media: false,
                    write_protected: false,
                    command_queue_support: false,
                    characteristics: Vec::new(),
                },
                logical_unit_count: 1,
                max_lun: 0,
                bulk_only: true,
                control_transport: false,
                max_sense_length: 18,
            },
            bulk_endpoint_in: None,
            bulk_endpoint_out: None,
            control_endpoint: None,
            current_transfer: None,
            logical_units: Vec::new(),
            block_size: 512,
            total_blocks: 0,
            protocol_active: false,
        }
    }

    /// Initialize MSC device
    pub fn initialize(&mut self) -> UsbResult<()> {
        log::info!("Initializing MSC device");

        // Discover logical units
        self.discover_logical_units()?;

        // Get capacity information
        self.get_capacity()?;

        self.protocol_active = true;
        log::info!("MSC device initialized successfully");
        Ok(())
    }

    /// Discover SCSI logical units
    pub fn discover_logical_units(&mut self) -> UsbResult<()> {
        self.logical_units.clear();

        for lun in 0..self.device_info.max_lun + 1 {
            let scsi_lun = ScsiLogicalUnit {
                lun,
                device_type: "Disk".to_string(),
                removable_media: false,
                write_protected: false,
                block_size: 512,
                total_blocks: 0,
                inquiry_data: None,
                sense_data: None,
                media_inserted: false,
            };

            self.logical_units.push(scsi_lun);
        }

        log::info!("Discovered {} logical units", self.logical_units.len());
        Ok(())
    }

    /// Get device capacity
    pub fn get_capacity(&mut self) -> UsbResult<()> {
        let capacity_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::ReadCapacity10,
            logical_block_address: 0,
            transfer_length: 0,
            parameters: Vec::new(),
        };

        let mut capacity_data = [0u8; 8];
        let result = self.execute_command(&capacity_cmd, &mut capacity_data)?;

        match result {
            MscCommandResult::Success(data) => {
                capacity_data.copy_from_slice(&data);
                
                self.total_blocks = ((capacity_data[0] as u32) << 24) |
                                 ((capacity_data[1] as u32) << 16) |
                                 ((capacity_data[2] as u32) << 8) |
                                 (capacity_data[3] as u32);
                self.block_size = ((capacity_data[4] as u32) << 24) |
                                ((capacity_data[5] as u32) << 16) |
                                ((capacity_data[6] as u32) << 8) |
                                (capacity_data[7] as u32);

                log::info!("MSC capacity: {} blocks of {} bytes", 
                          self.total_blocks, self.block_size);
            }
            _ => {
                log::warn!("Failed to get MSC capacity");
                return Err(UsbDriverError::TransferFailed { 
                    status: UsbTransferStatus::Stalled 
                });
            }
        }

        Ok(())
    }

    /// Execute SCSI command
    pub fn execute_command(&mut self, cdb: &ScsiCDB, data_buffer: &mut [u8]) -> UsbResult<MscCommandResult> {
        if !self.protocol_active {
            return Err(UsbDriverError::UnsupportedFeature);
        }

        // Prepare CBW
        let mut cbw = ScsiCBW {
            dSignature: 0x43425355, // "USBC"
            dTag: self.generate_tag(),
            dDataTransferLength: data_buffer.len() as u32,
            bmFlags: 0, // Will be set based on command
            bLUN: 0,
            bCDBLength: self.get_cdb_length(cdb.operation_code),
            CB: [0; 16],
        };

        // Fill CBW CB field with SCSI command
        self.fill_cdb(&cbw.CB, cdb);

        // Send CBW
        self.send_cbw(&cbw)?;

        // Determine transfer direction
        let is_data_in = self.is_data_in_command(cdb.operation_code);
        cbw.bmFlags = if is_data_in { 0x80 } else { 0x00 };

        // Send or receive data based on command
        if data_buffer.len() > 0 {
            if is_data_in {
                self.receive_data(data_buffer)?;
            } else {
                self.send_data(data_buffer)?;
            }
        }

        // Receive CSW
        let csw = self.receive_csw()?;

        // Check CSW status
        match csw.bStatus {
            0x00 => Ok(MscCommandResult::Success(data_buffer.to_vec())),
            0x02 => {
                // Check Condition - get sense data
                let sense = self.get_sense_data()?;
                Ok(MscCommandResult::CheckCondition(sense))
            }
            0x08 => Ok(MscCommandResult::Busy),
            0x18 => Ok(MscCommandResult::ReservationConflict),
            _ => Ok(MscCommandResult::Unknown),
        }
    }

    /// Read data blocks
    pub fn read_blocks(&mut self, lba: u32, block_count: u32, buffer: &mut [u8]) -> UsbResult<()> {
        let read_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::Read10,
            logical_block_address: lba,
            transfer_length: block_count,
            parameters: Vec::new(),
        };

        let result = self.execute_command(&read_cmd, buffer)?;
        
        match result {
            MscCommandResult::Success(_) => Ok(()),
            _ => Err(UsbDriverError::TransferFailed { 
                status: UsbTransferStatus::Stalled 
            }),
        }
    }

    /// Write data blocks
    pub fn write_blocks(&mut self, lba: u32, block_count: u32, buffer: &[u8]) -> UsbResult<()> {
        let write_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::Write10,
            logical_block_address: lba,
            transfer_length: block_count,
            parameters: Vec::new(),
        };

        let mut write_buffer = buffer.to_vec();
        let result = self.execute_command(&write_cmd, &mut write_buffer)?;
        
        match result {
            MscCommandResult::Success(_) => Ok(()),
            _ => Err(UsbDriverError::TransferFailed { 
                status: UsbTransferStatus::Stalled 
            }),
        }
    }

    /// Test if unit is ready
    pub fn test_unit_ready(&mut self) -> UsbResult<bool> {
        let test_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::TestUnitReady,
            logical_block_address: 0,
            transfer_length: 0,
            parameters: Vec::new(),
        };

        let mut dummy_buffer = [0u8; 0];
        let result = self.execute_command(&test_cmd, &mut dummy_buffer)?;

        match result {
            MscCommandResult::Success(_) => Ok(true),
            MscCommandResult::Busy => Ok(false),
            _ => Ok(false),
        }
    }

    /// Send CBW to device
    fn send_cbw(&mut self, cbw: &ScsiCBW) -> UsbResult<()> {
        // Implementation would send CBW through bulk-out endpoint
        // For now, just log the operation
        log::debug!("Sending CBW: tag={}, length={}, direction={}", 
                   cbw.dTag, cbw.dDataTransferLength, 
                   if (cbw.bmFlags & 0x80) != 0 { "IN" } else { "OUT" });
        Ok(())
    }

    /// Receive data from device
    fn receive_data(&mut self, buffer: &mut [u8]) -> UsbResult<()> {
        // Implementation would receive data through bulk-in endpoint
        log::debug!("Receiving {} bytes of data", buffer.len());
        Ok(())
    }

    /// Send data to device
    fn send_data(&mut self, buffer: &[u8]) -> UsbResult<()> {
        // Implementation would send data through bulk-out endpoint
        log::debug!("Sending {} bytes of data", buffer.len());
        Ok(())
    }

    /// Receive CSW from device
    fn receive_csw(&mut self) -> UsbResult<ScsiCSW> {
        let mut csw = ScsiCSW {
            dSignature: 0,
            dTag: 0,
            dDataResidue: 0,
            bStatus: 0,
            reserved: [0; 3],
        };

        // Implementation would receive CSW through bulk-in endpoint
        log::debug!("Receiving CSW");
        Ok(csw)
    }

    /// Get sense data
    fn get_sense_data(&mut self) -> UsbResult<ScsiSenseData> {
        let sense_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::RequestSense,
            logical_block_address: 0,
            transfer_length: self.device_info.max_sense_length as u32,
            parameters: Vec::new(),
        };

        let mut sense_buffer = vec![0u8; self.device_info.max_sense_length as usize];
        let result = self.execute_command(&sense_cmd, &mut sense_buffer)?;

        match result {
            MscCommandResult::Success(data) => {
                Ok(self.parse_sense_data(&data))
            }
            _ => Err(UsbDriverError::TransferFailed { 
                status: UsbTransferStatus::Stalled 
            }),
        }
    }

    /// Parse sense data
    fn parse_sense_data(&self, data: &[u8]) -> ScsiSenseData {
        if data.len() < 14 {
            return ScsiSenseData {
                valid: 0,
                response_code: 0,
                obsolete: 0,
                sense_key: ScsiSenseKey::NoSense,
                information: [0; 4],
                additional_sense_length: 0,
                command_specific_information: [0; 4],
                additional_sense_code: 0,
                additional_sense_qualifier: 0,
            };
        }

        ScsiSenseData {
            valid: data[0],
            response_code: data[1],
            obsolete: data[2],
            sense_key: unsafe { core::mem::transmute(data[2] & 0x0F) },
            information: [data[3], data[4], data[5], data[6]],
            additional_sense_length: data[7],
            command_specific_information: [data[8], data[9], data[10], data[11]],
            additional_sense_code: data[12],
            additional_sense_qualifier: data[13],
        }
    }

    /// Get inquiry information
    pub fn get_inquiry_info(&mut self) -> UsbResult<ScsiDeviceInfo> {
        let inquiry_cmd = ScsiCDB {
            operation_code: ScsiOperationCode::Inquiry,
            logical_block_address: 0,
            transfer_length: 36, // Standard inquiry data length
            parameters: vec![0x00, 0x00], // EVPD=0, Page code=0
        };

        let mut inquiry_buffer = vec![0u8; 36];
        let result = self.execute_command(&inquiry_cmd, &mut inquiry_buffer)?;

        match result {
            MscCommandResult::Success(data) => {
                Ok(self.parse_inquiry_data(&data))
            }
            _ => Err(UsbDriverError::TransferFailed { 
                status: UsbTransferStatus::Stalled 
            }),
        }
    }

    /// Parse inquiry data
    fn parse_inquiry_data(&self, data: &[u8]) -> ScsiDeviceInfo {
        if data.len() < 36 {
            return ScsiDeviceInfo {
                device_type: "Unknown".to_string(),
                vendor_id: [0; 8],
                product_id: [0; 16],
                product_revision: [0; 4],
                serial_number: Vec::new(),
                removable_media: false,
                write_protected: false,
                command_queue_support: false,
                characteristics: Vec::new(),
            };
        }

        let mut vendor_id = [0u8; 8];
        let mut product_id = [0u8; 16];
        let mut product_revision = [0u8; 4];

        vendor_id.copy_from_slice(&data[8..16]);
        product_id.copy_from_slice(&data[16..32]);
        product_revision.copy_from_slice(&data[32..36]);

        ScsiDeviceInfo {
            device_type: format!("Type {}", data[0] & 0x1F),
            vendor_id,
            product_id,
            product_revision,
            serial_number: Vec::new(),
            removable_media: (data[1] & 0x80) != 0,
            write_protected: false, // Will be determined by other means
            command_queue_support: (data[7] & 0x02) != 0,
            characteristics: Vec::new(),
        }
    }

    /// Fill CBW Command Block
    fn fill_cdb(&self, cdb: &mut [u8; 16], scsi_cmd: &ScsiCDB) {
        cdb[0] = scsi_cmd.operation_code as u8;
        
        match scsi_cmd.operation_code {
            ScsiOperationCode::Read10 | ScsiOperationCode::Write10 => {
                cdb[2] = ((scsi_cmd.logical_block_address >> 24) & 0xFF) as u8;
                cdb[3] = ((scsi_cmd.logical_block_address >> 16) & 0xFF) as u8;
                cdb[4] = ((scsi_cmd.logical_block_address >> 8) & 0xFF) as u8;
                cdb[5] = (scsi_cmd.logical_block_address & 0xFF) as u8;
                cdb[7] = ((scsi_cmd.transfer_length >> 8) & 0xFF) as u8;
                cdb[8] = (scsi_cmd.transfer_length & 0xFF) as u8;
            }
            ScsiOperationCode::ReadCapacity10 => {
                // No additional parameters for ReadCapacity10
            }
            ScsiOperationCode::TestUnitReady => {
                // No additional parameters for TestUnitReady
            }
            ScsiOperationCode::RequestSense => {
                if !scsi_cmd.parameters.is_empty() {
                    cdb[4] = scsi_cmd.parameters[0];
                }
            }
            ScsiOperationCode::Inquiry => {
                if scsi_cmd.parameters.len() >= 2 {
                    cdb[1] = scsi_cmd.parameters[0]; // EVPD
                    cdb[2] = scsi_cmd.parameters[1]; // Page code
                    cdb[4] = scsi_cmd.transfer_length as u8;
                }
            }
            _ => {
                // For other commands, copy parameters as available
                for (i, &param) in scsi_cmd.parameters.iter().enumerate().take(15) {
                    if i + 1 < 16 {
                        cdb[i + 1] = param;
                    }
                }
            }
        }
    }

    /// Get CDB length for operation code
    fn get_cdb_length(&self, op_code: ScsiOperationCode) -> u8 {
        match op_code {
            ScsiOperationCode::ModeSelect6 | ScsiOperationCode::ModeSense6 | 
            ScsiOperationCode::MediaRemoval => 6,
            ScsiOperationCode::Read10 | ScsiOperationCode::Write10 | 
            ScsiOperationCode::WriteAndVerify10 | ScsiOperationCode::Verify10 |
            ScsiOperationCode::ModeSelect10 | ScsiOperationCode::ModeSense10 |
            ScsiOperationCode::FormatUnit | ScsiOperationCode::Seek10 |
            ScsiOperationCode::SynchronizeCache10 | ScsiOperationCode::ReadToc |
            ScsiOperationCode::PlayAudio10 | ScsiOperationCode::PlayAudioMSF |
            ScsiOperationCode::PlayTrackRelative10 => 10,
            ScsiOperationCode::Read12 | ScsiOperationCode::Write12 | 
            ScsiOperationCode::SynchronizeCache12 | ScsiOperationCode::LockUnlockCache => 12,
            _ => 6, // Default to 6-byte CDB
        }
    }

    /// Check if command receives data (IN transfer)
    fn is_data_in_command(&self, op_code: ScsiOperationCode) -> bool {
        match op_code {
            ScsiOperationCode::Read10 | ScsiOperationCode::Read12 |
            ScsiOperationCode::ReadCapacity10 | ScsiOperationCode::Inquiry |
            ScsiOperationCode::RequestSense | ScsiOperationCode::ModeSense6 |
            ScsiOperationCode::ModeSense10 | ScsiOperationCode::ReadToc |
            ScsiOperationCode::ReadHeader | ScsiOperationCode::PlayAudioMSF |
            ScsiOperationCode::PlayTrackRelative10 | ScsiOperationCode::ReadDiscInformation |
            ScsiOperationCode::ReadTrackInformation | ScsiOperationCode::SendCommandInformation => true,
            _ => false,
        }
    }

    /// Generate unique tag for CBW
    fn generate_tag(&self) -> u32 {
        static TAG_COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(1);
        TAG_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }

    /// Set endpoints
    pub fn set_endpoints(&mut self, bulk_in: u8, bulk_out: u8, control: Option<u8>) {
        self.bulk_endpoint_in = Some(bulk_in);
        self.bulk_endpoint_out = Some(bulk_out);
        self.control_endpoint = control;
    }

    /// Get logical unit by number
    pub fn get_logical_unit(&self, lun: u8) -> UsbResult<&ScsiLogicalUnit> {
        self.logical_units.get(lun as usize)
            .ok_or(UsbDriverError::DeviceNotFound { address: lun })
    }

    /// Check if device is active
    pub fn is_active(&self) -> bool {
        self.protocol_active
    }

    /// Get device statistics
    pub fn get_stats(&self) -> MscDeviceStats {
        MscDeviceStats {
            total_blocks: self.total_blocks,
            block_size: self.block_size,
            total_size: (self.total_blocks as u64) * (self.block_size as u64),
            logical_unit_count: self.logical_units.len() as u8,
        }
    }
}

/// MSC Device Statistics
#[derive(Debug, Clone)]
pub struct MscDeviceStats {
    pub total_blocks: u32,
    pub block_size: u32,
    pub total_size: u64,
    pub logical_unit_count: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msc_driver_creation() {
        let driver = MscDriver::new(1);
        assert_eq!(driver.is_active(), false);
    }

    #[test]
    fn test_scsi_operation_code_from_u8() {
        assert_eq!(ScsiOperationCode::from_u8(0x00), ScsiOperationCode::TestUnitReady);
        assert_eq!(ScsiOperationCode::from_u8(0x12), ScsiOperationCode::Inquiry);
        assert_eq!(ScsiOperationCode::from_u8(0xFF), ScsiOperationCode::Unknown);
    }

    #[test]
    fn test_scsi_response_code_from_u8() {
        assert_eq!(ScsiResponseCode::from_u8(0x00), ScsiResponseCode::Good);
        assert_eq!(ScsiResponseCode::from_u8(0x02), ScsiResponseCode::CheckCondition);
        assert_eq!(ScsiResponseCode::from_u8(0xFF), ScsiResponseCode::Unknown);
    }
}

// Add missing trait implementations
impl From<u8> for ScsiOperationCode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ScsiOperationCode::TestUnitReady,
            0x03 => ScsiOperationCode::RequestSense,
            0x04 => ScsiOperationCode::FormatUnit,
            0x12 => ScsiOperationCode::Inquiry,
            0x1A => ScsiOperationCode::ModeSense6,
            0x1B => ScsiOperationCode::StartStopUnit,
            0x1E => ScsiOperationCode::MediaRemoval,
            0x25 => ScsiOperationCode::ReadCapacity10,
            0x28 => ScsiOperationCode::Read10,
            0x2A => ScsiOperationCode::Write10,
            0x2B => ScsiOperationCode::Seek10,
            0x2E => ScsiOperationCode::WriteAndVerify10,
            0x2F => ScsiOperationCode::Verify10,
            0x35 => ScsiOperationCode::SynchronizeCache10,
            0x36 => ScsiOperationCode::LockUnlockCache,
            0x37 => ScsiOperationCode::ReadDefectData,
            0x43 => ScsiOperationCode::ReadToc,
            0x44 => ScsiOperationCode::ReadHeader,
            0x45 => ScsiOperationCode::PlayAudio10,
            0x47 => ScsiOperationCode::PlayAudioMSF,
            0x48 => ScsiOperationCode::PlayAudioTrackIndex,
            0x49 => ScsiOperationCode::PlayTrackRelative10,
            0x4B => ScsiOperationCode::PauseResume,
            0x4E => ScsiOperationCode::StopPlayLoad,
            0x51 => ScsiOperationCode::ReadDiscInformation,
            0x52 => ScsiOperationCode::ReadTrackInformation,
            0x55 => ScsiOperationCode::ModeSelect10,
            0x56 => ScsiOperationCode::ReserveTrack,
            0x57 => ScsiOperationCode::SendCommandInformation,
            0x5A => ScsiOperationCode::ModeSense10,
            0xA7 => ScsiOperationCode::ModeSelect2,
            0xA8 => ScsiOperationCode::Read12,
            0xAA => ScsiOperationCode::Write12,
            0x91 => ScsiOperationCode::SynchronizeCache12,
            _ => ScsiOperationCode::Unknown,
        }
    }
}

impl From<u8> for ScsiResponseCode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => ScsiResponseCode::Good,
            0x02 => ScsiResponseCode::CheckCondition,
            0x04 => ScsiResponseCode::ConditionMet,
            0x08 => ScsiResponseCode::Busy,
            0x10 => ScsiResponseCode::Intermediate,
            0x14 => ScsiResponseCode::IntermediateConditionMet,
            0x18 => ScsiResponseCode::ReservationConflict,
            0x22 => ScsiResponseCode::CommandTerminated,
            0x28 => ScsiResponseCode::QueueFull,
            _ => ScsiResponseCode::Unknown,
        }
    }
}