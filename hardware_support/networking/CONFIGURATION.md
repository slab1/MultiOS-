# MultiOS Networking Configuration Guide

This document provides comprehensive configuration examples for the MultiOS networking drivers, including Wi-Fi, Ethernet, security, QoS, and advanced features.

## Table of Contents

1. [Network Interface Configuration](#network-interface-configuration)
2. [Wi-Fi Configuration](#wifi-configuration)
3. [Ethernet Configuration](#ethernet-configuration)
4. [Security Configuration](#security-configuration)
5. [QoS Configuration](#qos-configuration)
6. [Hotplug Configuration](#hotplug-configuration)
7. [Debug Configuration](#debug-configuration)
8. [Performance Tuning](#performance-tuning)
9. [Troubleshooting](#troubleshooting)

## Network Interface Configuration

### Basic Interface Setup

```json
{
  "interfaces": [
    {
      "name": "eth0",
      "type": "ethernet",
      "mac_address": "00:1A:79:12:34:56",
      "mtu": 1500,
      "enabled": true,
      "auto_config": true,
      "qos_enabled": true,
      "ipv4_config": {
        "dhcp_enabled": true,
        "dhcp_options": {
          "lease_time": 3600,
          "renew_time": 1800,
          "rebind_time": 2700,
          "host_name": "multios-device",
          "domain_name": "local"
        }
      }
    },
    {
      "name": "wlan0",
      "type": "wifi",
      "mac_address": "00:1C:42:78:9A:BC",
      "mtu": 1500,
      "enabled": true,
      "auto_config": true,
      "qos_enabled": true,
      "ipv4_config": {
        "dhcp_enabled": true,
        "dhcp_options": {
          "lease_time": 7200,
          "renew_time": 3600,
          "rebind_time": 5400
        }
      }
    },
    {
      "name": "lo",
      "type": "loopback",
      "mtu": 65536,
      "enabled": true,
      "auto_config": false,
      "ipv4_config": {
        "address": "127.0.0.1",
        "netmask": "255.0.0.0",
        "dhcp_enabled": false
      },
      "ipv6_config": {
        "addresses": ["::1"],
        "autoconf_enabled": true,
        "dhcp_enabled": false
      }
    }
  ]
}
```

### Static IP Configuration

```json
{
  "interfaces": [
    {
      "name": "eth0",
      "type": "ethernet",
      "enabled": true,
      "ipv4_config": {
        "dhcp_enabled": false,
        "address": "192.168.1.100",
        "netmask": "255.255.255.0",
        "gateway": "192.168.1.1",
        "dns_servers": ["192.168.1.1", "8.8.8.8", "8.8.4.4"]
      },
      "ipv6_config": {
        "addresses": ["2001:db8::100/64"],
        "gateway": "2001:db8::1",
        "dns_servers": ["2001:db8::1", "2001:4860:4860::8888"],
        "autoconf_enabled": true,
        "dhcp_enabled": false
      }
    }
  ]
}
```

## Wi-Fi Configuration

### Basic Wi-Fi Setup

```json
{
  "wifi_configs": [
    {
      "ssid": "MultiOS_Main",
      "security": "WPA3",
      "password": "secure_password_123!",
      "auto_connect": true,
      "prioritize_saved": true,
      "hidden_network": false
    },
    {
      "ssid": "MultiOS_Guest",
      "security": "WPA2",
      "password": "guest_password_456!",
      "auto_connect": false,
      "prioritize_saved": false,
      "hidden_network": false
    }
  ]
}
```

### Wi-Fi Scanning Configuration

```json
{
  "wifi_scanner": {
    "scan_config": {
      "flags": ["ACTIVE_SCAN", "BAND_2_4GHZ", "BAND_5GHZ", "BAND_6GHZ"],
      "scan_interval": 30000,
      "channel_timeout": 200,
      "signal_threshold": -80,
      "retry_count": 3,
      "background_scan_enabled": true,
      "intelligent_channel_selection": true
    },
    "auto_connect_config": {
      "enabled": true,
      "preferred_networks": [
        {
          "ssid": "MultiOS_Main",
          "priority": 10,
          "security_type": "WPA3",
          "min_signal": -65,
          "network_quality": ["GOOD", "SECURE"],
          "band_preference": "Prefer6GHz"
        },
        {
          "ssid": "MultiOS_Guest",
          "priority": 5,
          "security_type": "WPA2",
          "min_signal": -70,
          "network_quality": ["FAIR", "SECURE"],
          "band_preference": "Any"
        }
      ],
      "auto_roam_enabled": true,
      "signal_threshold": -75,
      "network_quality_threshold": 60,
      "connection_timeout": 15000
    }
  }
}
```

### Advanced Wi-Fi Security

```json
{
  "security_configs": [
    {
      "wireless_security": [
        {
          "ssid": "Enterprise_AP",
          "security_type": "WPA3_Enterprise",
          "certificate": "/etc/ssl/certs/enterprise.crt",
          "key_management": "EAP",
          "eap_config": {
            "method": "TLS",
            "identity": "user@company.com",
            "client_cert": "/etc/ssl/certs/client.crt",
            "client_key": "/etc/ssl/private/client.key",
            "ca_cert": "/etc/ssl/certs/ca.crt",
            "server_name": "radius.company.com"
          }
        }
      ],
      "firewall_enabled": true,
      "firewall_rules": [
        {
          "name": "Allow Wi-Fi Management",
          "action": "Allow",
          "direction": "Both",
          "source_address": "any",
          "dest_address": "any",
          "protocol": "any",
          "enabled": true
        }
      ],
      "encryption_enabled": true,
      "pmf_enabled": true,
      "ft_enabled": false
    }
  ]
}
```

## Ethernet Configuration

### Basic Ethernet Setup

```json
{
  "ethernet_configs": [
    {
      "name": "eth0",
      "auto_negotiate": true,
      "speed": "Auto",
      "duplex": "Auto",
      "flow_control": true,
      "eee_enabled": true,
      "interrupt_coalescing": {
        "rx_usecs": 125,
        "tx_usecs": 50,
        "rx_frames": 8,
        "tx_frames": 4
      }
    },
    {
      "name": "eth1",
      "auto_negotiate": false,
      "speed": 1000,
      "duplex": "Full",
      "flow_control": false,
      "eee_enabled": false
    }
  ]
}
```

### VLAN Configuration

```json
{
  "ethernet_configs": [
    {
      "name": "eth0",
      "auto_negotiate": true,
      "vlan_configs": [
        {
          "vlan_id": 1,
          "name": "Management",
          "priority": 0,
          "enabled": true
        },
        {
          "vlan_id": 100,
          "name": "Guest",
          "priority": 1,
          "enabled": true
        },
        {
          "vlan_id": 200,
          "name": "Voice",
          "priority": 5,
          "enabled": true
        },
        {
          "vlan_id": 300,
          "name": "IoT",
          "priority": 2,
          "enabled": true
        }
      ]
    }
  ]
}
```

### Link Aggregation (LACP)

```json
{
  "link_aggregation": [
    {
      "name": "bond0",
      "mode": "LACP",
      "members": ["eth0", "eth1"],
      "load_balance": "HashAll",
      "hash_algorithm": "L3_L4",
      "lacp_config": {
        "fast_rate": true,
        "timeout": "Slow",
        "monitor_interval": 1
      }
    },
    {
      "name": "bond1",
      "mode": "Static",
      "members": ["eth2", "eth3"],
      "load_balance": "RoundRobin",
      "active_backup": true
    }
  ]
}
```

## Security Configuration

### Firewall Rules

```json
{
  "security_configs": [
    {
      "firewall_enabled": true,
      "firewall_rules": [
        {
          "name": "Allow Loopback",
          "action": "Allow",
          "direction": "Both",
          "source_address": "127.0.0.1",
          "dest_address": "127.0.0.1",
          "protocol": "any",
          "enabled": true
        },
        {
          "name": "Allow SSH",
          "action": "Allow",
          "direction": "In",
          "dest_port": 22,
          "protocol": "tcp",
          "enabled": true
        },
        {
          "name": "Allow HTTP/HTTPS",
          "action": "Allow",
          "direction": "In",
          "dest_ports": [80, 443],
          "protocol": "tcp",
          "enabled": true
        },
        {
          "name": "Allow DNS",
          "action": "Allow",
          "direction": "Both",
          "dest_port": 53,
          "protocol": "udp",
          "enabled": true
        },
        {
          "name": "Block All Other Traffic",
          "action": "Deny",
          "direction": "In",
          "enabled": true
        }
      ]
    }
  ]
}
```

### Wi-Fi Security (WPA2/WPA3)

```json
{
  "wireless_security": [
    {
      "ssid": "Secure_Network",
      "security_type": "WPA3",
      "passphrase": "ComplexPassword123!",
      "key_management": "PSK",
      "encryption": "GCMP",
      "pmf_required": true,
      "ft_enabled": true,
      "robust_security_network": true
    },
    {
      "ssid": "Enterprise_Network",
      "security_type": "WPA3_Enterprise",
      "key_management": "EAP",
      "eap_method": "PEAP",
      "eap_identity": "user@company.com",
      "eap_password": "secure_password",
      "server_certificate_validation": true,
      "ca_certificate": "/etc/ssl/certs/ca.crt"
    }
  ]
}
```

### Certificates and PKI

```json
{
  "certificate_store": [
    {
      "type": "CA",
      "path": "/etc/ssl/certs/root-ca.crt",
      "trusted": true
    },
    {
      "type": "Server",
      "path": "/etc/ssl/certs/server.crt",
      "private_key": "/etc/ssl/private/server.key",
      "password": "key_password"
    },
    {
      "type": "Client",
      "path": "/etc/ssl/certs/client.crt",
      "private_key": "/etc/ssl/private/client.key"
    }
  ]
}
```

## QoS Configuration

### Traffic Classes

```json
{
  "qos_configs": [
    {
      "name": "Default QoS",
      "enabled": true,
      "default_class": "BEST_EFFORT",
      "bandwidth_limit": null,
      "burst_rate": null,
      "classes": [
        {
          "class_name": "Voice",
          "priority": 7,
          "bandwidth_percentage": 15,
          "flow_control": true,
          "filters": [
            {
              "protocol": "tcp",
              "port": 5060,
              "dscp": 46
            },
            {
              "protocol": "udp",
              "port_range": [10000, 20000],
              "dscp": 46
            }
          ]
        },
        {
          "class_name": "Video",
          "priority": 5,
          "bandwidth_percentage": 30,
          "flow_control": false,
          "filters": [
            {
              "protocol": "tcp",
              "port_range": [1935, 1935],
              "dscp": 34
            }
          ]
        },
        {
          "class_name": "Gaming",
          "priority": 6,
          "bandwidth_percentage": 20,
          "flow_control": false,
          "filters": [
            {
              "protocol": "udp",
              "port_range": [27015, 27015],
              "dscp": 32
            }
          ]
        },
        {
          "class_name": "File Transfer",
          "priority": 2,
          "bandwidth_percentage": 25,
          "flow_control": false,
          "filters": [
            {
              "protocol": "tcp",
              "port": 20,
              "dscp": 8
            },
            {
              "protocol": "tcp",
              "port": 21,
              "dscp": 8
            }
          ]
        }
      ]
    }
  ]
}
```

### Bandwidth Limiting

```json
{
  "bandwidth_policies": [
    {
      "interface": "eth0",
      "policy": "RateLimit",
      "limit": 100000,  // 100 Mbps in kbps
      "burst": 20000,   // 20 Mbps burst
      "direction": "Both",
      "filters": [
        {
          "protocol": "tcp",
          "port": 80,
          "limit": 50000  // HTTP limited to 50 Mbps
        }
      ]
    }
  ]
}
```

## Hotplug Configuration

### Device Hotplug Settings

```json
{
  "hotplug_config": {
    "auto_detect": true,
    "auto_configure": true,
    "power_management": true,
    "event_logging": true,
    "max_devices": 16,
    "recovery_timeout": 30,
    "health_check_interval": 10,
    "thermal_monitoring": true,
    "load_balancing_enabled": true,
    "device_policies": {
      "wifi_adapters": {
        "auto_connect": true,
        "prefer_saved_networks": true,
        "signal_threshold": -75
      },
      "ethernet_adapters": {
        "enable_eee": true,
        "enable_flow_control": true,
        "preferred_speed": "Auto"
      }
    }
  }
}
```

### Event Handlers

```json
{
  "hotplug_event_handlers": [
    {
      "name": "LogHandler",
      "type": "Logger",
      "enabled": true
    },
    {
      "name": "AutoConfigHandler",
      "type": "AutoConfiguration",
      "enabled": true,
      "config_timeout": 30
    },
    {
      "name": "AlertHandler",
      "type": "Alert",
      "enabled": true,
      "alert_levels": ["Warning", "Error", "Critical"]
    },
    {
      "name": "RecoveryHandler",
      "type": "AutoRecovery",
      "enabled": true,
      "max_recovery_attempts": 3
    }
  ]
}
```

## Debug Configuration

### Debug and Monitoring

```json
{
  "debug_configs": {
    "logging_enabled": true,
    "debug_flags": [
      "PACKET_CAPTURE",
      "SIGNAL_MONITORING",
      "INTERFERENCE_ANALYSIS",
      "PERFORMANCE_MONITORING",
      "TRAFFIC_MONITORING"
    ],
    "log_level": "Info",
    "performance_monitoring": true,
    "packet_capture_enabled": false,
    "alert_thresholds": {
      "low_signal_threshold": -70,
      "high_interference_threshold": 70,
      "packet_loss_threshold": 5,
      "temperature_threshold": 80.0
    }
  }
}
```

### Network Diagnostics

```json
{
  "diagnostic_config": {
    "periodic_health_checks": true,
    "health_check_interval": 60,
    "automated_testing": true,
    "performance_baselines": {
      "throughput": 1000,  // Mbps
      "latency": 10,       // milliseconds
      "packet_loss": 0.1,  // percentage
      "signal_strength": -50  // dBm
    },
    "alerts": {
      "email_notifications": true,
      "snmp_traps": false,
      "syslog_integration": true
    }
  }
}
```

## Performance Tuning

### High-Performance Configuration

```json
{
  "performance_config": {
    "profile": "HighPerformance",
    "buffer_sizes": {
      "rx_buffer_size": 4096,
      "tx_buffer_size": 2048,
      "large_buffer_size": 8192
    },
    "interrupt_coalescing": {
      "rx_usecs": 50,
      "tx_usecs": 25,
      "rx_frames": 16,
      "tx_frames": 8
    },
    "interrupt_affinity": {
      "rx_cpus": [0, 1],
      "tx_cpus": [2, 3]
    },
    "features": {
      "rss_enabled": true,
      "rps_enabled": false,
      "xfrm_enabled": true,
      "gro_enabled": true,
      "tso_enabled": true,
      "gso_enabled": true,
      "lro_enabled": true
    }
  }
}
```

### Power-Saving Configuration

```json
{
  "power_config": {
    "profile": "PowerSaver",
    "eee_settings": {
      "enabled": true,
      "link_down_timeout": 1000,
      "power_saving_level": "Maximum"
    },
    "wifi_power_management": {
      "mode": "PowerSave",
      "beacon_interval": 100,
      "listen_interval": 5,
      "dtim_period": 3
    },
    "ethernet_power_management": {
      "eee_enabled": true,
      "auto_speed_downgrade": true,
      "cable_detection": true
    },
    "sleep_settings": {
      "idle_timeout": 300,
      "deep_sleep_enabled": true,
      "wake_on_lan": true,
      "magic_packet_enabled": true
    }
  }
}
```

### Gaming Configuration

```json
{
  "gaming_config": {
    "profile": "Gaming",
    "latency_optimization": {
      "interrupt_coalescing": {
        "rx_usecs": 25,
        "tx_usecs": 10
      },
      "buffer_sizes": {
        "rx_buffer_size": 1024,
        "tx_buffer_size": 512
      },
      "priority_queues": true,
      "flow_control": false
    },
    "traffic_shaping": {
      "gaming_traffic_priority": 7,
      "reserved_bandwidth": 20,
      "latency_threshold": 20
    },
    "features": {
      "rss_enabled": true,
      "xfrm_enabled": false,
      "gro_enabled": false,
      "tso_enabled": false,
      "gso_enabled": false
    }
  }
}
```

## Troubleshooting

### Common Issues and Solutions

#### Wi-Fi Connection Issues

```json
{
  "troubleshooting": {
    "wifi_issues": {
      "connection_timeout": {
        "diagnosis": "Check signal strength and interference",
        "solutions": [
          "Move closer to access point",
          "Change to less congested channel",
          "Disable other devices causing interference",
          "Update Wi-Fi driver"
        ]
      },
      "authentication_failed": {
        "diagnosis": "Verify security credentials and protocols",
        "solutions": [
          "Check WPA/WPA2 password",
          "Verify security protocol compatibility",
          "Check for certificate issues (Enterprise)",
          "Try WPS connection method"
        ]
      },
      "low_throughput": {
        "diagnosis": "Analyze signal quality and channel utilization",
        "solutions": [
          "Check for background applications",
          "Verify Wi-Fi standard support",
          "Analyze interference sources",
          "Consider upgrading to Wi-Fi 6"
        ]
      }
    }
  }
}
```

#### Ethernet Issues

```json
{
  "ethernet_troubleshooting": {
    "link_down": {
      "diagnosis": "Check physical connection and auto-negotiation",
      "solutions": [
        "Verify cable connections",
        "Check link partner status",
        "Restart network adapter",
        "Update driver firmware"
      ]
    },
    "slow_performance": {
      "diagnosis": "Analyze link speed, duplex, and error rates",
      "solutions": [
        "Check for speed/duplex mismatches",
        "Enable flow control",
        "Disable power saving features",
        "Analyze packet loss and errors"
      ]
    },
    "high_latency": {
      "diagnosis": "Check interrupt handling and buffer sizes",
      "solutions": [
        "Adjust interrupt coalescing",
        "Increase buffer sizes",
        "Enable interrupt moderation",
        "Check for buffer overruns"
      ]
    }
  }
}
```

### Monitoring and Diagnostics

```bash
# Network interface status
netif-stat eth0
netif-stat wlan0

# Wi-Fi scan and connection status
wifi-scan --detailed
wifi-connect --status

# Performance monitoring
net-perf --continuous --interface eth0
net-monitor --alerts --log-level info

# Traffic analysis
tcpdump --interface eth0 --port 80
netstat --statistics --protocol tcp

# Hardware diagnostics
hwinfo --network
dmidecode --type network
```

This configuration guide provides comprehensive examples for configuring all aspects of the MultiOS networking system. Adjust the configurations based on your specific hardware and requirements.