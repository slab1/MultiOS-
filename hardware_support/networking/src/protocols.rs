//! Network Protocol Parsing and Analysis
//! 
//! This module provides comprehensive protocol parsing for Wi-Fi and Ethernet:
//! - Wi-Fi (802.11) frame parsing
//! - Ethernet frame parsing
//! - TCP/IP protocol parsing
//! - ICMP, ARP, DHCP protocol handling
//! - Security protocol frame parsing
//! - Packet capture and analysis
//! - Protocol validation and verification
//! - Network diagnostic tools
//! - Frame generation and transmission

use crate::{NetworkingError};
use multios_memory::{MemoryManager, PhysicalAddress, VirtualAddress};
use bitflags::bitflags;
use core::fmt;

/// Protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolType {
    Ethernet,
    IPv4,
    IPv6,
    Arp,
    Icmp,
    Icmpv6,
    Tcp,
    Udp,
    Dhcp,
    Dns,
    WifiMgmt,
    WifiCtrl,
    WifiData,
}

/// Frame types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Management,
    Control,
    Data,
}

/// Frame subtypes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSubtype {
    // Management frames
    AssociationRequest,
    AssociationResponse,
    ReassociationRequest,
    ReassociationResponse,
    ProbeRequest,
    ProbeResponse,
    Beacon,
    Atim,
    Disassociation,
    Authentication,
    Deauthentication,
    Action,
    
    // Control frames
    PowerSavePoll,
    RequestToSend,
    ClearToSend,
    Acknowledgment,
    ContentionFreeEnd,
    CFEnd,
    CFEndCFStart,
    
    // Data frames
    Data,
    DataCFACK,
    DataCFPoll,
    DataCFAckCFpoll,
    Null,
    CFACK,
    CFPoll,
    CFAckCFpoll,
}

/// Ethernet header
#[derive(Debug, Clone)]
pub struct EthernetHeader {
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub ethertype: u16,
    pub vlan_tag: Option<VlanTag>,
    pub priority: u8,  // Priority code point (PCP)
    pub drop_eligible: bool,
}

/// VLAN tag structure
#[derive(Debug, Clone)]
pub struct VlanTag {
    pub tpid: u16,   // Tag Protocol Identifier (0x8100)
    pub tci: u16,    // Tag Control Information
}

/// Wi-Fi management frame header
#[derive(Debug, Clone)]
pub struct WifiManagementHeader {
    pub frame_control: FrameControl,
    pub duration: u16,
    pub destination_mac: [u8; 6],
    pub source_mac: [u8; 6],
    pub bssid: [u8; 6],
    pub sequence_control: SequenceControl,
}

/// Frame control field
#[derive(Debug, Clone)]
pub struct FrameControl {
    pub protocol_version: u8,   // 2 bits
    pub frame_type: FrameType,  // 2 bits
    pub frame_subtype: FrameSubtype, // 4 bits
    pub to_ds: bool,
    pub from_ds: bool,
    pub more_fragments: bool,
    pub retry: bool,
    pub power_management: bool,
    pub more_data: bool,
    pub protected: bool,
    pub order: bool,
}

/// Sequence control field
#[derive(Debug, Clone)]
pub struct SequenceControl {
    pub sequence_number: u16,   // 12 bits
    pub fragment_number: u16,   // 4 bits
}

/// Beacon frame
#[derive(Debug, Clone)]
pub struct BeaconFrame {
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capability_info: CapabilityInfo,
    pub ssid: String,
    pub supported_rates: Vec<u8>,
    pub ds_parameter: Option<DsParameter>,
    pub rsn_information: Option<RsnInformation>,
    pub extended_rates: Vec<u8>,
    pub country_information: Option<CountryInformation>,
}

/// Capability information
#[derive(Debug, Clone)]
pub struct CapabilityInfo {
    pub ess: bool,              // Extended Service Set
    pub ibss: bool,             // Independent BSS
    pub cf_pollable: bool,      // CFPollable
    pub cf_poll_request: bool,  // CFPollRequest
    pub privacy: bool,          // Privacy
    pub short_preamble: bool,
    pub pbcc: bool,             // PBCC
    pub channel_agility: bool,
    pub spectrum_mgmt: bool,
    pub short_slot_time: bool,
    pub apsd: bool,             // Automatic Power Save Delivery
    pub radio_measurement: bool,
    pub dsss_ofdm: bool,        // DSSS-OFDM
    pub delayed_block_ack: bool,
    pub immediate_block_ack: bool,
}

