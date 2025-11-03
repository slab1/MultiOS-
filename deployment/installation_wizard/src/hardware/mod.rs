pub mod error;

use crate::hardware::error::HardwareError;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Hardware detection system for MultiOS installation
pub struct HardwareDetector;

impl HardwareDetector {
    /// Detect all hardware components
    pub async fn detect_all() -> Result<HardwareInfo, HardwareError> {
        log::info!("Starting comprehensive hardware detection");
        
        let cpu_info = Self::detect_cpu().await?;
        let memory_info = Self::detect_memory().await?;
        let storage_info = Self::detect_storage().await?;
        let network_info = Self::detect_network().await?;
        let graphics_info = Self::detect_graphics().await?;
        let boot_info = Self::detect_boot_system().await?;
        let audio_info = Self::detect_audio().await?;
        let input_info = Self::detect_input_devices().await?;
        
        let hardware_info = HardwareInfo {
            cpu: cpu_info,
            memory: memory_info,
            storage: storage_info,
            network: network_info,
            graphics: graphics_info,
            boot: boot_info,
            audio: audio_info,
            input: input_info,
            timestamp: chrono::Utc::now(),
            detection_version: "1.0".to_string(),
        };
        
        log::info!("Hardware detection completed successfully");
        Ok(hardware_info)
    }

    /// Detect CPU information
    async fn detect_cpu() -> Result<CpuInfo, HardwareError> {
        log::info!("Detecting CPU information");
        
        let architecture = std::env::consts::ARCH.to_string();
        let vendor = Self::detect_cpu_vendor().unwrap_or_else(|| "Unknown".to_string());
        let model = Self::detect_cpu_model().unwrap_or_else(|| "Unknown CPU".to_string());
        let core_count = Self::detect_cpu_cores().unwrap_or(1);
        let thread_count = Self::detect_cpu_threads().unwrap_or(core_count);
        let frequency = Self::detect_cpu_frequency().unwrap_or(0);
        let features = Self::detect_cpu_features().unwrap_or_default();
        let cache_sizes = Self::detect_cpu_cache_sizes().unwrap_or_default();
        let power_management = Self::detect_cpu_power_management().unwrap_or_default();
        
        Ok(CpuInfo {
            architecture,
            vendor,
            model,
            core_count,
            thread_count,
            frequency_mhz: frequency,
            features,
            cache_sizes,
            power_management,
            temperature: None, // Would need additional sensors
            utilization: None, // Would need monitoring
        })
    }

