//! MultiOS IPC System Examples
//! 
//! This module contains examples demonstrating how to use the various
//! IPC mechanisms implemented in the MultiOS system.

use crate::channels::{ChannelHandle, ChannelMessage};
use crate::shared_memory::{SharedMemoryHandle, MemoryPermissions};
use crate::semaphores::{SemaphoreHandle, Mutex};
use crate::message_queue::{MessageQueueHandle, Message};
use crate::pipes::{Pipe, PipeFlags};
use crate::signals::{SignalManager, SignalHandler, SignalAction, SignalFlags};
use crate::events::{Event, EventType, EventFlags, EventData};
use crate::network::{NetworkManager, NetworkEndpoint, AddressType, NetworkProtocol};

/// Example 1: Basic Channel Communication
/// Demonstrates message passing between processes
pub fn example_channel_communication() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create a channel for communication
    let channel_handle = crate::create_channel(1024).unwrap();
    
    // Process 1 sends a message
    let data = b"Hello, from process 1!";
    let send_result = channel_handle.send(data, 100);
    match send_result {
        Ok(bytes_sent) => println!("Process 1 sent {} bytes", bytes_sent),
        Err(e) => println!("Process 1 failed to send: {:?}", e),
    }
    
    // Process 2 receives the message
    let mut buffer = vec![0u8; data.len()];
    let receive_result = channel_handle.receive(101, &mut buffer);
    match receive_result {
        Ok(Some(message)) => println!("Process 2 received: {}", String::from_utf8_lossy(&message.data)),
        Ok(None) => println!("Process 2: no message available"),
        Err(e) => println!("Process 2 failed to receive: {:?}", e),
    }
}

/// Example 2: Shared Memory Producer-Consumer
/// Demonstrates shared memory usage with synchronization
pub fn example_shared_memory_producer_consumer() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create shared memory for the buffer
    let shm_handle = crate::create_shared_memory(4096).unwrap();
    
    // Create semaphores for synchronization
    let empty_semaphore = crate::create_semaphore(10).unwrap(); // 10 empty slots
    let full_semaphore = crate::create_semaphore(0).unwrap();   // 0 full slots initially
    let mutex = crate::create_semaphore(1).unwrap();            // Mutual exclusion
    
    // Producer process
    {
        let data = b"Shared memory data";
        
        // Wait for empty slot
        empty_semaphore.wait(100, None).unwrap();
        
        // Acquire mutex
        mutex.wait(100, None).unwrap();
        
        // Write to shared memory
        shm_handle.write(0, data).unwrap();
        
        // Release mutex
        mutex.post(100).unwrap();
        
        // Signal that slot is now full
        full_semaphore.post(100).unwrap();
        
        println!("Producer: wrote {} bytes to shared memory", data.len());
    }
    
    // Consumer process
    {
        let mut buffer = vec![0u8; 4096];
        
        // Wait for full slot
        full_semaphore.wait(101, None).unwrap();
        
        // Acquire mutex
        mutex.wait(101, None).unwrap();
        
        // Read from shared memory
        let bytes_read = shm_handle.read(0, &mut buffer).unwrap();
        
        // Release mutex
        mutex.post(101).unwrap();
        
        // Signal that slot is now empty
        empty_semaphore.post(101).unwrap();
        
        println!("Consumer: read {} bytes from shared memory", bytes_read);
    }
}

/// Example 3: Message Queue with Priorities
/// Demonstrates priority-based message queuing
pub fn example_message_queue_priorities() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create a message queue with priority ordering
    let flags = message_queue::MessageQueueFlags::PRIORITY_ORDER;
    let mq_handle = crate::create_message_queue(b"priority_queue", 1024, 10, flags).unwrap();
    
    // Send messages with different priorities
    let high_priority_msg = Message {
        data: b"High priority message".to_vec(),
        priority: 3,
        sender_id: 100,
        timestamp: 0,
        message_id: 1,
        delivery_count: 1,
    };
    
    let medium_priority_msg = Message {
        data: b"Medium priority message".to_vec(),
        priority: 2,
        sender_id: 100,
        timestamp: 0,
        message_id: 2,
        delivery_count: 1,
    };
    
    let low_priority_msg = Message {
        data: b"Low priority message".to_vec(),
        priority: 1,
        sender_id: 100,
        timestamp: 0,
        message_id: 3,
        delivery_count: 1,
    };
    
    // Send messages
    println!("Sending messages with different priorities...");
    
    // Receive messages (should be in priority order)
    for i in 1..=3 {
        let mut buffer = vec![0u8; 1024];
        let mut priority = 0;
        
        match mq_handle.receive(&mut buffer, &mut priority, 101, None) {
            Ok(bytes_read) => {
                let message = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received (priority {}): {}", priority, message);
            }
            Err(e) => println!("Failed to receive message {}: {:?}", i, e),
        }
    }
}

