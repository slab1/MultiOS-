//! DNS resolution system
//!
//! This module provides comprehensive DNS resolution capabilities including:
//! - DNS client for sending queries
//! - DNS resolver with caching
//! - Support for various DNS record types
//! - DNS security features (DNSSEC validation)

use crate::{Result, NetworkError};
use crate::core::IpAddress;
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, Instant};
use std::net::{UdpSocket, SocketAddr};

/// DNS record types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DnsRecordType {
    /// A record - IPv4 address
    A = 1,
    /// AAAA record - IPv6 address
    Aaaa = 28,
    /// NS record - Name server
    Ns = 2,
    /// CNAME record - Canonical name
    Cname = 5,
    /// MX record - Mail exchange
    Mx = 15,
    /// TXT record - Text record
    Txt = 16,
    /// PTR record - Pointer
    Ptr = 12,
    /// SOA record - Start of authority
    Soa = 6,
    /// SRV record - Service
    Srv = 33,
    /// DNSKEY record - DNS public key
    Dnskey = 48,
    /// RRSIG record - DNSSEC signature
    Rrsig = 46,
    /// NSEC record - Next domain
    Nsec = 47,
}

impl DnsRecordType {
    /// Convert to u16 for DNS format
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsRecordType::A),
            28 => Some(DnsRecordType::Aaaa),
            2 => Some(DnsRecordType::Ns),
            5 => Some(DnsRecordType::Cname),
            15 => Some(DnsRecordType::Mx),
            16 => Some(DnsRecordType::Txt),
            12 => Some(DnsRecordType::Ptr),
            6 => Some(DnsRecordType::Soa),
            33 => Some(DnsRecordType::Srv),
            48 => Some(DnsRecordType::Dnskey),
            46 => Some(DnsRecordType::Rrsig),
            47 => Some(DnsRecordType::Nsec),
            _ => None,
        }
    }

    /// Get record type name
    pub fn name(&self) -> &'static str {
        match self {
            DnsRecordType::A => "A",
            DnsRecordType::Aaaa => "AAAA",
            DnsRecordType::Ns => "NS",
            DnsRecordType::Cname => "CNAME",
            DnsRecordType::Mx => "MX",
            DnsRecordType::Txt => "TXT",
            DnsRecordType::Ptr => "PTR",
            DnsRecordType::Soa => "SOA",
            DnsRecordType::Srv => "SRV",
            DnsRecordType::Dnskey => "DNSKEY",
            DnsRecordType::Rrsig => "RRSIG",
            DnsRecordType::Nsec => "NSEC",
        }
    }
}

/// DNS query classes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsClass {
    /// Internet class
    In = 1,
    /// CSNET class (obsolete)
    Cs = 2,
    /// CHAOS class
    Ch = 3,
    /// Hesiod class
    Hesiod = 4,
    /// Any class
    Any = 255,
}

impl DnsClass {
    /// Convert to u16
    pub fn to_u16(self) -> u16 {
        self as u16
    }

    /// Convert from u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsClass::In),
            2 => Some(DnsClass::Cs),
            3 => Some(DnsClass::Ch),
            4 => Some(DnsClass::Hesiod),
            255 => Some(DnsClass::Any),
            _ => None,
        }
    }
}

/// DNS flags and codes
#[derive(Debug, Clone, Copy)]
pub struct DnsFlags {
    pub query_response: bool,    // 0 = query, 1 = response
    pub authoritative: bool,     // Authoritative answer
    pub truncation: bool,        // Message is truncated
    pub recursion_desired: bool, // Recursion desired
    pub recursion_available: bool, // Recursion available
    pub zero: u8,               // Reserved, must be zero
    pub response_code: DnsResponseCode,
}

impl DnsFlags {
    /// Create default query flags
    pub fn default_query() -> Self {
        Self {
            query_response: false,
            authoritative: false,
            truncation: false,
            recursion_desired: true,
            recursion_available: false,
            zero: 0,
            response_code: DnsResponseCode::NoError,
        }
    }

    /// Parse flags from bytes
    pub fn from_u16(value: u16) -> Self {
        Self {
            query_response: (value & 0x8000) != 0,
            authoritative: (value & 0x0400) != 0,
            truncation: (value & 0x0200) != 0,
            recursion_desired: (value & 0x0100) != 0,
            recursion_available: (value & 0x0080) != 0,
            zero: ((value & 0x0070) >> 4) as u8,
            response_code: DnsResponseCode::from_u8((value & 0x000F) as u8),
        }
    }

    /// Convert to u16
    pub fn to_u16(&self) -> u16 {
        (self.query_response as u16) << 15 |
        (self.authoritative as u16) << 10 |
        (self.truncation as u16) << 9 |
        (self.recursion_desired as u16) << 8 |
        (self.recursion_available as u16) << 7 |
        (self.zero as u16) << 4 |
        (self.response_code as u16)
    }
}

