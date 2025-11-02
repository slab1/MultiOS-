#!/usr/bin/env python3
"""
Hardware Profile Examples
Sample hardware profiles for different system types
"""

import json
from datetime import datetime

def create_workstation_profile():
    """High-performance workstation profile"""
    return {
        "detection_timestamp": datetime.now().timestamp(),
        "profile_version": "1.0",
        "system_type": "workstation",
        "hardware": {
            "system": {
                "hostname": "workstation-01",
                "os_name": "Linux",
                "os_release": "6.1.0",
                "architecture": "x86_64",
                "uptime_seconds": 86400,
                "temperature_sensors": [
                    {"name": "CPU", "temperature_c": 45.2},
                    {"name": "GPU", "temperature_c": 38.7}
                ],
                "fan_sensors": [
                    {"name": "CPU Fan", "speed_rpm": 1200},
                    {"name": "Case Fan 1", "speed_rpm": 800}
                ]
            },
            "cpu": {
                "model": "Intel Core i9-13900K",
                "cores_physical": 24,
                "cores_logical": 32,
                "frequency_current": 3.5,
                "frequency_max": 5.8,
                "cache_l3": 32768,
                "vendor": "Intel",
                "features": ["sse4_1", "sse4_2", "avx", "avx2", "avx512", "fma", "bmi1", "bmi2"]
            },
            "memory": {
                "total_gb": 64,
                "memory_type": "DDR5-5600",
                "speed_mhz": 5600,
                "channels": 4,
                "ecc_support": False,
                "manufacturer": "Corsair"
            },
            "storage": [
                {
                    "device": "/dev/nvme0n1",
                    "size_gb": 2000,
                    "type": "nvme",
                    "filesystem": "ext4",
                    "model": "Samsung 980 PRO 2TB"
                },
                {
                    "device": "/dev/sda",
                    "size_gb": 4000,
                    "type": "ssd",
                    "filesystem": "ext4",
                    "model": "WD Black 4TB"
                }
            ],
            "network": [
                {
                    "name": "enp4s0",
                    "speed_mbps": 10000,
                    "is_ethernet": True,
                    "status": "up",
                    "ip_addresses": ["192.168.1.100"]
                },
                {
                    "name": "wlp3s0",
                    "speed_mbps": 1200,
                    "is_wireless": True,
                    "status": "up"
                }
            ],
            "gpu": [
                {
                    "name": "NVIDIA GeForce RTX 4080",
                    "memory_mb": 16384,
                    "vendor": "NVIDIA",
                    "driver_version": "530.41.03"
                }
            ],
            "usb_devices": [
                {"name": "Logitech USB Receiver", "category": "input"},
                {"name": "Corsair K95 Keyboard", "category": "input"}
            ]
        },
        "test_recommendations": {
            "cpu_tests": ["multi_core_stress_test", "thread_scaling_test", "avx_performance_test"],
            "memory_tests": ["large_memory_bandwidth_test", "memory_latency_test"],
            "storage_tests": ["nvme_performance_test", "ssd_endurance_test"],
            "gpu_tests": ["gpu_compute_test", "graphics_performance_test"],
            "network_tests": ["10g_performance_test", "wifi_performance_test"],
            "thermal_tests": ["sustained_load_test", "thermal_cycling_test"]
        }
    }

def create_server_profile():
    """Enterprise server profile"""
    return {
        "detection_timestamp": datetime.now().timestamp(),
        "profile_version": "1.0",
        "system_type": "server",
        "hardware": {
            "system": {
                "hostname": "server-01",
                "os_name": "Linux",
                "os_release": "5.15.0",
                "architecture": "x86_64",
                "uptime_seconds": 172800,
                "temperature_sensors": [
                    {"name": "CPU 1", "temperature_c": 52.3},
                    {"name": "CPU 2", "temperature_c": 51.8},
                    {"name": "Ambient", "temperature_c": 25.1}
                ]
            },
            "cpu": {
                "model": "Intel Xeon Gold 6348",
                "cores_physical": 28,
                "cores_logical": 56,
                "frequency_current": 2.4,
                "frequency_max": 3.6,
                "cache_l3": 42949,
                "vendor": "Intel",
                "features": ["sse4_1", "sse4_2", "avx", "avx2", "avx512", "fma"]
            },
            "memory": {
                "total_gb": 256,
                "memory_type": "DDR4-3200",
                "speed_mhz": 3200,
                "channels": 8,
                "ecc_support": True,
                "manufacturer": "Micron"
            },
            "storage": [
                {
                    "device": "/dev/sda",
                    "size_gb": 8000,
                    "type": "ssd",
                    "filesystem": "ext4",
                    "model": "Intel D3-S4520 8TB"
                },
                {
                    "device": "/dev/sdb",
                    "size_gb": 8000,
                    "type": "ssd",
                    "filesystem": "ext4",
                    "model": "Intel D3-S4520 8TB"
                },
                {
                    "device": "/dev/sdc",
                    "size_gb": 8000,
                    "type": "ssd",
                    "filesystem": "ext4",
                    "model": "Intel D3-S4520 8TB"
                }
            ],
            "network": [
                {
                    "name": "enp4s0f0",
                    "speed_mbps": 25000,
                    "is_ethernet": True,
                    "status": "up",
                    "ip_addresses": ["10.0.1.100"]
                },
                {
                    "name": "enp4s0f1",
                    "speed_mbps": 25000,
                    "is_ethernet": True,
                    "status": "up"
                }
            ],
            "gpu": [],
            "usb_devices": []
        },
        "test_recommendations": {
            "cpu_tests": ["multi_socket_test", "virtualization_test", "numa_performance_test"],
            "memory_tests": ["ecc_memory_test", "large_memory_test", "memory_bandwidth_test"],
            "storage_tests": ["raid_performance_test", "sas_sata_compatibility_test"],
            "network_tests": ["25g_performance_test", "bonding_test"],
            "power_tests": ["redundant_psu_test", "power_consumption_test"],
            "stability_tests": ["extended_stress_test", "memory_leak_test"]
        }
    }

