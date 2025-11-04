//! Integration module for device-drivers compatibility
//!
//! This module provides seamless integration between the comprehensive driver testing framework
//! and the existing device-drivers crate testing infrastructure.

pub mod device_drivers_bridge;

pub use device_drivers_bridge::{
    DeviceDriversTestBridge,
    DeviceDriversIntegrationConfig,
    ComprehensiveTestResults,
    IntegrationStatistics,
    AdvancedTestSuiteBuilder,
};