/// DS parameter
#[derive(Debug, Clone)]
pub struct DsParameter {
    pub current_channel: u8,
}

/// RSN information (WPA/WPA2)
#[derive(Debug, Clone)]
pub struct RsnInformation {
    pub version: u16,
    pub group_suite: [u8; 4],
    pub pairwise_suite_count: u16,
    pub pairwise_suites: Vec<[u8; 4]>,
    pub akm_suite_count: u16,
    pub akm_suites: Vec<[u8; 4]>,
    pub capabilities: u16,
    pub pmkid_count: u16,
    pub pmkids: Vec<u8>,
}

/// Country information
#[derive(Debug, Clone)]
pub struct CountryInformation {
    pub country_code: [u8; 3],
    pub first_channel: u8,
    pub number_of_channels: u8,
    pub maximum_transmit_power: u8,
}

/// Probe request frame
#[derive(Debug, Clone)]
pub struct ProbeRequestFrame {
    pub ssid: String,
    pub supported_rates: Vec<u8>,
    pub extended_rates: Vec<u8>,
    pub ssid_list: Vec<String>,
    pub channel_candidate_list: Vec<u8>,
}

/// Probe response frame
#[derive(Debug, Clone)]
pub struct ProbeResponseFrame {
    pub timestamp: u64,
    pub beacon_interval: u16,
    pub capability_info: CapabilityInfo,
    pub ssid: String,
    pub supported_rates: Vec<u8>,
    pub extended_rates: Vec<u8>,
    pub ds_parameter: Option<DsParameter>,
    pub rsn_information: Option<RsnInformation>,
}

/// Association request frame
#[derive(Debug, Clone)]
pub struct AssociationRequestFrame {
    pub capability_info: CapabilityInfo,
    pub listen_interval: u16,
    pub ssid: String,
    pub supported_rates: Vec<u8>,
    pub extended_rates: Vec<u8>,
    pub supported_mcs: Option<Vec<u8>>,
    pub supported_ht_capabilities: Option<HtCapabilities>,
    pub supported_vht_capabilities: Option<VhtCapabilities>,
}

/// HT (802.11n) capabilities
#[derive(Debug, Clone)]
pub struct HtCapabilities {
    pub ht_capabilities_info: u16,
    pub ampdu_parameters: u8,
    pub supported_mcs_set: [u8; 16],
    pub ht_extended_capabilities: u16,
    pub tx_beamforming_capabilities: u32,
    pub asel_capabilities: u8,
}

/// VHT (802.11ac) capabilities
#[derive(Debug, Clone)]
pub struct VhtCapabilities {
    pub vht_capabilities_info: u32,
    pub supported_vht_mcs_nss: [u8; 8],
    pub vht_extended_capabilities: u16,
    pub vht_tx_beamforming_capabilities: u32,
    pub vht_rx_beamforming_capabilities: u32,
}

/// Authentication frame
#[derive(Debug, Clone)]
pub struct AuthenticationFrame {
    pub algorithm_number: u16,
    pub sequence_number: u16,
    pub status_code: u16,
    pub challenge_text: Option<Vec<u8>>,
}

/// Deauthentication frame
#[derive(Debug, Clone)]
pub struct DeauthenticationFrame {
    pub reason_code: u16,
    pub bssid: [u8; 6],
}

/// TCP header
#[derive(Debug, Clone)]
pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence_number: u32,
    pub acknowledgment_number: u32,
    pub data_offset: u8,    // 4 bits
    pub reserved: u8,       // 3 bits
    pub flags: TcpFlags,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
    pub options: Vec<TcpOption>,
}

/// TCP flags
#[derive(Debug, Clone)]
pub struct TcpFlags {
    pub fin: bool,
    pub syn: bool,
    pub rst: bool,
    pub psh: bool,
    pub ack: bool,
    pub urg: bool,
    pub ece: bool,
    pub cwr: bool,
}

/// TCP options
#[derive(Debug, Clone)]
pub struct TcpOption {
    pub kind: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

/// UDP header
#[derive(Debug, Clone)]
pub struct UdpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub length: u16,
    pub checksum: u16,
}

/// ICMP header
#[derive(Debug, Clone)]
pub struct IcmpHeader {
    pub type_code: u8,
    pub code: u8,
    pub checksum: u16,
    pub identifier: u16,
    pub sequence_number: u16,
    pub data: Vec<u8>,
}