def create_embedded_profile():
    """Embedded ARM system profile"""
    return {
        "detection_timestamp": datetime.now().timestamp(),
        "profile_version": "1.0",
        "system_type": "embedded",
        "hardware": {
            "system": {
                "hostname": "embedded-01",
                "os_name": "Linux",
                "os_release": "5.10.0",
                "architecture": "aarch64",
                "uptime_seconds": 43200,
                "temperature_sensors": [
                    {"name": "CPU", "temperature_c": 41.2}
                ]
            },
            "cpu": {
                "model": "ARM Cortex-A57",
                "cores_physical": 4,
                "cores_logical": 4,
                "frequency_current": 1.5,
                "frequency_max": 2.0,
                "cache_l3": 2048,
                "vendor": "ARM",
                "features": ["asimd", "fp", "neon", "sha1", "sha256", "aes"]
            },
            "memory": {
                "total_gb": 8,
                "memory_type": "LPDDR4",
                "speed_mhz": 2133,
                "channels": 2,
                "ecc_support": False,
                "manufacturer": "Samsung"
            },
            "storage": [
                {
                    "device": "/dev/mmcblk0",
                    "size_gb": 64,
                    "type": "mmc",
                    "filesystem": "ext4",
                    "model": "SanDisk 64GB eMMC"
                }
            ],
            "network": [
                {
                    "name": "eth0",
                    "speed_mbps": 1000,
                    "is_ethernet": True,
                    "status": "up",
                    "ip_addresses": ["192.168.100.50"]
                }
            ],
            "gpu": [
                {
                    "name": "Mali-T760",
                    "memory_mb": 2048,
                    "vendor": "ARM",
                    "driver_version": "1.0"
                }
            ],
            "usb_devices": [
                {"name": "USB Hub", "category": "hub"}
            ]
        },
        "test_recommendations": {
            "cpu_tests": ["arm_specific_test", "low_power_test"],
            "memory_tests": ["lpddr_test", "limited_memory_test"],
            "storage_tests": ["emmc_test", "io_performance_test"],
            "network_tests": ["ethernet_reliability_test"],
            "power_tests": ["low_power_test", "power_measurement_test"],
            "thermal_tests": ["thermal_envelope_test"]
        }
    }

def create_raspberry_pi_profile():
    """Raspberry Pi profile"""
    return {
        "detection_timestamp": datetime.now().timestamp(),
        "profile_version": "1.0",
        "system_type": "single_board",
        "hardware": {
            "system": {
                "hostname": "raspberrypi",
                "os_name": "Linux",
                "os_release": "6.1.21",
                "architecture": "aarch64",
                "uptime_seconds": 86400,
                "temperature_sensors": [
                    {"name": "SoC", "temperature_c": 38.5}
                ]
            },
            "cpu": {
                "model": "ARM Cortex-A72",
                "cores_physical": 4,
                "cores_logical": 4,
                "frequency_current": 1.5,
                "frequency_max": 2.4,
                "cache_l1": 32,
                "cache_l2": 512,
                "vendor": "Broadcom",
                "features": ["asimd", "fp", "neon", "sha1", "sha256"]
            },
            "memory": {
                "total_gb": 8,
                "memory_type": "LPDDR4",
                "speed_mhz": 2400,
                "channels": 4,
                "ecc_support": False,
                "manufacturer": "Micron"
            },
            "storage": [
                {
                    "device": "/dev/mmcblk0",
                    "size_gb": 32,
                    "type": "sd_card",
                    "filesystem": "ext4",
                    "model": "SanDisk 32GB SDHC"
                }
            ],
            "network": [
                {
                    "name": "eth0",
                    "speed_mbps": 1000,
                    "is_ethernet": True,
                    "status": "up",
                    "ip_addresses": ["192.168.1.200"]
                },
                {
                    "name": "wlan0",
                    "speed_mbps": 150,
                    "is_wireless": True,
                    "status": "up"
                }
            ],
            "gpu": [
                {
                    "name": "VideoCore VI",
                    "memory_mb": 1024,
                    "vendor": "Broadcom",
                    "driver_version": "7.12.11"
                }
            ],
            "usb_devices": [
                {"name": "USB Hub", "category": "hub"},
                {"name": "USB Storage", "category": "storage"}
            ]
        },
        "test_recommendations": {
            "cpu_tests": ["arm_benchmark", "thermal_throttling_test"],
            "memory_tests": ["sd_card_performance_test"],
            "storage_tests": ["sd_performance_test", "io_scheduler_test"],
            "network_tests": ["wifi_reliability_test", "ethernet_test"],
            "power_tests": ["raspberry_pi_power_test"],
            "gpu_tests": ["vc4_opengl_test", "h264_encoding_test"]
        }
    }