/// Example 4: Event-Based Synchronization
/// Demonstrates event objects for process coordination
pub fn example_event_synchronization() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create an auto-reset event
    let event_handle = crate::create_event(EventType::AutoReset, EventFlags::empty()).unwrap();
    
    // Process 1: Signal the event
    {
        let data = Some(EventData {
            data: b"Event signaled!".to_vec(),
            data_type: 0,
            sender_id: 100,
            timestamp: 0,
        });
        
        let woken_processes = event_handle.signal(100, data).unwrap();
        println!("Process 1: signaled event (woke {} processes)", woken_processes);
    }
    
    // Process 2: Wait for the event
    {
        let wait_result = event_handle.wait(101, 1, None);
        match wait_result {
            Ok(()) => {
                println!("Process 2: event was already signaled or was signaled while waiting");
                
                // Check if there's event data
                if let Some(data) = event_handle.get_data() {
                    let message = String::from_utf8_lossy(&data.data);
                    println!("Process 2: received event data: {}", message);
                }
            }
            Err(e) => println!("Process 2: wait failed: {:?}", e),
        }
    }
    
    // Process 3: Manual reset event
    {
        let manual_event = crate::create_event(EventType::ManualReset, EventFlags::BROADCAST).unwrap();
        
        // Signal the event
        manual_event.signal(102, None).unwrap();
        
        // Multiple processes can wait on manual reset events
        println!("Process 3: created manual reset event for broadcasting");
        
        // Broadcast to all waiters
        let woken_count = manual_event.broadcast(102, None).unwrap();
        println!("Process 3: broadcasted to {} waiters", woken_count);
    }
}

/// Example 5: Pipe Communication
/// Demonstrates unidirectional pipe communication
pub fn example_pipe_communication() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create a pipe
    let pipe = crate::create_pipe(1024, PipeFlags::NON_BLOCKING).unwrap();
    
    // Get file descriptors for read and write ends
    let read_fd = pipe.read_fd();
    let write_fd = pipe.write_fd();
    
    println!("Pipe created: read_fd={}, write_fd={}", read_fd, write_fd);
    
    // Write end: send data
    {
        let data = b"Pipe communication test data";
        let bytes_written = pipe.write(data, 100).unwrap();
        println!("Write end: sent {} bytes", bytes_written);
    }
    
    // Read end: receive data
    {
        let mut buffer = vec![0u8; 1024];
        let bytes_read = pipe.read(&mut buffer, 101).unwrap();
        let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Read end: received {} bytes: {}", bytes_read, received_data);
    }
    
    // Test named pipes
    {
        let named_pipe = crate::create_named_pipe(b"named_pipe", 2048, PipeFlags::empty()).unwrap();
        let pipe_name = named_pipe.get_name().unwrap();
        println!("Named pipe created: {}", String::from_utf8_lossy(pipe_name));
    }
}

/// Example 6: Signal Handling
/// Demonstrates signal management between processes
pub fn example_signal_handling() {
    let signal_manager = SignalManager::new();
    
    // Register processes
    signal_manager.register_process(100);
    signal_manager.register_process(200);
    
    // Send signals between processes
    println!("Sending SIGUSR1 from process 100 to process 200");
    signal_manager.send_signal(signal::SIGUSR1, 200, 100, b"Signal data".to_vec()).unwrap();
    
    // Process signals
    let delivered_signals = signal_manager.process_signals(200);
    for signal in delivered_signals {
        println!("Process 200 received signal {} from process {}", signal.signal, signal.sender_pid);
    }
    
    // Set up signal handler
    let handler = SignalHandler {
        action: SignalAction::Custom,
        handler_function: Some(custom_signal_handler),
        mask: vec![signal::SIGUSR2],
        flags: SignalFlags::RELIABLE,
    };
    
    signal_manager.install_handler(100, signal::SIGUSR2, handler).unwrap();
    
    // Broadcast signal to multiple processes
    let target_pids = vec![100, 200, 300];
    let delivered_count = signal_manager.broadcast_signal(signal::SIGTERM, &target_pids, 999, Vec::new()).unwrap();
    println!("Broadcasted signal to {} processes", delivered_count);
    
    // Get signal statistics
    let stats = signal_manager.get_statistics();
    println!("Signal statistics: {} sent, {} delivered", stats.signals_sent, stats.signals_delivered);
}