/// ARP header
#[derive(Debug, Clone)]
pub struct ArpHeader {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_size: u8,
    pub protocol_size: u8,
    pub operation: ArpOperation,
    pub sender_hardware_address: [u8; 6],
    pub sender_protocol_address: [u8; 4],
    pub target_hardware_address: [u8; 6],
    pub target_protocol_address: [u8; 4],
}

/// ARP operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpOperation {
    Request = 1,
    Reply = 2,
    ReverseRequest = 3,
    ReverseReply = 4,
}

/// DHCP message
#[derive(Debug, Clone)]
pub struct DhcpMessage {
    pub operation: u8,
    pub hardware_type: u8,
    pub hardware_address_length: u8,
    pub hops: u8,
    pub transaction_id: u32,
    pub seconds: u16,
    pub flags: u16,
    pub client_ip: [u8; 4],
    pub your_ip: [u8; 4],
    pub server_ip: [u8; 4],
    pub gateway_ip: [u8; 4],
    pub client_hardware_address: [u8; 16],
    pub server_name: [u8; 64],
    pub boot_filename: [u8; 128],
    pub options: Vec<DhcpOption>,
}

/// DHCP options
#[derive(Debug, Clone)]
pub struct DhcpOption {
    pub code: u8,
    pub length: u8,
    pub data: Vec<u8>,
}

/// Main protocol parser
pub struct ProtocolParser {
    memory_manager: &'static MemoryManager,
    debug_mode: bool,
    validation_enabled: bool,
    packet_capture_enabled: bool,
}

impl ProtocolParser {
    /// Create a new protocol parser
    pub fn new() -> Result<Self, NetworkingError> {
        Ok(Self {
            memory_manager: unsafe { &*0x2000 }, // TODO: Proper reference
            debug_mode: false,
            validation_enabled: true,
            packet_capture_enabled: false,
        })
    }
    
    /// Parse Ethernet frame
    pub fn parse_ethernet_frame(&self, data: &[u8]) -> Result<(EthernetHeader, ProtocolType, Vec<u8>), NetworkingError> {
        if data.len() < 14 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let destination_mac = [
            data[0], data[1], data[2], data[3], data[4], data[5]
        ];
        let source_mac = [
            data[6], data[7], data[8], data[9], data[10], data[11]
        ];
        let ethertype = u16::from_be_bytes([data[12], data[13]]);
        
        let mut offset = 14;
        let mut vlan_tag = None;
        let mut priority = 0;
        let mut drop_eligible = false;
        
        // Check for VLAN tag
        if ethertype == 0x8100 {
            if data.len() < 18 {
                return Err(NetworkingError::InvalidConfiguration);
            }
            
            vlan_tag = Some(VlanTag {
                tpid: ethertype,
                tci: u16::from_be_bytes([data[offset], data[offset + 1]]),
            });
            
            // Extract PCP and DEI
            priority = ((vlan_tag.as_ref().unwrap().tci >> 13) & 0x7) as u8;
            drop_eligible = (vlan_tag.as_ref().unwrap().tci & 0x1000) != 0;
            
            offset += 2;
        }
        
        let header = EthernetHeader {
            destination_mac,
            source_mac,
            ethertype,
            vlan_tag,
            priority,
            drop_eligible,
        };
        
        // Determine protocol type
        let protocol_type = match ethertype {
            0x0800 => ProtocolType::Ipv4,
            0x0806 => ProtocolType::Arp,
            0x86DD => ProtocolType::Ipv6,
            _ => ProtocolType::Ethernet,
        };
        
        let payload = data[offset..].to_vec();
        
        Ok((header, protocol_type, payload))
    }
    