def create_gaming_rig_profile():
    """High-end gaming system profile"""
    return {
        "detection_timestamp": datetime.now().timestamp(),
        "profile_version": "1.0",
        "system_type": "gaming_rig",
        "hardware": {
            "system": {
                "hostname": "gaming-pc",
                "os_name": "Linux",
                "os_release": "6.2.0",
                "architecture": "x86_64",
                "uptime_seconds": 3600,
                "temperature_sensors": [
                    {"name": "CPU", "temperature_c": 42.8},
                    {"name": "GPU", "temperature_c": 45.2},
                    {"name": "VRM", "temperature_c": 38.1}
                ],
                "fan_sensors": [
                    {"name": "Front Fans", "speed_rpm": 1200},
                    {"name": "Rear Fans", "speed_rpm": 1000},
                    {"name": "GPU Fans", "speed_rpm": 800}
                ]
            },
            "cpu": {
                "model": "AMD Ryzen 9 7950X",
                "cores_physical": 16,
                "cores_logical": 32,
                "frequency_current": 4.2,
                "frequency_max": 5.7,
                "cache_l3": 65536,
                "vendor": "AMD",
                "features": ["sse4_1", "sse4_2", "avx", "avx2", "fma", "bmi1", "bmi2"]
            },
            "memory": {
                "total_gb": 32,
                "memory_type": "DDR5-6000",
                "speed_mhz": 6000,
                "channels": 2,
                "ecc_support": False,
                "manufacturer": "G.Skill"
            },
            "storage": [
                {
                    "device": "/dev/nvme0n1",
                    "size_gb": 2000,
                    "type": "nvme",
                    "filesystem": "ext4",
                    "model": "WD Black SN850X 2TB"
                },
                {
                    "device": "/dev/sda",
                    "size_gb": 4000,
                    "type": "ssd",
                    "filesystem": "ntfs",
                    "model": "Samsung 870 EVO 4TB"
                }
            ],
            "network": [
                {
                    "name": "enp5s0",
                    "speed_mbps": 10000,
                    "is_ethernet": True,
                    "status": "up",
                    "ip_addresses": ["192.168.1.150"]
                }
            ],
            "gpu": [
                {
                    "name": "NVIDIA GeForce RTX 4090",
                    "memory_mb": 24576,
                    "vendor": "NVIDIA",
                    "driver_version": "535.104.05"
                }
            ],
            "usb_devices": [
                {"name": "SteelSeries Keyboard", "category": "input"},
                {"name": "Razer Mouse", "category": "input"},
                {"name": "USB Headset", "category": "audio"}
            ]
        },
        "test_recommendations": {
            "cpu_tests": ["gaming_workload_test", "streaming_test", "zen4_optimization_test"],
            "memory_tests": ["ddr5_latency_test", "gaming_memory_test"],
            "storage_tests": ["nvme_gaming_test", "load_time_test"],
            "gpu_tests": ["ray_tracing_test", "dlss_performance_test", "4k_gaming_test"],
            "network_tests": ["low_latency_test", "gaming_network_test"],
            "thermal_tests": ["gaming_thermal_test", "noise_level_test"]
        }
    }

def main():
    """Generate all hardware profiles"""
    profiles = {
        "workstation": create_workstation_profile(),
        "server": create_server_profile(),
        "embedded": create_embedded_profile(),
        "raspberry_pi": create_raspberry_pi_profile(),
        "gaming_rig": create_gaming_rig_profile()
    }
    
    # Save profiles
    for profile_name, profile_data in profiles.items():
        filename = f"/workspace/testing/hardware_tests/profiles/{profile_name}_profile.json"
        with open(filename, 'w') as f:
            json.dump(profile_data, f, indent=2)
        print(f"Created {profile_name} profile: {filename}")

if __name__ == "__main__":
    main()