    /// Detect CPU vendor
    fn detect_cpu_vendor() -> Option<String> {
        // Read CPU info from /proc/cpuinfo on Linux
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("vendor_id") {
                        return line.split(':').nth(1).map(|s| s.trim().to_string());
                    }
                }
            }
        }
        
        // On other platforms, return based on architecture
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("sse4_1") {
                return Some("Intel".to_string());
            } else if is_x86_feature_detected!("sse2") {
                return Some("AMD".to_string());
            }
        }
        
        None
    }

    /// Detect CPU model name
    fn detect_cpu_model() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("model name") {
                        return line.split(':').nth(1).map(|s| s.trim().to_string());
                    }
                }
            }
        }
        None
    }

    /// Detect CPU core count
    fn detect_cpu_cores() -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                let mut core_count = 0;
                for line in content.lines() {
                    if line.starts_with("processor") {
                        core_count += 1;
                    }
                }
                if core_count > 0 {
                    return Some(core_count);
                }
            }
        }
        
        // Fallback using num_cpus crate
        Some(num_cpus::get())
    }

    /// Detect CPU thread count
    fn detect_cpu_threads() -> Option<usize> {
        // Most modern CPUs have 2 threads per core (SMT/Hyper-threading)
        Self::detect_cpu_cores().map(|cores| cores * 2)
    }

    /// Detect CPU frequency
    fn detect_cpu_frequency() -> Option<u32> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("cpu MHz") {
                        if let Some(freq_str) = line.split(':').nth(1) {
                            return freq_str.trim().split('.').next()
                                .and_then(|s| s.parse::<u32>().ok());
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect CPU features/flags
    fn detect_cpu_features() -> Option<Vec<String>> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
                for line in content.lines() {
                    if line.starts_with("flags") {
                        return line.split(':').nth(1)
                            .map(|s| s.trim().split_whitespace().map(|f| f.to_string()).collect());
                    }
                }
            }
        }
        None
    }

    /// Detect CPU cache sizes
    fn detect_cpu_cache_sizes() -> Option<HashMap<String, u32>> {
        let mut cache_sizes = HashMap::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cache/index0/size") {
                if let Ok(size_str) = content.trim().parse::<String>() {
                    // Parse size like "32K"
                    if let Some(size_kb) = size_str.strip_suffix('K').and_then(|s| s.parse::<u32>().ok()) {
                        cache_sizes.insert("L1".to_string(), size_kb);
                    }
                }
            }
        }
        
        if cache_sizes.is_empty() {
            None
        } else {
            Some(cache_sizes)
        }
    }

    /// Detect CPU power management features
    fn detect_cpu_power_management() -> Option<Vec<String>> {
        let mut features = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Check for various power management features
            let power_features = [
                "/sys/devices/system/cpu/cpu0/cpufreq",
                "/sys/devices/system/cpu/cpufreq",
                "/sys/module/intel_pstate",
                "/sys/module/amd_pstate",
            ];
            
            for feature in power_features {
                if Path::new(feature).exists() {
                    features.push(Path::new(feature).file_name().unwrap().to_string_lossy().to_string());
                }
            }
        }
        
        if features.is_empty() {
            None
        } else {
            Some(features)
        }
    }

    /// Detect memory information
    async fn detect_memory() -> Result<MemoryInfo, HardwareError> {
        log::info!("Detecting memory information");
        
        let total_bytes = Self::detect_total_memory();
        let available_bytes = Self::detect_available_memory().unwrap_or(total_bytes);
        let module_count = Self::detect_memory_modules().unwrap_or(1);
        let modules = Self::detect_memory_module_details().unwrap_or_default();
        let speed = Self::detect_memory_speed().unwrap_or(0);
        let memory_type = Self::detect_memory_type().unwrap_or_else(|| "Unknown".to_string());
        let ecc_enabled = Self::detect_ecc_support().unwrap_or(false);
        
        Ok(MemoryInfo {
            total_bytes,
            available_bytes,
            module_count,
            modules,
            speed_mhz: speed,
            memory_type,
            ecc_enabled,
        })
    }

    /// Detect total system memory
    fn detect_total_memory() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemTotal") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback using sysinfo crate
        use sysinfo::{System, SystemExt};
        let mut sys = System::new_all();
        sys.refresh_all();
        sys.total_memory()
    }

    /// Detect available system memory
    fn detect_available_memory() -> Option<u64> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                for line in content.lines() {
                    if line.starts_with("MemAvailable") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return Some(kb * 1024);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Detect number of memory modules
    fn detect_memory_modules() -> Option<usize> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/devices/system/memory") {
                let mut count = 0;
                for entry in entries {
                    if let Ok(entry) = entry {
                        if entry.file_name().to_string_lossy().starts_with("memory") {
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    return Some(count);
                }
            }
        }
        None
    }

    /// Detect memory module details
    fn detect_memory_module_details() -> Option<Vec<MemoryModule>> {
        #[cfg(target_os = "linux")]
        {
            let mut modules = Vec::new();
            
            if let Ok(entries) = std::fs::read_dir("/sys/devices/system/memory") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.to_string_lossy().contains("memory") {
                            let size_path = path.join("size");
                            let state_path = path.join("state");
                            
                            if let (Ok(size_str), Ok(state_str)) = 
                                (std::fs::read_to_string(size_path), std::fs::read_to_string(state_path)) {
                                
                                if let Ok(size_kb) = size_str.trim().parse::<u64>() {
                                    modules.push(MemoryModule {
                                        size_mb: size_kb / 1024,
                                        state: state_str.trim().to_string(),
                                        index: entry.file_name().to_string_lossy().to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            
            if !modules.is_empty() {
                return Some(modules);
            }
        }
        None
    }

    /// Detect memory speed
    fn detect_memory_speed() -> Option<u32> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/devices/system/memory/memory0/size") {
                // This is a simplified detection - real implementation would be more complex
                return Some(3200); // Typical DDR4 speed
            }
        }
        None
    }

    /// Detect memory type
    fn detect_memory_type() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/devices/system/memory/memory0/size") {
                // Simplified detection - real implementation would check DMI/SMBIOS data
                return Some("DDR4".to_string());
            }
        }
        None
    }

    /// Detect ECC support
    fn detect_ecc_support() -> Option<bool> {
        #[cfg(target_os = "linux")]
        {
            // Check for ECC support in memory
            // This would require more sophisticated detection in reality
            return Some(false); // Most consumer systems don't have ECC
        }
        None
    }

    /// Detect storage devices
    async fn detect_storage() -> Result<StorageInfo, HardwareError> {
        log::info!("Detecting storage devices");
        
        let mut devices = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/block") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let device_name = entry.file_name().to_string_lossy().to_string();
                        
                        if Self::is_storage_device(&device_name) {
                            let device_info = Self::detect_storage_device(&device_name).await?;
                            devices.push(device_info);
                        }
                    }
                }
            }
        }
        
        Ok(StorageInfo {
            devices,
            total_capacity: devices.iter().map(|d| d.capacity).sum(),
            available_capacity: devices.iter().map(|d| d.available).sum(),
        })
    }

    /// Check if a device is a storage device
    fn is_storage_device(device_name: &str) -> bool {
        // Filter out non-storage devices
        let excluded = ["loop", "ram", "sr", "st"];
        !excluded.iter().any(|&prefix| device_name.starts_with(prefix))
    }

    /// Detect individual storage device information
    async fn detect_storage_device(device_name: &str) -> Result<StorageDevice, HardwareError> {
        let device_path = format!("/sys/block/{}", device_name);
        
        let device_type = Self::detect_storage_type(&device_path).unwrap_or_else(|| "Unknown".to_string());
        let capacity = Self::detect_storage_capacity(&device_path).unwrap_or(0);
        let available = Self::detect_storage_available(&device_path).unwrap_or(capacity);
        let interface = Self::detect_storage_interface(&device_path).unwrap_or_else(|| "Unknown".to_string());
        let model = Self::detect_storage_model(&device_path).unwrap_or_else(|| "Unknown Device".to_string());
        let serial = Self::detect_storage_serial(&device_path);
        let firmware = Self::detect_storage_firmware(&device_path);
        let rotational = Self::detect_storage_is_rotational(&device_path).unwrap_or(true);
        let removable = Self::detect_storage_is_removable(&device_path).unwrap_or(false);
        
        Ok(StorageDevice {
            device_name: device_name.to_string(),
            device_type,
            capacity,
            available,
            interface,
            model,
            serial,
            firmware,
            is_rotational: rotational,
            is_removable: removable,
            partitions: Vec::new(), // Would be populated by partition detection
        })
    }

    /// Detect storage device type
    fn detect_storage_type(device_path: &str) -> Option<String> {
        let size_path = format!("{}/size", device_path);
        if let Ok(size_str) = std::fs::read_to_string(size_path) {
            if let Ok(sectors) = size_str.trim().parse::<u64>() {
                // Estimate device type based on size
                let size_gb = sectors / (1024 * 1024 * 2); // Assuming 512-byte sectors
                if size_gb < 1000 {
                    Some("SSD".to_string())
                } else {
                    Some("HDD".to_string())
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Detect storage device capacity
    fn detect_storage_capacity(device_path: &str) -> Option<u64> {
        let size_path = format!("{}/size", device_path);
        if let Ok(size_str) = std::fs::read_to_string(size_path) {
            if let Ok(sectors) = size_str.trim().parse::<u64>() {
                // Assuming 512-byte sectors
                return Some(sectors * 512);
            }
        }
        None
    }

    /// Detect storage device available space
    fn detect_storage_available(device_path: &str) -> Option<u64> {
        // This would require additional system calls to get available space
        // For now, return the total capacity
        Self::detect_storage_capacity(device_path)
    }

    /// Detect storage device interface
    fn detect_storage_interface(device_path: &str) -> Option<String> {
        // Check for various interface indicators
        let queue_path = format!("{}/queue", device_path);
        
        if Path::new(&queue_path).join("rotational").exists() {
            return Some("SATA".to_string());
        } else if Path::new(&queue_path).join("nr_requests").exists() {
            return Some("NVMe".to_string());
        }
        
        Some("Unknown".to_string())
    }

    /// Detect storage device model
    fn detect_storage_model(device_path: &str) -> Option<String> {
        let device_name = device_path.split('/').last().unwrap_or("unknown");
        
        #[cfg(target_os = "linux")]
        {
            // Try to read model from udev
            let model_path = format!("/sys/class/block/{}/device/model", device_name);
            if let Ok(model) = std::fs::read_to_string(model_path) {
                return Some(model.trim().to_string());
            }
        }
        
        None
    }

    /// Detect storage device serial number
    fn detect_storage_serial(device_path: &str) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            let device_name = device_path.split('/').last().unwrap_or("unknown");
            let serial_path = format!("/sys/class/block/{}/device/serial", device_name);
            if let Ok(serial) = std::fs::read_to_string(serial_path) {
                return Some(serial.trim().to_string());
            }
        }
        None
    }

    /// Detect storage device firmware version
    fn detect_storage_firmware(device_path: &str) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            let device_name = device_path.split('/').last().unwrap_or("unknown");
            let firmware_path = format!("/sys/class/block/{}/device/firmware_rev", device_name);
            if let Ok(firmware) = std::fs::read_to_string(firmware_path) {
                return Some(firmware.trim().to_string());
            }
        }
        None
    }

    /// Check if storage device is rotational (HDD vs SSD)
    fn detect_storage_is_rotational(device_path: &str) -> Option<bool> {
        let rotational_path = format!("{}/queue/rotational", device_path);
        if let Ok(rotational_str) = std::fs::read_to_string(rotational_path) {
            if let Ok(rotational) = rotational_str.trim().parse::<u8>() {
                return Some(rotational == 1);
            }
        }
        None
    }

    /// Check if storage device is removable
    fn detect_storage_is_removable(device_path: &str) -> Option<bool> {
        let removable_path = format!("{}/removable", device_path);
        if let Ok(removable_str) = std::fs::read_to_string(removable_path) {
            if let Ok(removable) = removable_str.trim().parse::<u8>() {
                return Some(removable == 1);
            }
        }
        None
    }

    /// Detect network devices
    async fn detect_network() -> Result<NetworkInfo, HardwareError> {
        log::info!("Detecting network devices");
        
        let mut devices = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let interface_name = entry.file_name().to_string_lossy().to_string();
                        
                        // Skip virtual interfaces
                        if !["lo", "docker", "veth", "br-"].iter().any(|&prefix| interface_name.starts_with(prefix)) {
                            let device_info = Self::detect_network_interface(&interface_name).await?;
                            devices.push(device_info);
                        }
                    }
                }
            }
        }
        
        Ok(NetworkInfo {
            devices,
        })
    }

    /// Detect individual network interface information
    async fn detect_network_interface(interface_name: &str) -> Result<NetworkInterface, HardwareError> {
        let interface_path = format!("/sys/class/net/{}", interface_name);
        
        let device_type = Self::detect_network_device_type(&interface_path).unwrap_or_else(|| "Unknown".to_string());
        let mac_address = Self::detect_network_mac_address(&interface_path).unwrap_or_else(|| "00:00:00:00:00:00".to_string());
        let speed = Self::detect_network_speed(&interface_path).unwrap_or(0);
        let duplex = Self::detect_network_duplex(&interface_path).unwrap_or_else(|| "Full".to_string());
        let mtu = Self::detect_network_mtu(&interface_path).unwrap_or(1500);
        let state = Self::detect_network_state(&interface_path).unwrap_or_else(|| "Unknown".to_string());
        let driver = Self::detect_network_driver(&interface_path).unwrap_or_else(|| "Unknown".to_string());
        let supported_features = Self::detect_network_features(&interface_path).unwrap_or_default();
        
        Ok(NetworkInterface {
            interface_name: interface_name.to_string(),
            device_type,
            mac_address,
            speed_mbps: speed,
            duplex,
            mtu,
            state,
            driver,
            supported_features,
        })
    }

    /// Detect network device type
    fn detect_network_device_type(interface_path: &str) -> Option<String> {
        let type_path = format!("{}/type", interface_path);
        if let Ok(type_str) = std::fs::read_to_string(type_path) {
            if let Ok(type_num) = type_str.trim().parse::<u16>() {
                match type_num {
                    1 => return Some("Ethernet".to_string()),
                    6 => return Some("Token Ring".to_string()),
                    9 => return Some("FDDI".to_string()),
                    776 => return Some("WiFi".to_string()),
                    801 => return Some("Wireless".to_string()),
                    _ => return Some("Unknown".to_string()),
                }
            }
        }
        None
    }

    /// Detect network MAC address
    fn detect_network_mac_address(interface_path: &str) -> Option<String> {
        let address_path = format!("{}/address", interface_path);
        if let Ok(address) = std::fs::read_to_string(address_path) {
            return Some(address.trim().to_string());
        }
        None
    }

    /// Detect network interface speed
    fn detect_network_speed(interface_path: &str) -> Option<u32> {
        let speed_path = format!("{}/speed", interface_path);
        if let Ok(speed_str) = std::fs::read_to_string(speed_path) {
            if let Ok(speed) = speed_str.trim().parse::<u32>() {
                return Some(speed);
            }
        }
        None
    }

    /// Detect network duplex mode
    fn detect_network_duplex(interface_path: &str) -> Option<String> {
        let duplex_path = format!("{}/duplex", interface_path);
        if let Ok(duplex) = std::fs::read_to_string(duplex_path) {
            return Some(duplex.trim().to_string());
        }
        None
    }

    /// Detect network MTU
    fn detect_network_mtu(interface_path: &str) -> Option<u32> {
        let mtu_path = format!("{}/mtu", interface_path);
        if let Ok(mtu_str) = std::fs::read_to_string(mtu_path) {
            if let Ok(mtu) = mtu_str.trim().parse::<u32>() {
                return Some(mtu);
            }
        }
        None
    }

    /// Detect network interface state
    fn detect_network_state(interface_path: &str) -> Option<String> {
        let operstate_path = format!("{}/operstate", interface_path);
        if let Ok(state) = std::fs::read_to_string(operstate_path) {
            return Some(state.trim().to_string());
        }
        None
    }

    /// Detect network driver
    fn detect_network_driver(interface_path: &str) -> Option<String> {
        let device_path = format!("{}/device", interface_path);
        if let Ok(device_link) = std::fs::read_link(device_path) {
            if let Some(driver_name) = device_link.file_name() {
                return Some(driver_name.to_string_lossy().to_string());
            }
        }
        None
    }

    /// Detect network interface features
    fn detect_network_features(interface_path: &str) -> Option<Vec<String>> {
        let mut features = Vec::new();
        
        let features_path = format!("{}/features", interface_path);
        if let Ok(features_str) = std::fs::read_to_string(features_path) {
            for feature in features_str.split_whitespace() {
                features.push(feature.to_string());
            }
        }
        
        if features.is_empty() {
            None
        } else {
            Some(features)
        }
    }

    /// Detect graphics information
    async fn detect_graphics() -> Result<GraphicsInfo, HardwareError> {
        log::info!("Detecting graphics devices");
        
        let mut devices = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Try to detect graphics from /sys/class/drm
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let card_name = entry.file_name().to_string_lossy().to_string();
                        if card_name.starts_with("card") {
                            let device_info = Self::detect_graphics_device(&card_name).await?;
                            devices.push(device_info);
                        }
                    }
                }
            }
            
            // If no DRM devices found, try other methods
            if devices.is_empty() {
                let device_info = Self::detect_graphics_fallback().await?;
                devices.push(device_info);
            }
        }
        
        let primary_device = devices.first().cloned().unwrap_or_default();
        
        Ok(GraphicsInfo {
            devices,
            primary_device,
            gpu_vendor: primary_device.vendor,
            gpu_model: primary_device.model,
            driver_in_use: primary_device.driver,
            resolution: (1920, 1080), // Would be detected from X11/Wayland
            color_depth: 32,
            refresh_rate: 60,
        })
    }

    /// Detect individual graphics device
    async fn detect_graphics_device(card_name: &str) -> Result<GraphicsDevice, HardwareError> {
        let device_path = format!("/sys/class/drm/{}", card_name);
        
        let device_name = card_name.to_string();
        let vendor = Self::detect_graphics_vendor(&device_path).unwrap_or_else(|| "Unknown".to_string());
        let model = Self::detect_graphics_model(&device_path).unwrap_or_else(|| "Unknown GPU".to_string());
        let driver = Self::detect_graphics_driver(&device_path).unwrap_or_else(|| "Unknown".to_string());
        let memory_size = Self::detect_graphics_memory(&device_path).unwrap_or(0);
        let max_resolution = Self::detect_graphics_max_resolution(&device_path).unwrap_or((1920, 1080));
        let supported_outputs = Self::detect_graphics_outputs(&device_path).unwrap_or_default();
        let power_management = Self::detect_graphics_power_management(&device_path).unwrap_or_default();
        
        Ok(GraphicsDevice {
            device_name,
            vendor,
            model,
            driver,
            memory_mb: memory_size,
            max_resolution,
            supported_outputs,
            power_management,
        })
    }

    /// Fallback graphics detection
    async fn detect_graphics_fallback() -> Result<GraphicsDevice, HardwareError> {
        Ok(GraphicsDevice {
            device_name: "fallback".to_string(),
            vendor: "Unknown".to_string(),
            model: "Unknown Graphics".to_string(),
            driver: "Unknown".to_string(),
            memory_mb: 0,
            max_resolution: (1920, 1080),
            supported_outputs: Vec::new(),
            power_management: Vec::new(),
        })
    }

    /// Detect graphics vendor
    fn detect_graphics_vendor(device_path: &str) -> Option<String> {
        let vendor_path = format!("{}/device/vendor", device_path);
        if let Ok(vendor_str) = std::fs::read_to_string(vendor_path) {
            if let Ok(vendor_id) = u16::from_str_radix(vendor_str.trim().trim_start_matches("0x"), 16) {
                return match vendor_id {
                    0x1002 => Some("AMD".to_string()),
                    0x8086 => Some("Intel".to_string()),
                    0x10de => Some("NVIDIA".to_string()),
                    _ => Some(format!("Unknown ({:04x})", vendor_id)),
                };
            }
        }
        None
    }

    /// Detect graphics model
    fn detect_graphics_model(device_path: &str) -> Option<String> {
        let device_path_full = format!("{}/device/device", device_path);
        if let Ok(device_str) = std::fs::read_to_string(device_path_full) {
            if let Ok(device_id) = u16::from_str_radix(device_str.trim().trim_start_matches("0x"), 16) {
                return Some(format!("Device ID: 0x{:04x}", device_id));
            }
        }
        None
    }

    /// Detect graphics driver
    fn detect_graphics_driver(device_path: &str) -> Option<String> {
        let driver_path = format!("{}/device/driver/module/drivers", device_path);
        if let Ok(entries) = std::fs::read_dir(driver_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let driver_name = entry.file_name().to_string_lossy().to_string();
                    if driver_name.contains("drm") || driver_name.contains("video") {
                        return Some(driver_name);
                    }
                }
            }
        }
        None
    }

    /// Detect graphics memory size
    fn detect_graphics_memory(device_path: &str) -> Option<u32> {
        let mem_info_path = format!("{}/device/mem_info_vram", device_path);
        if let Ok(mem_str) = std::fs::read_to_string(mem_info_path) {
            if let Ok(mem_bytes) = mem_str.trim().parse::<u64>() {
                return Some((mem_bytes / (1024 * 1024)) as u32);
            }
        }
        None
    }

    /// Detect graphics maximum resolution
    fn detect_graphics_max_resolution(device_path: &str) -> Option<(u32, u32)> {
        // This would typically require querying the actual display
        // For now, return a common default
        Some((1920, 1080))
    }

    /// Detect graphics outputs
    fn detect_graphics_outputs(device_path: &str) -> Option<Vec<String>> {
        let mut outputs = Vec::new();
        
        let modes_path = format!("{}/modes", device_path);
        if let Ok(modes_str) = std::fs::read_to_string(modes_path) {
            for mode in modes_str.split_whitespace() {
                outputs.push(mode.to_string());
            }
        }
        
        if outputs.is_empty() {
            None
        } else {
            Some(outputs)
        }
    }

    /// Detect graphics power management features
    fn detect_graphics_power_management(device_path: &str) -> Option<Vec<String>> {
        let mut features = Vec::new();
        
        // Check for various power management features
        let power_features = [
            "power/runtime_status",
            "power/runtime_enabled",
        ];
        
        for feature in power_features {
            let feature_path = format!("{}/device/{}", device_path, feature);
            if Path::new(&feature_path).exists() {
                features.push(feature.to_string());
            }
        }
        
        if features.is_empty() {
            None
        } else {
            Some(features)
        }
    }

    /// Detect boot system information
    async fn detect_boot_system() -> Result<BootInfo, HardwareError> {
        log::info!("Detecting boot system");
        
        let boot_type = Self::detect_boot_type();
        let boot_loader = Self::detect_boot_loader();
        let firmware_vendor = Self::detect_firmware_vendor();
        let secure_boot = Self::detect_secure_boot();
        let fast_boot = Self::detect_fast_boot();
        
        Ok(BootInfo {
            boot_type,
            boot_loader,
            firmware_vendor,
            secure_boot,
            fast_boot,
        })
    }

    /// Detect boot type (UEFI vs Legacy)
    fn detect_boot_type() -> String {
        #[cfg(target_os = "linux")]
        {
            if std::fs::read_dir("/sys/firmware/efi").is_ok() {
                return "UEFI".to_string();
            }
        }
        "Legacy".to_string()
    }

    /// Detect boot loader
    fn detect_boot_loader() -> String {
        #[cfg(target_os = "linux")]
        {
            if Path::new("/boot/grub/grub.cfg").exists() {
                return "GRUB".to_string();
            } else if Path::new("/boot/efi/EFI").exists() {
                return "UEFI".to_string();
            }
        }
        "Unknown".to_string()
    }

    /// Detect firmware vendor
    fn detect_firmware_vendor() -> String {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/devices/virtual/dmi/id/sys_vendor") {
                return content.trim().to_string();
            }
        }
        "Unknown".to_string()
    }

    /// Detect secure boot status
    fn detect_secure_boot() -> bool {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/sys/firmware/efi/efivars/SecureBoot-*") {
                return content.as_bytes().get(4).copied() == Some(1);
            }
        }
        false
    }

    /// Detect fast boot status
    fn detect_fast_boot() -> bool {
        #[cfg(target_os = "linux")]
        {
            // Fast boot detection would require checking firmware settings
            // This is a simplified check
            return false;
        }
        false
    }

    /// Detect audio devices
    async fn detect_audio() -> Result<AudioInfo, HardwareError> {
        log::info!("Detecting audio devices");
        
        let mut devices = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            // Try to detect audio devices from ALSA
            if let Ok(entries) = std::fs::read_dir("/proc/asound") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let device_name = entry.file_name().to_string_lossy().to_string();
                        if device_name.starts_with("card") {
                            let device_info = Self::detect_audio_device(&device_name).await?;
                            devices.push(device_info);
                        }
                    }
                }
            }
        }
        
        Ok(AudioInfo {
            devices,
            default_device: devices.first().map(|d| d.device_name.clone()),
        })
    }

    /// Detect individual audio device
    async fn detect_audio_device(card_name: &str) -> Result<AudioDevice, HardwareError> {
        let device_name = card_name.to_string();
        let driver = "ALSA".to_string(); // Most Linux systems use ALSA
        let supported_formats = Self::detect_audio_formats(&card_name).unwrap_or_default();
        let sample_rates = Self::detect_audio_sample_rates(&card_name).unwrap_or_default();
        let channels = Self::detect_audio_channels(&card_name).unwrap_or(2);
        
        Ok(AudioDevice {
            device_name,
            driver,
            supported_formats,
            sample_rates,
            channels,
        })
    }

    /// Detect audio device supported formats
    fn detect_audio_formats(card_name: &str) -> Option<Vec<String>> {
        let formats_path = format!("/proc/asound/{}/pcm0p/sub0/hw_params", card_name);
        if let Ok(content) = std::fs::read_to_string(formats_path) {
            let mut formats = Vec::new();
            for line in content.lines() {
                if line.starts_with("format:") {
                    let format = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    if !format.is_empty() {
                        formats.push(format);
                    }
                }
            }
            return if formats.is_empty() { None } else { Some(formats) };
        }
        None
    }

    /// Detect audio device sample rates
    fn detect_audio_sample_rates(card_name: &str) -> Option<Vec<u32>> {
        let rates_path = format!("/proc/asound/{}/pcm0p/sub0/hw_params", card_name);
        if let Ok(content) = std::fs::read_to_string(rates_path) {
            let mut rates = Vec::new();
            for line in content.lines() {
                if line.starts_with("rate:") {
                    if let Ok(rate) = line.split(':').nth(1).unwrap_or("").trim().parse::<u32>() {
                        rates.push(rate);
                    }
                }
            }
            return if rates.is_empty() { None } else { Some(rates) };
        }
        None
    }

    /// Detect audio device channel count
    fn detect_audio_channels(card_name: &str) -> Option<u8> {
        let channels_path = format!("/proc/asound/{}/pcm0p/sub0/hw_params", card_name);
        if let Ok(content) = std::fs::read_to_string(channels_path) {
            for line in content.lines() {
                if line.starts_with("channels:") {
                    if let Ok(channels) = line.split(':').nth(1).unwrap_or("").trim().parse::<u8>() {
                        return Some(channels);
                    }
                }
            }
        }
        None
    }

    /// Detect input devices
    async fn detect_input_devices() -> Result<InputInfo, HardwareError> {
        log::info!("Detecting input devices");
        
        let mut keyboards = Vec::new();
        let mut mice = Vec::new();
        let mut touchpads = Vec::new();
        let mut touchscreens = Vec::new();
        
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/sys/class/input") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let device_name = entry.file_name().to_string_lossy().to_string();
                        if device_name.starts_with("input") {
                            let device_type = Self::detect_input_device_type(&device_name);
                            match device_type.as_str() {
                                "keyboard" => keyboards.push(device_name),
                                "mouse" => mice.push(device_name),
                                "touchpad" => touchpads.push(device_name),
                                "touchscreen" => touchscreens.push(device_name),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        Ok(InputInfo {
            keyboards,
            mice,
            touchpads,
            touchscreens,
        })
    }

    /// Detect input device type
    fn detect_input_device_type(device_name: &str) -> String {
        let device_path = format!("/sys/class/input/{}", device_name);
        let name_path = format!("{}/device/name", device_path);
        
        if let Ok(name) = std::fs::read_to_string(name_path) {
            let name_lower = name.to_lowercase();
            if name_lower.contains("keyboard") {
                return "keyboard".to_string();
            } else if name_lower.contains("mouse") {
                if name_lower.contains("touchpad") {
                    return "touchpad".to_string();
                }
                return "mouse".to_string();
            } else if name_lower.contains("touch") {
                return "touchscreen".to_string();
            }
        }
        
        "unknown".to_string()
    }
}

// Hardware information structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub storage: StorageInfo,
    pub network: NetworkInfo,
    pub graphics: GraphicsInfo,
    pub boot: BootInfo,
    pub audio: AudioInfo,
    pub input: InputInfo,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub detection_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub architecture: String,
    pub vendor: String,
    pub model: String,
    pub core_count: usize,
    pub thread_count: usize,
    pub frequency_mhz: u32,
    pub features: Vec<String>,
    pub cache_sizes: HashMap<String, u32>,
    pub power_management: Vec<String>,
    pub temperature: Option<f32>,
    pub utilization: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub module_count: usize,
    pub modules: Vec<MemoryModule>,
    pub speed_mhz: u32,
    pub memory_type: String,
    pub ecc_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModule {
    pub size_mb: u32,
    pub state: String,
    pub index: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub devices: Vec<StorageDevice>,
    pub total_capacity: u64,
    pub available_capacity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub device_name: String,
    pub device_type: String,
    pub capacity: u64,
    pub available: u64,
    pub interface: String,
    pub model: String,
    pub serial: Option<String>,
    pub firmware: Option<String>,
    pub is_rotational: bool,
    pub is_removable: bool,
    pub partitions: Vec<PartitionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub device_name: String,
    pub filesystem: String,
    pub size: u64,
    pub mount_point: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub devices: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub interface_name: String,
    pub device_type: String,
    pub mac_address: String,
    pub speed_mbps: u32,
    pub duplex: String,
    pub mtu: u32,
    pub state: String,
    pub driver: String,
    pub supported_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsInfo {
    pub devices: Vec<GraphicsDevice>,
    pub primary_device: GraphicsDevice,
    pub gpu_vendor: String,
    pub gpu_model: String,
    pub driver_in_use: String,
    pub resolution: (u32, u32),
    pub color_depth: u8,
    pub refresh_rate: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsDevice {
    pub device_name: String,
    pub vendor: String,
    pub model: String,
    pub driver: String,
    pub memory_mb: u32,
    pub max_resolution: (u32, u32),
    pub supported_outputs: Vec<String>,
    pub power_management: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootInfo {
    pub boot_type: String,
    pub boot_loader: String,
    pub firmware_vendor: String,
    pub secure_boot: bool,
    pub fast_boot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub devices: Vec<AudioDevice>,
    pub default_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub device_name: String,
    pub driver: String,
    pub supported_formats: Vec<String>,
    pub sample_rates: Vec<u32>,
    pub channels: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputInfo {
    pub keyboards: Vec<String>,
    pub mice: Vec<String>,
    pub touchpads: Vec<String>,
    pub touchscreens: Vec<String>,
}

impl Default for GraphicsDevice {
    fn default() -> Self {
        Self {
            device_name: "Unknown".to_string(),
            vendor: "Unknown".to_string(),
            model: "Unknown GPU".to_string(),
            driver: "Unknown".to_string(),
            memory_mb: 0,
            max_resolution: (1920, 1080),
            supported_outputs: Vec::new(),
            power_management: Vec::new(),
        }
    }
}