    /// Parse Wi-Fi frame
    pub fn parse_wifi_frame(&self, data: &[u8]) -> Result<(WifiManagementHeader, FrameType, FrameSubtype, Vec<u8>), NetworkingError> {
        if data.len() < 24 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        // Parse frame control
        let frame_control_word = u16::from_be_bytes([data[0], data[1]]);
        let protocol_version = (frame_control_word >> 14) & 0x03;
        let frame_type_value = (frame_control_word >> 12) & 0x03;
        let frame_subtype_value = (frame_control_word >> 8) & 0x0F;
        
        let frame_type = match frame_type_value {
            0 => FrameType::Management,
            1 => FrameType::Control,
            2 => FrameType::Data,
            _ => FrameType::Management,
        };
        
        let frame_subtype = self.parse_frame_subtype(frame_type, frame_subtype_value);
        
        let more_fragments = (frame_control_word & 0x0800) != 0;
        let retry = (frame_control_word & 0x0400) != 0;
        let power_management = (frame_control_word & 0x0200) != 0;
        let more_data = (frame_control_word & 0x0100) != 0;
        let protected = (frame_control_word & 0x0080) != 0;
        let order = (frame_control_word & 0x0040) != 0;
        
        let frame_control = FrameControl {
            protocol_version: protocol_version as u8,
            frame_type,
            frame_subtype,
            to_ds: (frame_control_word & 0x0100) != 0,
            from_ds: (frame_control_word & 0x0200) != 0,
            more_fragments,
            retry,
            power_management,
            more_data,
            protected,
            order,
        };
        
        let duration = u16::from_be_bytes([data[2], data[3]]);
        
        let destination_mac = [
            data[4], data[5], data[6], data[7], data[8], data[9]
        ];
        let source_mac = [
            data[10], data[11], data[12], data[13], data[14], data[15]
        ];
        let bssid = [
            data[16], data[17], data[18], data[19], data[20], data[21]
        ];
        
        let sequence_control_word = u16::from_be_bytes([data[22], data[23]]);
        let sequence_control = SequenceControl {
            sequence_number: (sequence_control_word >> 4) & 0x0FFF,
            fragment_number: sequence_control_word & 0x0F,
        };
        
        let header = WifiManagementHeader {
            frame_control,
            duration,
            destination_mac,
            source_mac,
            bssid,
            sequence_control,
        };
        
        let payload = data[24..].to_vec();
        
        Ok((header, frame_type, frame_subtype, payload))
    }
    
    /// Parse frame subtype
    fn parse_frame_subtype(&self, frame_type: FrameType, subtype_value: u16) -> FrameSubtype {
        match frame_type {
            FrameType::Management => match subtype_value {
                0 => FrameSubtype::AssociationRequest,
                1 => FrameSubtype::AssociationResponse,
                2 => FrameSubtype::ReassociationRequest,
                3 => FrameSubtype::ReassociationResponse,
                4 => FrameSubtype::ProbeRequest,
                5 => FrameSubtype::ProbeResponse,
                6 => FrameSubtype::Beacon,
                7 => FrameSubtype::Atim,
                8 => FrameSubtype::Disassociation,
                9 => FrameSubtype::Authentication,
                10 => FrameSubtype::Deauthentication,
                11 => FrameSubtype::Action,
                _ => FrameSubtype::Beacon,
            },
            FrameType::Control => match subtype_value {
                0 => FrameSubtype::PowerSavePoll,
                1 => FrameSubtype::RequestToSend,
                2 => FrameSubtype::ClearToSend,
                3 => FrameSubtype::Acknowledgment,
                4 => FrameSubtype::ContentionFreeEnd,
                5 => FrameSubtype::CFEnd,
                6 => FrameSubtype::CFEndCFStart,
                _ => FrameSubtype::Acknowledgment,
            },
            FrameType::Data => match subtype_value {
                0 => FrameSubtype::Data,
                1 => FrameSubtype::DataCFACK,
                2 => FrameSubtype::DataCFPoll,
                3 => FrameSubtype::DataCFAckCFpoll,
                4 => FrameSubtype::Null,
                5 => FrameSubtype::CFACK,
                6 => FrameSubtype::CFPoll,
                7 => FrameSubtype::CFAckCFpoll,
                _ => FrameSubtype::Data,
            },
        }
    }
    