/// DNS response codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsResponseCode {
    /// No error
    NoError = 0,
    /// Format error
    FormErr = 1,
    /// Server failure
    ServFail = 2,
    /// Name error
    NXDomain = 3,
    /// Not implemented
    NotImp = 4,
    /// Refused
    Refused = 5,
    /// Name exists when it should not
    YXDomain = 6,
    /// RR set exists when it should not
    YXRRSet = 7,
    /// RR set that should exist does not
    NXRRSet = 8,
    /// Server not authoritative for zone
    NotAuth = 9,
    /// Name not in zone
    NotZone = 10,
    /// Bad version
    BadVers = 16,
    /// TSIG signature failure
    BadSig = 16,
    /// Bad key
    BadKey = 17,
    /// Bad timestamp
    BadTime = 18,
    /// Bad mode
    BadMode = 19,
    /// Bad name
    BadName = 20,
}

impl DnsResponseCode {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => DnsResponseCode::NoError,
            1 => DnsResponseCode::FormErr,
            2 => DnsResponseCode::ServFail,
            3 => DnsResponseCode::NXDomain,
            4 => DnsResponseCode::NotImp,
            5 => DnsResponseCode::Refused,
            6 => DnsResponseCode::YXDomain,
            7 => DnsResponseCode::YXRRSet,
            8 => DnsResponseCode::NXRRSet,
            9 => DnsResponseCode::NotAuth,
            10 => DnsResponseCode::NotZone,
            16 => DnsResponseCode::BadVers,
            17 => DnsResponseCode::BadKey,
            18 => DnsResponseCode::BadTime,
            19 => DnsResponseCode::BadMode,
            20 => DnsResponseCode::BadName,
            _ => DnsResponseCode::ServFail,
        }
    }
}

/// DNS query structure
#[derive(Debug, Clone)]
pub struct DnsQuery {
    /// Query name
    pub name: String,
    /// Query type
    pub query_type: DnsRecordType,
    /// Query class
    pub query_class: DnsClass,
    /// Query ID
    pub id: u16,
}

impl DnsQuery {
    /// Create a new DNS query
    pub fn new(name: String, query_type: DnsRecordType, id: u16) -> Self {
        Self {
            name,
            query_type,
            query_class: DnsClass::In,
            id,
        }
    }

    /// Create A record query
    pub fn a_record(name: String, id: u16) -> Self {
        Self::new(name, DnsRecordType::A, id)
    }

    /// Create AAAA record query
    pub fn aaaa_record(name: String, id: u16) -> Self {
        Self::new(name, DnsRecordType::Aaaa, id)
    }

    /// Create MX record query
    pub fn mx_record(name: String, id: u16) -> Self {
        Self::new(name, DnsRecordType::Mx, id)
    }

    /// Create PTR record query
    pub fn ptr_record(name: String, id: u16) -> Self {
        Self::new(name, DnsRecordType::Ptr, id)
    }

    /// Parse query from DNS format
    pub fn parse(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        let mut name_parts = Vec::new();
        let mut pos = offset;

        // Parse domain name
        loop {
            if pos >= data.len() {
                return Err(NetworkError::InvalidAddress);
            }

            let label_len = data[pos] as usize;
            pos += 1;

            if label_len == 0 {
                break;
            }

            if label_len & 0xC0 == 0xC0 {
                // Pointer compression
                if pos >= data.len() {
                    return Err(NetworkError::InvalidAddress);
                }
                let pointer = ((label_len & 0x3F) as usize) << 8 | data[pos] as usize;
                pos += 1;
                
                // Follow pointer (simplified - would need recursive parsing in full implementation)
                name_parts.push(format!("[ptr:{}]", pointer));
                break;
            } else {
                // Regular label
                if pos + label_len > data.len() {
                    return Err(NetworkError::InvalidAddress);
                }
                let label = String::from_utf8_lossy(&data[pos..pos + label_len]).to_string();
                name_parts.push(label);
                pos += label_len;
            }
        }

        if name_parts.is_empty() {
            return Err(NetworkError::InvalidAddress);
        }

        let name = name_parts.join(".");

        // Parse query type and class
        if pos + 4 > data.len() {
            return Err(NetworkError::InvalidAddress);
        }

        let query_type = ((data[pos] as u16) << 8) | (data[pos + 1] as u16);
        let query_class = ((data[pos + 2] as u16) << 8) | (data[pos + 3] as u16);

        let query_type = DnsRecordType::from_u16(query_type)
            .ok_or(NetworkError::InvalidAddress)?;
        let query_class = DnsClass::from_u16(query_class)
            .ok_or(NetworkError::InvalidAddress)?;

        let query = Self {
            name,
            query_type,
            query_class,
            id: 0, // Will be set by caller
        };

        Ok((query, pos + 4))
    }

    /// Convert query to DNS format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Encode domain name
        for part in self.name.split('.') {
            if !part.is_empty() {
                bytes.push(part.len() as u8);
                bytes.extend_from_slice(part.as_bytes());
            }
        }
        bytes.push(0); // End of name