/// Custom signal handler function
extern "C" fn custom_signal_handler(
    signal: SignalNumber,
    info: &SignalInfo,
    context: *mut u8,
) {
    println!("Custom handler: received signal {} from process {}", signal, info.sender_pid);
    // Handle the signal...
}

/// Example 7: Network IPC Communication
/// Demonstrates network-based IPC between processes
pub fn example_network_ipc() {
    let network_manager = NetworkManager::new();
    
    // Create network interface
    let interface_endpoint = NetworkEndpoint {
        address_type: AddressType::IPv4,
        address: vec![192, 168, 1, 100],
        port: 0,
        protocol: NetworkProtocol::TCP,
    };
    
    let interface_id = network_manager.add_interface(b"eth0", interface_endpoint, 1500).unwrap();
    println!("Created network interface: {}", interface_id);
    
    // Create a server
    let server_endpoint = NetworkEndpoint {
        address_type: AddressType::IPv4,
        address: vec![0, 0, 0, 0], // Listen on all interfaces
        port: 8080,
        protocol: NetworkProtocol::TCP,
    };
    
    let server_id = network_manager.create_server(server_endpoint, NetworkProtocol::TCP, 100).unwrap();
    println!("Created server: {}", server_id);
    
    // Start listening
    network_manager.start_listening(server_id).unwrap();
    println!("Server started listening on port 8080");
    
    // Create a client
    let client_id = network_manager.create_client(server_endpoint).unwrap();
    println!("Created client: {}", client_id);
    
    // Connect client to server
    let connection_handle = network_manager.connect_client(client_id).unwrap();
    println!("Client connected with handle: {}", connection_handle.id);
    
    // Send message over network
    let message = b"Hello, Network IPC!";
    let bytes_sent = network_manager.send_message(connection_handle, message, None).unwrap();
    println!("Sent {} bytes over network", bytes_sent);
    
    // Receive message
    let mut buffer = vec![0u8; message.len()];
    let bytes_received = network_manager.receive_message(connection_handle, &mut buffer).unwrap();
    println!("Received {} bytes over network", bytes_received);
    
    // Broadcast message
    let broadcast_endpoint = NetworkEndpoint {
        address_type: AddressType::IPv4,
        address: vec![255, 255, 255, 255], // Broadcast address
        port: 8080,
        protocol: NetworkProtocol::UDP,
    };
    
    let broadcast_bytes = network_manager.broadcast_message(message, broadcast_endpoint).unwrap();
    println!("Broadcasted {} bytes", broadcast_bytes);
    
    // Get network statistics
    let network_stats = network_manager.get_network_statistics();
    println!("Network statistics: {} connections, {} servers", 
             network_stats.active_connections, network_stats.active_servers);
    
    // Ping test
    let ping_endpoint = NetworkEndpoint {
        address_type: AddressType::IPv4,
        address: vec![127, 0, 0, 1], // localhost
        port: 80,
        protocol: NetworkProtocol::TCP,
    };
    
    if let Ok(latency) = network_manager.ping(ping_endpoint, 1000000000) { // 1 second timeout
        println!("Ping latency: {} ns", latency);
    }
    
    // Close connection
    network_manager.close_connection(connection_handle).unwrap();
    println!("Closed network connection");
}