    /// Parse Beacon frame
    pub fn parse_beacon_frame(&self, data: &[u8]) -> Result<BeaconFrame, NetworkingError> {
        if data.len() < 12 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let timestamp = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]
        ]);
        let beacon_interval = u16::from_be_bytes([data[8], data[9]]);
        let capability_info_word = u16::from_be_bytes([data[10], data[11]]);
        
        let capability_info = CapabilityInfo {
            ess: (capability_info_word & 0x0001) != 0,
            ibss: (capability_info_word & 0x0002) != 0,
            cf_pollable: (capability_info_word & 0x0004) != 0,
            cf_poll_request: (capability_info_word & 0x0008) != 0,
            privacy: (capability_info_word & 0x0010) != 0,
            short_preamble: (capability_info_word & 0x0020) != 0,
            pbcc: (capability_info_word & 0x0040) != 0,
            channel_agility: (capability_info_word & 0x0080) != 0,
            spectrum_mgmt: (capability_info_word & 0x0100) != 0,
            short_slot_time: (capability_info_word & 0x0200) != 0,
            apsd: (capability_info_word & 0x0400) != 0,
            radio_measurement: (capability_info_word & 0x0800) != 0,
            dsss_ofdm: (capability_info_word & 0x1000) != 0,
            delayed_block_ack: (capability_info_word & 0x2000) != 0,
            immediate_block_ack: (capability_info_word & 0x4000) != 0,
        };
        
        let mut offset = 12;
        let mut ssid = String::new();
        let mut supported_rates = Vec::new();
        let mut extended_rates = Vec::new();
        let mut ds_parameter = None;
        let mut rsn_information = None;
        let mut country_information = None;
        
        // Parse tagged parameters
        while offset < data.len() {
            if offset + 2 > data.len() {
                break;
            }
            
            let tag_number = data[offset];
            let tag_length = data[offset + 1] as usize;
            offset += 2;
            
            if offset + tag_length > data.len() {
                break;
            }
            
            match tag_number {
                0 => { // SSID
                    ssid = String::from_utf8_lossy(&data[offset..offset + tag_length]).to_string();
                }
                1 => { // Supported Rates
                    supported_rates = data[offset..offset + tag_length].to_vec();
                }
                3 => { // DS Parameter
                    if tag_length >= 1 {
                        ds_parameter = Some(DsParameter {
                            current_channel: data[offset],
                        });
                    }
                }
                4 => { // Extended Supported Rates
                    extended_rates = data[offset..offset + tag_length].to_vec();
                }
                48 => { // RSN Information
                    if tag_length >= 2 {
                        let version = u16::from_be_bytes([data[offset], data[offset + 1]]);
                        let group_suite = [
                            data[offset + 2], data[offset + 3], data[offset + 4], data[offset + 5]
                        ];
                        
                        let mut pos = offset + 6;
                        let pairwise_count = if tag_length >= pos - offset + 2 {
                            u16::from_be_bytes([data[pos], data[pos + 1]]) as usize
                        } else { 0 };
                        pos += 2;
                        
                        let mut pairwise_suites = Vec::new();
                        for _ in 0..pairwise_count {
                            if pos + 4 <= data.len() {
                                pairwise_suites.push([
                                    data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
                                ]);
                                pos += 4;
                            }
                        }
                        
                        let akm_count = if tag_length >= pos - offset + 2 {
                            u16::from_be_bytes([data[pos], data[pos + 1]]) as usize
                        } else { 0 };
                        pos += 2;
                        
                        let mut akm_suites = Vec::new();
                        for _ in 0..akm_count {
                            if pos + 4 <= data.len() {
                                akm_suites.push([
                                    data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
                                ]);
                                pos += 4;
                            }
                        }
                        
                        let capabilities = if tag_length >= pos - offset + 2 {
                            u16::from_be_bytes([data[pos], data[pos + 1]])
                        } else { 0 };
                        
                        rsn_information = Some(RsnInformation {
                            version,
                            group_suite,
                            pairwise_suite_count: pairwise_count as u16,
                            pairwise_suites,
                            akm_suite_count: akm_count as u16,
                            akm_suites,
                            capabilities,
                            pmkid_count: 0,
                            pmkids: Vec::new(),
                        });
                    }
                }
                7 => { // Country Information
                    if tag_length >= 3 {
                        let country_code = [data[offset], data[offset + 1], data[offset + 2]];
                        let first_channel = data[offset + 3];
                        let number_of_channels = data[offset + 4];
                        let maximum_transmit_power = data[offset + 5];
                        
                        country_information = Some(CountryInformation {
                            country_code,
                            first_channel,
                            number_of_channels,
                            maximum_transmit_power,
                        });
                    }
                }
                _ => {},
            }
            
            offset += tag_length;
        }
        
        Ok(BeaconFrame {
            timestamp,
            beacon_interval,
            capability_info,
            ssid,
            supported_rates,
            ds_parameter,
            rsn_information,
            extended_rates,
            country_information,
        })
    }
    
    /// Parse TCP header
    pub fn parse_tcp_header(&self, data: &[u8]) -> Result<TcpHeader, NetworkingError> {
        if data.len() < 20 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let source_port = u16::from_be_bytes([data[0], data[1]]);
        let destination_port = u16::from_be_bytes([data[2], data[3]]);
        let sequence_number = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let acknowledgment_number = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        
        let data_offset = (data[12] >> 4) & 0x0F;
        let reserved = data[12] & 0x0E;
        
        let flags_byte = data[13];
        let flags = TcpFlags {
            fin: (flags_byte & 0x01) != 0,
            syn: (flags_byte & 0x02) != 0,
            rst: (flags_byte & 0x04) != 0,
            psh: (flags_byte & 0x08) != 0,
            ack: (flags_byte & 0x10) != 0,
            urg: (flags_byte & 0x20) != 0,
            ece: (flags_byte & 0x40) != 0,
            cwr: (flags_byte & 0x80) != 0,
        };
        
        let window_size = u16::from_be_bytes([data[14], data[15]]);
        let checksum = u16::from_be_bytes([data[16], data[17]]);
        let urgent_pointer = u16::from_be_bytes([data[18], data[19]]);
        
        let mut options = Vec::new();
        let mut offset = 20;
        
        // Parse TCP options
        while offset < (data_offset as usize) * 4 && offset + 1 < data.len() {
            let kind = data[offset];
            if kind == 0 { // End of options list
                break;
            } else if kind == 1 { // No operation
                offset += 1;
                continue;
            }
            
            if offset + 2 > data.len() {
                break;
            }
            
            let length = data[offset + 1];
            if length < 2 || offset + length > data.len() {
                break;
            }
            
            let option_data = data[offset + 2..offset + length as usize].to_vec();
            options.push(TcpOption {
                kind,
                length,
                data: option_data,
            });
            
            offset += length as usize;
        }
        
        Ok(TcpHeader {
            source_port,
            destination_port,
            sequence_number,
            acknowledgment_number,
            data_offset,
            reserved,
            flags,
            window_size,
            checksum,
            urgent_pointer,
            options,
        })
    }
    
    /// Parse UDP header
    pub fn parse_udp_header(&self, data: &[u8]) -> Result<UdpHeader, NetworkingError> {
        if data.len() < 8 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let source_port = u16::from_be_bytes([data[0], data[1]]);
        let destination_port = u16::from_be_bytes([data[2], data[3]]);
        let length = u16::from_be_bytes([data[4], data[5]]);
        let checksum = u16::from_be_bytes([data[6], data[7]]);
        
        Ok(UdpHeader {
            source_port,
            destination_port,
            length,
            checksum,
        })
    }
    
    /// Parse ARP header
    pub fn parse_arp_header(&self, data: &[u8]) -> Result<ArpHeader, NetworkingError> {
        if data.len() < 28 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let hardware_type = u16::from_be_bytes([data[0], data[1]]);
        let protocol_type = u16::from_be_bytes([data[2], data[3]]);
        let hardware_size = data[4];
        let protocol_size = data[5];
        let operation = match u16::from_be_bytes([data[6], data[7]]) {
            1 => ArpOperation::Request,
            2 => ArpOperation::Reply,
            3 => ArpOperation::ReverseRequest,
            4 => ArpOperation::ReverseReply,
            _ => ArpOperation::Request,
        };
        
        let sender_hardware_address = [
            data[8], data[9], data[10], data[11], data[12], data[13]
        ];
        let sender_protocol_address = [
            data[14], data[15], data[16], data[17]
        ];
        let target_hardware_address = [
            data[18], data[19], data[20], data[21], data[22], data[23]
        ];
        let target_protocol_address = [
            data[24], data[25], data[26], data[27]
        ];
        
        Ok(ArpHeader {
            hardware_type,
            protocol_type,
            hardware_size,
            protocol_size,
            operation,
            sender_hardware_address,
            sender_protocol_address,
            target_hardware_address,
            target_protocol_address,
        })
    }
    
    /// Parse DHCP message
    pub fn parse_dhcp_message(&self, data: &[u8]) -> Result<DhcpMessage, NetworkingError> {
        if data.len() < 236 {
            return Err(NetworkingError::InvalidConfiguration);
        }
        
        let operation = data[0];
        let hardware_type = data[1];
        let hardware_address_length = data[2];
        let hops = data[3];
        let transaction_id = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let seconds = u16::from_be_bytes([data[8], data[9]]);
        let flags = u16::from_be_bytes([data[10], data[11]]);
        
        let client_ip = [data[12], data[13], data[14], data[15]];
        let your_ip = [data[16], data[17], data[18], data[19]];
        let server_ip = [data[20], data[21], data[22], data[23]];
        let gateway_ip = [data[24], data[25], data[26], data[27]];
        
        let mut client_hardware_address = [0u8; 16];
        client_hardware_address[..hardware_address_length as usize]
            .copy_from_slice(&data[28..28 + hardware_address_length as usize]);
        
        let mut server_name = [0u8; 64];
        server_name.copy_from_slice(&data[44..108]);
        
        let mut boot_filename = [0u8; 128];
        boot_filename.copy_from_slice(&data[108..236]);
        
        let mut options = Vec::new();
        let mut offset = 236;
        
        // Parse DHCP options
        while offset + 2 <= data.len() {
            let code = data[offset];
            if code == 255 { // End of options
                break;
            }
            
            let length = data[offset + 1] as usize;
            offset += 2;
            
            if offset + length > data.len() {
                break;
            }
            
            let option_data = data[offset..offset + length].to_vec();
            options.push(DhcpOption {
                code,
                length: length as u8,
                data: option_data,
            });
            
            offset += length;
        }
        
        Ok(DhcpMessage {
            operation,
            hardware_type,
            hardware_address_length,
            hops,
            transaction_id,
            seconds,
            flags,
            client_ip,
            your_ip,
            server_ip,
            gateway_ip,
            client_hardware_address,
            server_name,
            boot_filename,
            options,
        })
    }
    
    /// Enable/disable debug mode
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
        info!("Protocol parser debug mode {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Enable/disable validation
    pub fn set_validation_enabled(&mut self, enabled: bool) {
        self.validation_enabled = enabled;
        info!("Protocol validation {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Enable/disable packet capture
    pub fn set_packet_capture_enabled(&mut self, enabled: bool) {
        self.packet_capture_enabled = enabled;
        info!("Packet capture {}", if enabled { "enabled" } else { "disabled" });
    }
}

/// Protocol analysis and statistics
pub struct ProtocolAnalyzer {
    frame_counts: [u32; 256],
    protocol_counts: [u32; 16],
    error_count: u32,
}

impl ProtocolAnalyzer {
    /// Create a new protocol analyzer
    pub fn new() -> Self {
        Self {
            frame_counts: [0; 256],
            protocol_counts: [0; 16],
            error_count: 0,
        }
    }
    
    /// Analyze a protocol frame
    pub fn analyze_frame(&mut self, protocol_type: ProtocolType) {
        let protocol_index = match protocol_type {
            ProtocolType::Ethernet => 0,
            ProtocolType::IPv4 => 1,
            ProtocolType::IPv6 => 2,
            ProtocolType::Arp => 3,
            ProtocolType::Icmp => 4,
            ProtocolType::Icmpv6 => 5,
            ProtocolType::Tcp => 6,
            ProtocolType::Udp => 7,
            ProtocolType::Dhcp => 8,
            ProtocolType::Dns => 9,
            ProtocolType::WifiMgmt => 10,
            ProtocolType::WifiCtrl => 11,
            ProtocolType::WifiData => 12,
        } as usize;
        
        self.protocol_counts[protocol_index] += 1;
    }
    
    /// Record frame analysis
    pub fn record_frame(&mut self, frame_type: FrameType) {
        let frame_index = match frame_type {
            FrameType::Management => 0,
            FrameType::Control => 1,
            FrameType::Data => 2,
        } as usize;
        
        self.frame_counts[frame_index] += 1;
    }
    
    /// Record error
    pub fn record_error(&mut self) {
        self.error_count += 1;
    }
    
    /// Get analysis statistics
    pub fn get_statistics(&self) -> ProtocolStatistics {
        ProtocolStatistics {
            total_frames: self.frame_counts.iter().sum(),
            total_protocols: self.protocol_counts.iter().sum(),
            error_count: self.error_count,
            frame_distribution: self.frame_counts,
            protocol_distribution: self.protocol_counts,
        }
    }
}

/// Protocol analysis statistics
#[derive(Debug, Clone)]
pub struct ProtocolStatistics {
    pub total_frames: u32,
    pub total_protocols: u32,
    pub error_count: u32,
    pub frame_distribution: [u32; 256],
    pub protocol_distribution: [u32; 16],
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_parser_creation() {
        let parser = ProtocolParser::new();
        assert!(parser.is_ok());
    }
    
    #[test]
    fn test_ethernet_frame_parsing() {
        let parser = ProtocolParser::new().unwrap();
        
        // Create a simple Ethernet frame
        let mut frame = Vec::new();
        frame.extend_from_slice(&[0x00, 0x1A, 0x79, 0x12, 0x34, 0x56]); // Dest
        frame.extend_from_slice(&[0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC]); // Src
        frame.extend_from_slice(&[0x08, 0x00]); // Ethertype (IPv4)
        frame.extend_from_slice(&[0x45, 0x00, 0x00, 0x3C]); // IPv4 header start
        
        let result = parser.parse_ethernet_frame(&frame);
        assert!(result.is_ok());
        
        let (header, protocol_type, _) = result.unwrap();
        assert_eq!(header.source_mac, [0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC]);
        assert_eq!(protocol_type, ProtocolType::Ipv4);
    }
    
    #[test]
    fn test_wifi_frame_parsing() {
        let parser = ProtocolParser::new().unwrap();
        
        // Create a simple Wi-Fi frame header
        let mut frame = Vec::new();
        frame.extend_from_slice(&[0x00, 0x00]); // Frame control (Beacon)
        frame.extend_from_slice(&[0x00, 0x00]); // Duration
        frame.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]); // Dest
        frame.extend_from_slice(&[0x00, 0x1A, 0x79, 0x12, 0x34, 0x56]); // Src
        frame.extend_from_slice(&[0x00, 0x1A, 0x79, 0x12, 0x34, 0x56]); // BSSID
        frame.extend_from_slice(&[0x00, 0x00]); // Sequence control
        
        let result = parser.parse_wifi_frame(&frame);
        assert!(result.is_ok());
        
        let (header, frame_type, frame_subtype, _) = result.unwrap();
        assert_eq!(frame_type, FrameType::Management);
        assert_eq!(frame_subtype, FrameSubtype::Beacon);
    }
    
    #[test]
    fn test_tcp_header_parsing() {
        let parser = ProtocolParser::new().unwrap();
        
        // Create a simple TCP header
        let mut header = Vec::new();
        header.extend_from_slice(&[0x00, 0x50]); // Source port (80)
        header.extend_from_slice(&[0x00, 0x50]); // Dest port (80)
        header.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // Sequence
        header.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Ack
        header.extend_from_slice(&[0x50, 0x00]); // Header length + flags (SYN)
        header.extend_from_slice(&[0x72, 0x10]); // Window size
        header.extend_from_slice(&[0x00, 0x00]); // Checksum
        header.extend_from_slice(&[0x00, 0x00]); // Urgent pointer
        
        let result = parser.parse_tcp_header(&header);
        assert!(result.is_ok());
        
        let tcp_header = result.unwrap();
        assert_eq!(tcp_header.source_port, 80);
        assert_eq!(tcp_header.destination_port, 80);
        assert!(tcp_header.flags.syn);
    }
    
    #[test]
    fn test_arp_header_parsing() {
        let parser = ProtocolParser::new().unwrap();
        
        // Create a simple ARP request
        let mut arp = Vec::new();
        arp.extend_from_slice(&[0x00, 0x01]); // Hardware type (Ethernet)
        arp.extend_from_slice(&[0x08, 0x00]); // Protocol type (IPv4)
        arp.extend_from_slice(&[0x06]); // Hardware size
        arp.extend_from_slice(&[0x04]); // Protocol size
        arp.extend_from_slice(&[0x00, 0x01]); // Operation (Request)
        arp.extend_from_slice(&[0x00, 0x1C, 0x42, 0x78, 0x9A, 0xBC]); // Sender MAC
        arp.extend_from_slice(&[0xC0, 0xA8, 0x01, 0x64]); // Sender IP (192.168.1.100)
        arp.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]); // Target MAC (unknown)
        arp.extend_from_slice(&[0xC0, 0xA8, 0x01, 0x01]); // Target IP (192.168.1.1)
        
        let result = parser.parse_arp_header(&arp);
        assert!(result.is_ok());
        
        let arp_header = result.unwrap();
        assert_eq!(arp_header.operation, ArpOperation::Request);
        assert_eq!(arp_header.sender_protocol_address, [192, 168, 1, 100]);
        assert_eq!(arp_header.target_protocol_address, [192, 168, 1, 1]);
    }
}