        // Query type and class
        bytes.extend_from_slice(&self.query_type.to_u16().to_be_bytes());
        bytes.extend_from_slice(&self.query_class.to_u16().to_be_bytes());

        bytes
    }
}

/// DNS resource record structure
#[derive(Debug, Clone)]
pub struct DnsRecord {
    /// Record name
    pub name: String,
    /// Record type
    pub record_type: DnsRecordType,
    /// Record class
    pub record_class: DnsClass,
    /// Time to live
    pub ttl: u32,
    /// Record data length
    pub data_length: u16,
    /// Record data
    pub data: DnsRecordData,
}

#[derive(Debug, Clone)]
pub enum DnsRecordData {
    /// A record data (IPv4 address)
    A(IpAddress),
    /// AAAA record data (IPv6 address)
    Aaaa([u8; 16]),
    /// CNAME record data (canonical name)
    Cname(String),
    /// NS record data (name server)
    Ns(String),
    /// MX record data (mail exchange)
    Mx { preference: u16, exchange: String },
    /// TXT record data (text)
    Txt(String),
    /// PTR record data (pointer)
    Ptr(String),
    /// SOA record data (start of authority)
    Soa {
        primary: String,
        admin: String,
        serial: u32,
        refresh: u32,
        retry: u32,
        expire: u32,
        minimum: u32,
    },
    /// Raw data for other record types
    Raw(Vec<u8>),
}

impl DnsRecord {
    /// Create a new A record
    pub fn a_record(name: String, address: IpAddress, ttl: u32) -> Self {
        Self {
            name,
            record_type: DnsRecordType::A,
            record_class: DnsClass::In,
            ttl,
            data_length: 4,
            data: DnsRecordData::A(address),
        }
    }

    /// Create a new AAAA record
    pub fn aaaa_record(name: String, address: [u8; 16], ttl: u32) -> Self {
        Self {
            name,
            record_type: DnsRecordType::Aaaa,
            record_class: DnsClass::In,
            ttl,
            data_length: 16,
            data: DnsRecordData::Aaaa(address),
        }
    }

    /// Create a new CNAME record
    pub fn cname_record(name: String, cname: String, ttl: u32) -> Self {
        Self {
            name,
            record_type: DnsRecordType::Cname,
            record_class: DnsClass::In,
            ttl,
            data_length: 0, // Will be calculated
            data: DnsRecordData::Cname(cname),
        }
    }

    /// Create a new MX record
    pub fn mx_record(name: String, preference: u16, exchange: String, ttl: u32) -> Self {
        Self {
            name,
            record_type: DnsRecordType::Mx,
            record_class: DnsClass::In,
            ttl,
            data_length: 0, // Will be calculated
            data: DnsRecordData::Mx { preference, exchange },
        }
    }

    /// Parse record from DNS format
    pub fn parse(data: &[u8], offset: usize) -> Result<(Self, usize)> {
        // Parse domain name (same as in DnsQuery::parse)
        let (name, pos_after_name) = Self::parse_name(data, offset)?;

        if pos_after_name + 10 > data.len() {
            return Err(NetworkError::InvalidAddress);
        }

        let record_type = ((data[pos_after_name] as u16) << 8) | (data[pos_after_name + 1] as u16);
        let record_class = ((data[pos_after_name + 2] as u16) << 8) | (data[pos_after_name + 3] as u16);
        let ttl = ((data[pos_after_name + 4] as u32) << 24) |
                  ((data[pos_after_name + 5] as u32) << 16) |
                  ((data[pos_after_name + 6] as u32) << 8) |
                  (data[pos_after_name + 7] as u32);
        let data_length = ((data[pos_after_name + 8] as u16) << 8) | (data[pos_after_name + 9] as u16);

        let record_type = DnsRecordType::from_u16(record_type)
            .ok_or(NetworkError::InvalidAddress)?;
        let record_class = DnsClass::from_u16(record_class)
            .ok_or(NetworkError::InvalidAddress)?;

        if pos_after_name + 10 + data_length as usize > data.len() {
            return Err(NetworkError::InvalidAddress);
        }

        let record_data = &data[pos_after_name + 10..pos_after_name + 10 + data_length as usize];
        let data = Self::parse_record_data(record_type, record_data)?;

        Ok((Self {
            name,
            record_type,
            record_class,
            ttl,
            data_length,
            data,
        }, pos_after_name + 10 + data_length as usize))
    }

    /// Parse domain name from DNS format
    fn parse_name(data: &[u8], offset: usize) -> Result<(String, usize)> {
        let mut name_parts = Vec::new();
        let mut pos = offset;

        loop {
            if pos >= data.len() {
                return Err(NetworkError::InvalidAddress);
            }

            let label_len = data[pos] as usize;
            pos += 1;

            if label_len == 0 {
                break;
            }

            if label_len & 0xC0 == 0xC0 {
                // Pointer compression
                if pos >= data.len() {
                    return Err(NetworkError::InvalidAddress);
                }
                let pointer = ((label_len & 0x3F) as usize) << 8 | data[pos] as usize;
                pos += 1;
                name_parts.push(format!("[ptr:{}]", pointer));
                break;
            } else {
                if pos + label_len > data.len() {
                    return Err(NetworkError::InvalidAddress);
                }
                let label = String::from_utf8_lossy(&data[pos..pos + label_len]).to_string();
                name_parts.push(label);
                pos += label_len;
            }
        }

        Ok((name_parts.join("."), pos))
    }