/// Example 8: Complex Multi-Process Coordination
/// Demonstrates using multiple IPC mechanisms together
pub fn example_complex_coordination() {
    // Initialize IPC system
    crate::init().unwrap();
    
    // Create a complex coordination scenario with multiple processes
    
    // 1. Create synchronization primitives
    let start_event = crate::create_event(EventType::ManualReset, EventFlags::empty()).unwrap();
    let ready_semaphore = crate::create_semaphore(0).unwrap(); // Indicates processes are ready
    let mutex = crate::create_semaphore(1).unwrap();
    
    // 2. Create communication channels
    let control_channel = crate::create_channel(256).unwrap();
    let data_channel = crate::create_channel(4096).unwrap();
    
    // 3. Create shared memory for data sharing
    let shared_buffer = crate::create_shared_memory(8192).unwrap();
    
    // 4. Create message queue for logging
    let log_queue = crate::create_message_queue(
        b"coordination_log",
        512,
        50,
        message_queue::MessageQueueFlags::PRIORITY_ORDER
    ).unwrap();
    
    println!("Created coordination primitives for multi-process coordination");
    
    // Simulate multi-process coordination:
    // - Master process coordinates worker processes
    // - Workers signal readiness
    // - Master distributes work via shared memory
    // - Results collected via data channels
    // - Coordination events manage synchronization
    // - Logging tracks progress
    
    // Process coordination simulation
    {
        // Signal all workers to start
        let start_count = start_event.broadcast(100, None).unwrap();
        println!("Master: started {} worker processes", start_count);
        
        // Wait for workers to be ready
        for i in 0..start_count {
            ready_semaphore.wait(100, None).unwrap();
            println!("Master: worker {} is ready", i);
        }
        
        // Distribute work (simulated)
        let work_data = b"Work assignment data for workers";
        shared_buffer.write(0, work_data).unwrap();
        
        // Send work notifications via channels
        for i in 0..start_count {
            let notification = format!("Worker {}: process your work", i);
            control_channel.send(notification.as_bytes(), 100).unwrap();
        }
        
        // Collect results (simulated)
        for _ in 0..start_count {
            let mut result_buffer = vec![0u8; 256];
            if let Ok(Some(message)) = data_channel.receive(100, &mut result_buffer) {
                println!("Master: received result: {}", String::from_utf8_lossy(&message.data));
            }
        }
        
        println!("Master: coordination completed successfully");
    }
}

/// Example 9: Performance Benchmarking
/// Demonstrates performance testing of IPC mechanisms
pub fn example_performance_benchmark() {
    use std::time::Instant;
    
    // Initialize IPC system
    crate::init().unwrap();
    
    println!("Starting IPC performance benchmarks...");
    
    // Benchmark 1: Channel throughput
    {
        let channel = crate::create_channel(4096).unwrap();
        let data = vec![42u8; 1024]; // 1KB messages
        let iterations = 1000;
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let _ = channel.send(&data, 100);
            let mut buffer = vec![0u8; data.len()];
            let _ = channel.receive(101, &mut buffer);
        }
        
        let elapsed = start.elapsed();
        let throughput = (iterations as f64 * data.len() as f64 * 2.0) / elapsed.as_secs_f64(); // Send + receive
        
        println!("Channel benchmark: {:.2} MB/s ({:.2} ops/s)", 
                 throughput / 1024.0 / 1024.0, 
                 throughput / data.len() as f64);
    }
    
    // Benchmark 2: Shared memory latency
    {
        let shm = crate::create_shared_memory(4096).unwrap();
        let data = vec![42u8; 1024];
        let iterations = 10000;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = shm.write(0, &data);
            let mut buffer = vec![0u8; data.len()];
            let _ = shm.read(0, &mut buffer);
        }
        
        let elapsed = start.elapsed();
        let latency_us = elapsed.as_micros() as f64 / iterations as f64;
        
        println!("Shared memory benchmark: {:.2} μs latency", latency_us);
    }
    
    // Benchmark 3: Semaphore operations
    {
        let semaphore = crate::create_semaphore(1).unwrap();
        let iterations = 10000;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = semaphore.try_wait(100);
            let _ = semaphore.post(100);
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!("Semaphore benchmark: {:.2} ops/s", ops_per_sec);
    }
    
    // Benchmark 4: Event signaling
    {
        let event = crate::create_event(EventType::AutoReset, EventFlags::empty()).unwrap();
        let iterations = 5000;
        
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = event.try_wait(100);
            let _ = event.signal(100, None);
        }
        
        let elapsed = start.elapsed();
        let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();
        
        println!("Event benchmark: {:.2} ops/s", ops_per_sec);
    }
    
    println!("Performance benchmarks completed");
}

#[cfg(test)]
mod examples_tests {
    use super::*;

    #[test]
    fn test_channel_example() {
        example_channel_communication();
    }

    #[test]
    fn test_shared_memory_example() {
        example_shared_memory_producer_consumer();
    }

    #[test]
    fn test_message_queue_example() {
        example_message_queue_priorities();
    }

    #[test]
    fn test_event_example() {
        example_event_synchronization();
    }

    #[test]
    fn test_pipe_example() {
        example_pipe_communication();
    }

    #[test]
    fn test_signal_example() {
        example_signal_handling();
    }

    #[test]
    fn test_network_example() {
        example_network_ipc();
    }

    #[test]
    fn test_complex_coordination_example() {
        example_complex_coordination();
    }

    #[test]
    fn test_performance_benchmark() {
        example_performance_benchmark();
    }
}