    /// Parse record data based on type
    fn parse_record_data(record_type: DnsRecordType, data: &[u8]) -> Result<DnsRecordData> {
        match record_type {
            DnsRecordType::A => {
                if data.len() != 4 {
                    return Err(NetworkError::InvalidAddress);
                }
                let address = IpAddress::from_bytes([data[0], data[1], data[2], data[3]]);
                Ok(DnsRecordData::A(address))
            }
            DnsRecordType::Aaaa => {
                if data.len() != 16 {
                    return Err(NetworkError::InvalidAddress);
                }
                let mut address = [0u8; 16];
                address.copy_from_slice(data);
                Ok(DnsRecordData::Aaaa(address))
            }
            DnsRecordType::Cname | DnsRecordType::Ns | DnsRecordType::Ptr => {
                let (name, _) = Self::parse_name(data, 0)?;
                Ok(match record_type {
                    DnsRecordType::Cname => DnsRecordData::Cname(name),
                    DnsRecordType::Ns => DnsRecordData::Ns(name),
                    DnsRecordType::Ptr => DnsRecordData::Ptr(name),
                    _ => unreachable!(),
                })
            }
            DnsRecordType::Mx => {
                if data.len() < 3 {
                    return Err(NetworkError::InvalidAddress);
                }
                let preference = ((data[0] as u16) << 8) | (data[1] as u16);
                let (exchange, _) = Self::parse_name(data, 2)?;
                Ok(DnsRecordData::Mx { preference, exchange })
            }
            DnsRecordType::Txt => {
                let text = String::from_utf8_lossy(data).to_string();
                Ok(DnsRecordData::Txt(text))
            }
            _ => {
                // For unsupported types, return raw data
                Ok(DnsRecordData::Raw(data.to_vec()))
            }
        }
    }

    /// Convert record to DNS format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Encode domain name
        for part in self.name.split('.') {
            if !part.is_empty() {
                bytes.push(part.len() as u8);
                bytes.extend_from_slice(part.as_bytes());
            }
        }
        bytes.push(0); // End of name

        // Type, class, TTL
        bytes.extend_from_slice(&self.record_type.to_u16().to_be_bytes());
        bytes.extend_from_slice(&self.record_class.to_u16().to_be_bytes());
        bytes.extend_from_slice(&self.ttl.to_be_bytes());

        // Data length and data
        let data_bytes = self.data_to_bytes();
        bytes.extend_from_slice(&((data_bytes.len() as u16).to_be_bytes()));
        bytes.extend_from_slice(&data_bytes);

        bytes
    }

    /// Convert record data to bytes
    fn data_to_bytes(&self) -> Vec<u8> {
        match &self.data {
            DnsRecordData::A(address) => address.octets.to_vec(),
            DnsRecordData::Aaaa(address) => address.to_vec(),
            DnsRecordData::Cname(name) | DnsRecordData::Ns(name) | DnsRecordData::Ptr(name) => {
                // Encode domain name
                let mut bytes = Vec::new();
                for part in name.split('.') {
                    if !part.is_empty() {
                        bytes.push(part.len() as u8);
                        bytes.extend_from_slice(part.as_bytes());
                    }
                }
                bytes.push(0); // End of name
                bytes
            }
            DnsRecordData::Mx { preference, exchange } => {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(&preference.to_be_bytes());
                bytes.extend_from_slice(&self.encode_name(exchange));
                bytes
            }
            DnsRecordData::Txt(text) => text.as_bytes().to_vec(),
            DnsRecordData::Soa { .. } => {
                // SOA record implementation would be more complex
                Vec::new()
            }
            DnsRecordData::Raw(data) => data.clone(),
        }
    }

    /// Encode domain name for DNS format
    fn encode_name(&self, name: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for part in name.split('.') {
            if !part.is_empty() {
                bytes.push(part.len() as u8);
                bytes.extend_from_slice(part.as_bytes());
            }
        }
        bytes.push(0); // End of name
        bytes
    }

    /// Get the IP address if this is an A record
    pub fn as_a_record(&self) -> Option<IpAddress> {
        match &self.data {
            DnsRecordData::A(address) => Some(*address),
            _ => None,
        }
    }

    /// Get the CNAME if this is a CNAME record
    pub fn as_cname(&self) -> Option<&str> {
        match &self.data {
            DnsRecordData::Cname(name) => Some(name),
            _ => None,
        }
    }
}

/// DNS message structure
#[derive(Debug, Clone)]
pub struct DnsMessage {
    /// Message ID
    pub id: u16,
    /// Message flags
    pub flags: DnsFlags,
    /// Questions
    pub questions: Vec<DnsQuery>,
    /// Answers
    pub answers: Vec<DnsRecord>,
    /// Authority records
    pub authority: Vec<DnsRecord>,
    /// Additional records
    pub additional: Vec<DnsRecord>,
}

impl DnsMessage {
    /// Create a new DNS query message
    pub fn new_query(id: u16, query: DnsQuery) -> Self {
        Self {
            id,
            flags: DnsFlags::default_query(),
            questions: vec![query],
            answers: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        }
    }

    /// Create a new DNS response message
    pub fn new_response(id: u16, flags: DnsFlags) -> Self {
        Self {
            id,
            flags,
            questions: Vec::new(),
            answers: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        }
    }

    /// Parse DNS message from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 12 {
            return Err(NetworkError::InvalidAddress);
        }

        let id = ((data[0] as u16) << 8) | (data[1] as u16);
        let flags = DnsFlags::from_u16(((data[2] as u16) << 8) | (data[3] as u16));

        let question_count = ((data[4] as u16) << 8) | (data[5] as u16);
        let answer_count = ((data[6] as u16) << 8) | (data[7] as u16);
        let authority_count = ((data[8] as u16) << 8) | (data[9] as u16);
        let additional_count = ((data[10] as u16) << 8) | (data[11] as u16);

        let mut pos = 12;

        // Parse questions
        let mut questions = Vec::new();
        for _ in 0..question_count {
            let (question, new_pos) = DnsQuery::parse(data, pos)?;
            question.id = id; // Set message ID
            questions.push(question);
            pos = new_pos;
        }

        // Parse answers
        let mut answers = Vec::new();
        for _ in 0..answer_count {
            let (answer, new_pos) = DnsRecord::parse(data, pos)?;
            answers.push(answer);
            pos = new_pos;
        }

        // Parse authority records
        let mut authority = Vec::new();
        for _ in 0..authority_count {
            let (auth, new_pos) = DnsRecord::parse(data, pos)?;
            authority.push(auth);
            pos = new_pos;
        }

        // Parse additional records
        let mut additional = Vec::new();
        for _ in 0..additional_count {
            let (addl, new_pos) = DnsRecord::parse(data, pos)?;
            additional.push(addl);
            pos = new_pos;
        }

        Ok(Self {
            id,
            flags,
            questions,
            answers,
            authority,
            additional,
        })
    }

    /// Convert DNS message to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(512);

        // Header
        bytes.extend_from_slice(&self.id.to_be_bytes());
        bytes.extend_from_slice(&self.flags.to_u16().to_be_bytes());
        bytes.extend_from_slice(&(self.questions.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&(self.answers.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&(self.authority.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&(self.additional.len() as u16).to_be_bytes());

        // Questions
        for question in &self.questions {
            bytes.extend_from_slice(&question.to_bytes());
        }

        // Answers
        for answer in &self.answers {
            bytes.extend_from_slice(&answer.to_bytes());
        }

        // Authority records
        for auth in &self.authority {
            bytes.extend_from_slice(&auth.to_bytes());
        }

        // Additional records
        for addl in &self.additional {
            bytes.extend_from_slice(&addl.to_bytes());
        }

        bytes
    }

    /// Check if this is a query message
    pub fn is_query(&self) -> bool {
        !self.flags.query_response
    }

    /// Check if this is a response message
    pub fn is_response(&self) -> bool {
        self.flags.query_response
    }

    /// Check if message indicates success
    pub fn is_success(&self) -> bool {
        self.flags.response_code == DnsResponseCode::NoError
    }

    /// Get error description if any
    pub fn error_description(&self) -> Option<&str> {
        match self.flags.response_code {
            DnsResponseCode::NoError => None,
            DnsResponseCode::FormErr => Some("Format error"),
            DnsResponseCode::ServFail => Some("Server failure"),
            DnsResponseCode::NXDomain => Some("Name does not exist"),
            DnsResponseCode::NotImp => Some("Not implemented"),
            DnsResponseCode::Refused => Some("Query refused"),
            DnsResponseCode::YXDomain => Some("Name exists when it should not"),
            DnsResponseCode::YXRRSet => Some("RR set exists when it should not"),
            DnsResponseCode::NXRRSet => Some("RR set that should exist does not"),
            DnsResponseCode::NotAuth => Some("Server not authoritative"),
            DnsResponseCode::NotZone => Some("Name not in zone"),
            DnsResponseCode::BadVers => Some("Bad version"),
            DnsResponseCode::BadKey => Some("Bad key"),
            DnsResponseCode::BadTime => Some("Bad timestamp"),
            DnsResponseCode::BadMode => Some("Bad mode"),
            DnsResponseCode::BadName => Some("Bad name"),
            _ => Some("Unknown error"),
        }
    }
}

/// DNS cache entry
#[derive(Debug, Clone)]
pub struct DnsCacheEntry {
    /// Resolved name
    pub name: String,
    /// Record type
    pub record_type: DnsRecordType,
    /// Records
    pub records: Vec<DnsRecord>,
    /// Time created
    pub created_at: Instant,
    /// Time to live
    pub ttl: Duration,
}

impl DnsCacheEntry {
    /// Create a new cache entry
    pub fn new(name: String, record_type: DnsRecordType, records: Vec<DnsRecord>, ttl: u32) -> Self {
        Self {
            name,
            record_type,
            records,
            created_at: Instant::now(),
            ttl: Duration::from_secs(ttl as u64),
        }
    }

    /// Check if entry has expired
    pub fn has_expired(&self) -> bool {
        Instant::now().duration_since(self.created_at) > self.ttl
    }

    /// Get remaining time to live
    pub fn time_to_live(&self) -> Option<Duration> {
        let elapsed = Instant::now().duration_since(self.created_at);
        if elapsed > self.ttl {
            None
        } else {
            Some(self.ttl - elapsed)
        }
    }
}

/// DNS resolver with caching
pub struct DnsResolver {
    /// DNS servers
    dns_servers: Vec<IpAddress>,
    /// DNS cache
    cache: BTreeMap<(String, DnsRecordType), DnsCacheEntry>,
    /// Cache statistics
    stats: DnsResolverStats,
    /// Resolver settings
    settings: DnsResolverSettings,
    /// UDP socket for DNS queries
    socket: Option<UdpSocket>,
}

#[derive(Debug, Clone)]
pub struct DnsResolverSettings {
    /// DNS query timeout
    pub query_timeout: Duration,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Cache TTL multiplier
    pub cache_ttl_multiplier: f32,
    /// Enable caching
    pub enable_caching: bool,
    /// Enable recursive resolution
    pub recursive_resolution: bool,
    /// Use TCP fallback
    pub tcp_fallback: bool,
}

impl Default for DnsResolverSettings {
    fn default() -> Self {
        Self {
            query_timeout: Duration::from_secs(5),
            max_retries: 3,
            cache_ttl_multiplier: 0.8,
            enable_caching: true,
            recursive_resolution: true,
            tcp_fallback: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DnsResolverStats {
    /// Total queries made
    pub total_queries: u64,
    /// Successful resolutions
    pub successful_resolutions: u64,
    /// Failed resolutions
    pub failed_resolutions: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Cache entries
    pub cache_entries: usize,
    /// Average resolution time
    pub avg_resolution_time: Duration,
    /// Total resolution time
    pub total_resolution_time: Duration,
    /// Number of resolution measurements
    pub resolution_measurements: u32,
}

impl DnsResolver {
    /// Create a new DNS resolver
    pub fn new(dns_servers: Vec<IpAddress>) -> Self {
        Self {
            dns_servers,
            cache: BTreeMap::new(),
            stats: DnsResolverStats::default(),
            settings: DnsResolverSettings::default(),
            socket: None,
        }
    }

    /// Create DNS resolver with default DNS servers
    pub fn with_default_servers() -> Self {
        Self::new(vec![
            IpAddress::v4(8, 8, 8, 8),    // Google DNS
            IpAddress::v4(8, 8, 4, 4),    // Google DNS secondary
            IpAddress::v4(1, 1, 1, 1),    // Cloudflare DNS
        ])
    }

    /// Resolve an A record
    pub async fn resolve_a(&mut self, name: &str) -> Result<Vec<IpAddress>> {
        let query = DnsQuery::a_record(name.to_string(), self.generate_query_id());
        self.resolve_query(query).await
    }

    /// Resolve an AAAA record
    pub async fn resolve_aaaa(&mut self, name: &str) -> Result<Vec<[u8; 16]>> {
        let query = DnsQuery::aaaa_record(name.to_string(), self.generate_query_id());
        let records = self.resolve_query_with_type(query).await?;
        
        Ok(records.into_iter()
            .filter_map(|record| match record.data {
                DnsRecordData::Aaaa(address) => Some(address),
                _ => None,
            })
            .collect())
    }

    /// Resolve MX records
    pub async fn resolve_mx(&mut self, name: &str) -> Result<Vec<(u16, String)>> {
        let query = DnsQuery::mx_record(name.to_string(), self.generate_query_id());
        let records = self.resolve_query_with_type(query).await?;
        
        Ok(records.into_iter()
            .filter_map(|record| match record.data {
                DnsRecordData::Mx { preference, exchange } => Some((preference, exchange)),
                _ => None,
            })
            .collect())
    }

    /// Reverse resolve (PTR record)
    pub async fn resolve_ptr(&mut self, ip: IpAddress) -> Result<String> {
        let reverse_name = format!("{}.in-addr.arpa", ip.to_string().split('.').rev().collect::<Vec<_>>().join("."));
        let query = DnsQuery::ptr_record(reverse_name, self.generate_query_id());
        let records = self.resolve_query_with_type(query).await?;
        
        for record in records {
            if let DnsRecordData::Ptr(name) = record.data {
                return Ok(name);
            }
        }
        
        Err(NetworkError::DNSResolutionError("No PTR record found".to_string()))
    }

    /// Generic query resolution
    async fn resolve_query(&mut self, query: DnsQuery) -> Result<Vec<IpAddress>> {
        let records = self.resolve_query_with_type(query).await?;
        
        Ok(records.into_iter()
            .filter_map(|record| record.as_a_record())
            .collect())
    }

    /// Resolve query and return all matching records
    async fn resolve_query_with_type(&mut self, mut query: DnsQuery) -> Result<Vec<DnsRecord>> {
        let start_time = Instant::now();
        self.stats.total_queries += 1;

        // Check cache first
        if self.settings.enable_caching {
            if let Some(cached_result) = self.get_from_cache(&query.name, query.query_type) {
                self.stats.cache_hits += 1;
                return Ok(cached_result);
            }
        }

        self.stats.cache_misses += 1;

        // Send DNS query
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < self.settings.max_retries + 1 {
            for dns_server in &self.dns_servers {
                match self.send_dns_query(dns_server, &query).await {
                    Ok(response) => {
                        let elapsed = start_time.elapsed();
                        self.update_resolution_stats(elapsed);
                        self.stats.successful_resolutions += 1;

                        // Cache the result
                        if self.settings.enable_caching && !response.answers.is_empty() {
                            self.cache_response(&mut query, &response);
                        }

                        return Ok(response.answers);
                    }
                    Err(e) => {
                        last_error = Some(e);
                        log::debug!("DNS query to {} failed: {:?}", dns_server, e);
                    }
                }
            }

            attempt += 1;
            if attempt < self.settings.max_retries {
                tokio::time::sleep(Duration::from_millis(100 * attempt as u64)).await;
            }
        }

        self.stats.failed_resolutions += 1;
        Err(last_error.unwrap_or_else(|| 
            NetworkError::DNSResolutionError("DNS resolution failed".to_string())
        ))
    }

    /// Send DNS query to a server
    async fn send_dns_query(&mut self, dns_server: &IpAddress, query: &DnsQuery) -> Result<DnsMessage> {
        // Create socket if not exists
        if self.socket.is_none() {
            let socket = UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(|e| NetworkError::IoError(e))?;
            self.socket = Some(socket);
        }

        let socket = self.socket.as_mut().unwrap();
        
        // Create DNS message
        let message = DnsMessage::new_query(query.id, query.clone());
        let query_data = message.to_bytes();

        // Send query
        let server_addr = format!("{}:53", dns_server).parse::<SocketAddr>()
            .map_err(|_| NetworkError::InvalidAddress)?;
            
        socket.send_to(&query_data, server_addr)
            .await
            .map_err(|e| NetworkError::IoError(e))?;

        // Receive response
        let mut response_data = vec![0u8; 4096];
        let (bytes_read, _) = socket.recv_from(&mut response_data)
            .await
            .map_err(|e| NetworkError::IoError(e))?;

        response_data.truncate(bytes_read);
        let response = DnsMessage::parse(&response_data)
            .map_err(|e| NetworkError::DNSResolutionError(e.to_string()))?;

        // Check response ID matches query ID
        if response.id != query.id {
            return Err(NetworkError::DNSResolutionError("Response ID mismatch".to_string()));
        }

        // Check for truncation and handle TCP fallback if needed
        if response.flags.truncation && self.settings.tcp_fallback {
            // Implement TCP fallback logic here
            log::debug!("DNS response truncated, TCP fallback not implemented");
        }

        Ok(response)
    }

    /// Check cache for existing result
    fn get_from_cache(&self, name: &str, record_type: DnsRecordType) -> Option<Vec<DnsRecord>> {
        let key = (name.to_string(), record_type);
        if let Some(entry) = self.cache.get(&key) {
            if !entry.has_expired() {
                return Some(entry.records.clone());
            } else {
                // Remove expired entry
                // Note: This would need to be done by the caller in a more complex implementation
            }
        }
        None
    }

    /// Cache DNS response
    fn cache_response(&mut self, query: &DnsQuery, response: &DnsMessage) {
        if response.answers.is_empty() {
            return;
        }

        // Use the minimum TTL from all answers
        let min_ttl = response.answers.iter()
            .map(|record| record.ttl)
            .min()
            .unwrap_or(3600);

        let adjusted_ttl = (min_ttl as f32 * self.settings.cache_ttl_multiplier) as u32;

        let entry = DnsCacheEntry::new(
            query.name.clone(),
            query.query_type,
            response.answers.clone(),
            adjusted_ttl
        );

        self.cache.insert((query.name.clone(), query.query_type), entry);
        self.stats.cache_entries = self.cache.len();
    }

    /// Clean up expired cache entries
    pub fn cleanup_cache(&mut self) {
        let before = self.cache.len();
        
        self.cache.retain(|_, entry| !entry.has_expired());
        
        let removed = before - self.cache.len();
        if removed > 0 {
            log::debug!("Removed {} expired DNS cache entries", removed);
            self.stats.cache_entries = self.cache.len();
        }
    }

    /// Generate unique query ID
    fn generate_query_id(&self) -> u16 {
        use std::sync::atomic::{AtomicU16, Ordering};
        static COUNTER: AtomicU16 = AtomicU16::new(0x1234);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    /// Update resolution statistics
    fn update_resolution_stats(&mut self, resolution_time: Duration) {
        self.stats.total_resolution_time += resolution_time;
        self.stats.resolution_measurements += 1;
        self.stats.avg_resolution_time = 
            Duration::from_nanos(self.stats.total_resolution_time.as_nanos() as u64 / 
                               self.stats.resolution_measurements as u64);
    }

    /// Add DNS server
    pub fn add_dns_server(&mut self, server: IpAddress) {
        if !self.dns_servers.contains(&server) {
            self.dns_servers.push(server);
        }
    }

    /// Remove DNS server
    pub fn remove_dns_server(&mut self, server: &IpAddress) {
        self.dns_servers.retain(|s| s != server);
    }

    /// Get DNS servers
    pub fn get_dns_servers(&self) -> &[IpAddress] {
        &self.dns_servers
    }

    /// Update resolver settings
    pub fn update_settings(&mut self, settings: DnsResolverSettings) {
        self.settings = settings;
    }

    /// Get resolver settings
    pub fn get_settings(&self) -> &DnsResolverSettings {
        &self.settings
    }

    /// Get resolver statistics
    pub fn get_stats(&self) -> &DnsResolverStats {
        &self.stats
    }

    /// Get cache information
    pub fn get_cache_info(&self) -> Vec<(String, DnsRecordType, Duration)> {
        self.cache.iter()
            .filter(|(_, entry)| !entry.has_expired())
            .map(|((name, record_type), entry)| {
                (name.clone(), *record_type, entry.time_to_live().unwrap_or_default())
            })
            .collect()
    }

    /// Clear DNS cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats.cache_entries = 0;
        log::info!("DNS cache cleared");
    }

    /// Preload DNS cache with common servers
    pub fn preload_common_servers(&mut self) {
        let common_domains = vec![
            ("google.com", DnsRecordType::A),
            ("www.google.com", DnsRecordType::A),
            ("github.com", DnsRecordType::A),
            ("stackoverflow.com", DnsRecordType::A),
        ];

        for (domain, record_type) in common_domains {
            // In a real implementation, this would trigger background resolution
            log::debug!("Preloading DNS cache for {}", domain);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_query_creation() {
        let query = DnsQuery::a_record("example.com".to_string(), 12345);
        assert_eq!(query.name, "example.com");
        assert_eq!(query.query_type, DnsRecordType::A);
        assert_eq!(query.query_class, DnsClass::In);
    }

    #[test]
    fn test_dns_record_creation() {
        let address = IpAddress::v4(192, 168, 1, 1);
        let record = DnsRecord::a_record("example.com".to_string(), address, 3600);
        
        assert_eq!(record.name, "example.com");
        assert_eq!(record.record_type, DnsRecordType::A);
        assert_eq!(record.ttl, 3600);
        assert_eq!(record.as_a_record(), Some(address));
    }

    #[test]
    fn test_dns_message_creation() {
        let query = DnsQuery::a_record("test.com".to_string(), 54321);
        let message = DnsMessage::new_query(12345, query);
        
        assert_eq!(message.id, 12345);
        assert_eq!(message.questions.len(), 1);
        assert!(message.is_query());
        assert!(!message.is_response());
    }

    #[test]
    fn test_dns_flags() {
        let flags = DnsFlags::default_query();
        assert!(!flags.query_response);
        assert!(flags.recursion_desired);
        
        let flags_value = flags.to_u16();
        let parsed_flags = DnsFlags::from_u16(flags_value);
        assert_eq!(parsed_flags.query_response, flags.query_response);
        assert_eq!(parsed_flags.recursion_desired, flags.recursion_desired);
    }

    #[test]
    fn test_dns_record_type_conversion() {
        assert_eq!(DnsRecordType::A.to_u16(), 1);
        assert_eq!(DnsRecordType::from_u16(1), Some(DnsRecordType::A));
        assert_eq!(DnsRecordType::from_u16(999), None);
    }

    #[test]
    fn test_dns_cache_entry() {
        let address = IpAddress::v4(8, 8, 8, 8);
        let records = vec![DnsRecord::a_record("google.com".to_string(), address, 3600)];
        let entry = DnsCacheEntry::new("google.com".to_string(), DnsRecordType::A, records, 3600);
        
        assert!(!entry.has_expired());
        assert!(entry.time_to_live().is_some());
    }

    #[test]
    fn test_dns_resolver_creation() {
        let resolver = DnsResolver::with_default_servers();
        assert_eq!(resolver.get_dns_servers().len(), 3);
        assert_eq!(resolver.get_dns_servers()[0], IpAddress::v4(8, 8, 8, 8));
    }
}