"""
Robot Communication Protocols Module

Provides communication interfaces for robot networking including WiFi, Bluetooth, and Serial
"""

import socket
import time
import threading
import json
import struct
from typing import Dict, List, Optional, Callable, Any, Tuple
from dataclasses import dataclass, asdict
from abc import ABC, abstractmethod
import asyncio
import websockets
import serial
import serial.tools.list_ports


@dataclass
class Message:
    """Container for communication messages"""
    sender_id: str
    receiver_id: Optional[str] = None
    message_type: str = "data"
    data: Any = None
    timestamp: float = 0.0
    priority: int = 1  # 1=low, 5=high
    
    def __post_init__(self):
        if self.timestamp == 0.0:
            self.timestamp = time.time()
            
    def to_dict(self) -> Dict:
        """Convert message to dictionary for serialization"""
        return asdict(self)
        
    @classmethod
    def from_dict(cls, data: Dict) -> 'Message':
        """Create message from dictionary"""
        return cls(**data)


class BaseCommunication(ABC):
    """Abstract base class for communication interfaces"""
    
    def __init__(self, config: Optional[Dict] = None):
        self.config = config or {}
        self.is_connected = False
        self.message_handlers = {}
        self._connection_lock = threading.Lock()
        
    @abstractmethod
    def connect(self) -> bool:
        """Establish communication connection"""
        pass
        
    @abstractmethod
    def disconnect(self) -> bool:
        """Close communication connection"""
        pass
        
    @abstractmethod
    def send_message(self, message: Message) -> bool:
        """Send a message"""
        pass
        
    @abstractmethod
    def receive_message(self) -> Optional[Message]:
        """Receive a message (non-blocking)"""
        pass
        
    def register_handler(self, message_type: str, handler: Callable[[Message], None]):
        """Register message handler"""
        if message_type not in self.message_handlers:
            self.message_handlers[message_type] = []
        self.message_handlers[message_type].append(handler)
        
    def _handle_message(self, message: Message):
        """Process received message through handlers"""
        if message.message_type in self.message_handlers:
            for handler in self.message_handlers[message.message_type]:
                try:
                    handler(message)
                except Exception as e:
                    print(f"Error in message handler: {e}")


class SerialCommunication(BaseCommunication):
    """Serial communication interface for robot control"""
    
    def __init__(self, config: Optional[Dict] = None):
        super().__init__(config)
        self.serial_port = None
        self.baudrate = config.get('baudrate', 115200) if config else 115200
        self.timeout = config.get('timeout', 1.0) if config else 1.0
        self.port = config.get('port', '/dev/ttyUSB0') if config else '/dev/ttyUSB0'
        
        self._read_thread = None
        self._running = False
        self._message_queue = []
        self._queue_lock = threading.Lock()
        
    def connect(self) -> bool:
        """Connect to serial device"""
        with self._connection_lock:
            try:
                # Try to connect to serial port
                self.serial_port = serial.Serial(
                    port=self.port,
                    baudrate=self.baudrate,
                    timeout=self.timeout,
                    parity=serial.PARITY_NONE,
                    stopbits=serial.STOPBITS_ONE,
                    bytesize=serial.EIGHTBITS
                )
                
                if self.serial_port.is_open:
                    self.is_connected = True
                    self._running = True
                    self._start_read_loop()
                    print(f"Serial communication established on {self.port} at {self.baudrate} baud")
                    return True
                else:
                    print("Failed to open serial port")
                    return False
                    
            except Exception as e:
                print(f"Serial connection error: {e}")
                # Try simulation mode
                self._start_simulation_mode()
                return True
                
    def _start_simulation_mode(self):
        """Start simulation mode for testing without serial device"""
        print("Serial communication simulation mode")
        self.is_connected = True
        self.serial_port = None
        self._running = True
        self._start_read_loop()
        
    def _start_read_loop(self):
        """Start reading from serial interface"""
        self._read_thread = threading.Thread(target=self._read_loop, daemon=True)
        self._read_thread.start()
        
    def _read_loop(self):
        """Main read loop for serial communication"""
        buffer = b""
        
        while self._running:
            try:
                if self.serial_port and self.serial_port.is_open:
                    # Read available data
                    if self.serial_port.in_waiting > 0:
                        data = self.serial_port.read(self.serial_port.in_waiting)
                        buffer += data
                        
                        # Process complete messages (assuming newline delimiter)
                        while b'\n' in buffer:
                            line, buffer = buffer.split(b'\n', 1)
                            try:
                                message_data = json.loads(line.decode('utf-8'))
                                message = Message.from_dict(message_data)
                                self._queue_message(message)
                            except (json.JSONDecodeError, UnicodeDecodeError) as e:
                                print(f"Error parsing serial message: {e}")
                                
                elif self.serial_port is None:
                    # Simulation mode - generate dummy messages occasionally
                    if self._running and len(self._message_queue) < 10:
                        sim_message = Message(
                            sender_id="simulator",
                            message_type="status",
                            data={"status": "simulation_active"},
                            timestamp=time.time()
                        )
                        self._queue_message(sim_message)
                        time.sleep(2.0)  # Generate message every 2 seconds
                        
                time.sleep(0.01)  # 10ms sleep
                
            except Exception as e:
                print(f"Serial read error: {e}")
                time.sleep(0.1)
                
    def _queue_message(self, message: Message):
        """Add message to queue for processing"""
        with self._queue_lock:
            self._message_queue.append(message)
            
    def disconnect(self) -> bool:
        """Disconnect from serial device"""
        self._running = False
        
        if self._read_thread and self._read_thread.is_alive():
            self._read_thread.join(timeout=1.0)
            
        if self.serial_port and self.serial_port.is_open:
            self.serial_port.close()
            
        self.is_connected = False
        print("Serial communication disconnected")
        return True
        
    def send_message(self, message: Message) -> bool:
        """Send message over serial interface"""
        try:
            # Serialize message to JSON
            message_data = json.dumps(message.to_dict())
            message_bytes = (message_data + '\n').encode('utf-8')
            
            if self.serial_port and self.serial_port.is_open:
                self.serial_port.write(message_bytes)
                return True
            elif self.serial_port is None:
                # Simulation mode
                print(f"Simulated sending: {message.message_type} to {message.receiver_id}")
                return True
            else:
                print("Serial port not connected")
                return False
                
        except Exception as e:
            print(f"Serial send error: {e}")
            return False
            
    def receive_message(self) -> Optional[Message]:
        """Receive message from queue"""
        with self._queue_lock:
            if self._message_queue:
                return self._message_queue.pop(0)
        return None
        
    @staticmethod
    def list_available_ports() -> List[str]:
        """List available serial ports"""
        ports = []
        try:
            port_list = serial.tools.list_ports.comports()
            for port in port_list:
                ports.append(port.device)
        except Exception as e:
            print(f"Error listing ports: {e}")
            
        return ports
        
    def send_robot_command(self, command: str, parameters: Dict) -> bool:
        """Send formatted robot command"""
        message = Message(
            sender_id="master",
            receiver_id="robot",
            message_type="command",
            data={
                "command": command,
                "parameters": parameters
            }
        )
        
        return self.send_message(message)


class WiFiCommunication(BaseCommunication):
    """WiFi communication interface for robot networking"""
    
    def __init__(self, config: Optional[Dict] = None):
        super().__init__(config)
        self.host = config.get('host', 'localhost') if config else 'localhost'
        self.port = config.get('port', 8080) if config else 8080
        self.role = config.get('role', 'client') if config else 'client'  # 'client' or 'server'
        
        self.socket = None
        self.server_socket = None
        self.connections = {}  # connection_id -> connection
        self._accept_thread = None
        self._running = False
        
    def connect(self) -> bool:
        """Connect via WiFi"""
        with self._connection_lock:
            try:
                if self.role == 'server':
                    return self._start_server()
                else:
                    return self._connect_client()
                    
            except Exception as e:
                print(f"WiFi connection error: {e}")
                return False
                
    def _start_server(self) -> bool:
        """Start WiFi server"""
        try:
            self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.server_socket.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self.server_socket.bind((self.host, self.port))
            self.server_socket.listen(5)
            
            self.is_connected = True
            self._running = True
            
            print(f"WiFi server started on {self.host}:{self.port}")
            
            # Start accepting connections
            self._accept_thread = threading.Thread(target=self._accept_connections, daemon=True)
            self._accept_thread.start()
            
            return True
            
        except Exception as e:
            print(f"Failed to start WiFi server: {e}")
            return False
            
    def _connect_client(self) -> bool:
        """Connect as WiFi client"""
        try:
            self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.socket.connect((self.host, self.port))
            
            self.is_connected = True
            self._running = True
            
            print(f"WiFi client connected to {self.host}:{self.port}")
            
            # Start receiving thread
            self._read_thread = threading.Thread(target=self._read_loop, daemon=True)
            self._read_thread.start()
            
            return True
            
        except Exception as e:
            print(f"Failed to connect WiFi client: {e}")
            return False
            
    def _accept_connections(self):
        """Accept incoming connections (server mode)"""
        while self._running:
            try:
                client_socket, client_address = self.server_socket.accept()
                client_id = f"{client_address[0]}:{client_address[1]}"
                self.connections[client_id] = client_socket
                
                print(f"New connection from {client_id}")
                
                # Start reading from this connection
                thread = threading.Thread(
                    target=self._client_read_loop,
                    args=(client_id, client_socket),
                    daemon=True
                )
                thread.start()
                
            except Exception as e:
                if self._running:
                    print(f"Connection acceptance error: {e}")
                    
    def _client_read_loop(self, client_id: str, client_socket: socket.socket):
        """Read loop for individual client connection"""
        buffer = b""
        
        while self._running and client_socket in self.connections.values():
            try:
                data = client_socket.recv(1024)
                if not data:
                    break
                    
                buffer += data
                
                # Process complete messages
                while b'\n' in buffer:
                    line, buffer = buffer.split(b'\n', 1)
                    try:
                        message_data = json.loads(line.decode('utf-8'))
                        message = Message.from_dict(message_data)
                        self._handle_message(message)
                    except (json.JSONDecodeError, UnicodeDecodeError) as e:
                        print(f"Error parsing WiFi message from {client_id}: {e}")
                        
            except Exception as e:
                print(f"WiFi read error from {client_id}: {e}")
                break
                
        # Clean up connection
        if client_id in self.connections:
            del self.connections[client_id]
        client_socket.close()
        print(f"Connection {client_id} closed")
        
    def _read_loop(self):
        """Read loop for client mode"""
        buffer = b""
        
        while self._running and self.socket:
            try:
                data = self.socket.recv(1024)
                if not data:
                    break
                    
                buffer += data
                
                # Process complete messages
                while b'\n' in buffer:
                    line, buffer = buffer.split(b'\n', 1)
                    try:
                        message_data = json.loads(line.decode('utf-8'))
                        message = Message.from_dict(message_data)
                        self._handle_message(message)
                    except (json.JSONDecodeError, UnicodeDecodeError) as e:
                        print(f"Error parsing WiFi message: {e}")
                        
            except Exception as e:
                print(f"WiFi read error: {e}")
                break
                
    def disconnect(self) -> bool:
        """Disconnect WiFi connection"""
        self._running = False
        
        # Close client connections
        for client_socket in self.connections.values():
            client_socket.close()
        self.connections.clear()
        
        # Close server socket
        if self.server_socket:
            self.server_socket.close()
            
        # Close client socket
        if self.socket:
            self.socket.close()
            
        self.is_connected = False
        print("WiFi communication disconnected")
        return True
        
    def send_message(self, message: Message) -> bool:
        """Send message over WiFi"""
        try:
            message_data = json.dumps(message.to_dict())
            message_bytes = (message_data + '\n').encode('utf-8')
            
            if self.role == 'server':
                # Send to specific client or broadcast
                if message.receiver_id:
                    if message.receiver_id in self.connections:
                        self.connections[message.receiver_id].sendall(message_bytes)
                        return True
                else:
                    # Broadcast to all clients
                    for client_socket in self.connections.values():
                        client_socket.sendall(message_bytes)
                    return True
            else:
                # Client mode - send to server
                if self.socket:
                    self.socket.sendall(message_bytes)
                    return True
                    
            return False
            
        except Exception as e:
            print(f"WiFi send error: {e}")
            return False
            
    def receive_message(self) -> Optional[Message]:
        """Receive message - in this implementation, messages are handled via callbacks"""
        # In WiFi mode, messages are typically handled asynchronously
        return None


class WebSocketCommunication(BaseCommunication):
    """WebSocket communication interface for modern web-based robot control"""
    
    def __init__(self, config: Optional[Dict] = None):
        super().__init__(config)
        self.uri = config.get('uri', 'ws://localhost:8080') if config else 'ws://localhost:8080'
        self.role = config.get('role', 'client') if config else 'client'
        
        self.websocket = None
        self.server = None
        self.clients = {}
        self._running = False
        self._main_loop_task = None
        
    async def connect(self) -> bool:
        """Connect via WebSocket"""
        try:
            if self.role == 'server':
                return await self._start_server()
            else:
                return await self._connect_client()
                
        except Exception as e:
            print(f"WebSocket connection error: {e}")
            return False
            
    async def _start_server(self) -> bool:
        """Start WebSocket server"""
        async def handle_client(websocket, path):
            client_id = f"client_{len(self.clients)}"
            self.clients[client_id] = websocket
            print(f"New WebSocket client: {client_id}")
            
            try:
                async for message in websocket:
                    try:
                        message_data = json.loads(message)
                        received_message = Message.from_dict(message_data)
                        await self._handle_message_async(received_message)
                    except json.JSONDecodeError:
                        print("Invalid WebSocket message format")
                        
            except websockets.exceptions.ConnectionClosed:
                print(f"WebSocket client {client_id} disconnected")
            finally:
                if client_id in self.clients:
                    del self.clients[client_id]
                    
        self.server = await websockets.serve(handle_client, "localhost", 8080)
        self._running = True
        print("WebSocket server started on ws://localhost:8080")
        return True
        
    async def _connect_client(self) -> bool:
        """Connect as WebSocket client"""
        self.websocket = await websockets.connect(self.uri)
        self._running = True
        print(f"WebSocket client connected to {self.uri}")
        
        # Start receiving messages
        asyncio.create_task(self._receive_loop())
        return True
        
    async def _receive_loop(self):
        """Receive messages asynchronously"""
        try:
            async for message in self.websocket:
                try:
                    message_data = json.loads(message)
                    received_message = Message.from_dict(message_data)
                    await self._handle_message_async(received_message)
                except json.JSONDecodeError:
                    print("Invalid WebSocket message format")
                    
        except websockets.exceptions.ConnectionClosed:
            print("WebSocket connection closed")
            
    async def _handle_message_async(self, message: Message):
        """Handle received message asynchronously"""
        # Create event loop if needed
        try:
            loop = asyncio.get_event_loop()
        except RuntimeError:
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            
        # Execute handlers in thread pool
        if message.message_type in self.message_handlers:
            for handler in self.message_handlers[message.message_type]:
                if asyncio.iscoroutinefunction(handler):
                    await handler(message)
                else:
                    loop.run_in_executor(None, handler, message)
                    
    def disconnect(self) -> bool:
        """Disconnect WebSocket"""
        self._running = False
        
        if self.server:
            self.server.close()
            
        if self.websocket:
            asyncio.run(self.websocket.close())
            
        self.is_connected = False
        print("WebSocket communication disconnected")
        return True
        
    async def send_message(self, message: Message) -> bool:
        """Send message over WebSocket"""
        try:
            message_data = json.dumps(message.to_dict())
            
            if self.role == 'server':
                # Broadcast to all clients
                if self.clients:
                    disconnected = []
                    for client_id, websocket in self.clients.items():
                        try:
                            await websocket.send(message_data)
                        except websockets.exceptions.ConnectionClosed:
                            disconnected.append(client_id)
                            
                    # Clean up disconnected clients
                    for client_id in disconnected:
                        del self.clients[client_id]
                        
                    return len(disconnected) < len(self.clients)
            else:
                # Send to server
                if self.websocket:
                    await self.websocket.send(message_data)
                    return True
                    
            return False
            
        except Exception as e:
            print(f"WebSocket send error: {e}")
            return False
            
    def receive_message(self) -> Optional[Message]:
        """Receive message - in WebSocket mode, messages are handled asynchronously"""
        return None


class RobotNetwork:
    """High-level robot networking interface"""
    
    def __init__(self, robot_id: str):
        self.robot_id = robot_id
        self.communication_channels = {}
        self.message_handlers = {}
        self._running = False
        
    def add_communication_channel(self, name: str, communication: BaseCommunication):
        """Add communication channel"""
        self.communication_channels[name] = communication
        
        # Register default message handlers
        communication.register_handler('ping', self._handle_ping)
        communication.register_handler('command', self._handle_command)
        communication.register_handler('data', self._handle_data)
        
    async def start_network(self):
        """Start all communication channels"""
        self._running = True
        
        # Connect all channels
        for name, comm in self.communication_channels.items():
            try:
                if hasattr(comm, 'connect'):
                    result = comm.connect()
                    if result:
                        print(f"Started communication channel: {name}")
                    else:
                        print(f"Failed to start communication channel: {name}")
                elif hasattr(comm, 'connect') and asyncio.iscoroutinefunction(comm.connect):
                    result = await comm.connect()
                    if result:
                        print(f"Started communication channel: {name}")
                    else:
                        print(f"Failed to start communication channel: {name}")
            except Exception as e:
                print(f"Error starting channel {name}: {e}")
                
    def stop_network(self):
        """Stop all communication channels"""
        self._running = False
        
        for name, comm in self.communication_channels.items():
            try:
                comm.disconnect()
                print(f"Stopped communication channel: {name}")
            except Exception as e:
                print(f"Error stopping channel {name}: {e}")
                
    def send_to_all(self, message: Message):
        """Send message through all communication channels"""
        for comm in self.communication_channels.values():
            try:
                comm.send_message(message)
            except Exception as e:
                print(f"Error sending message via {type(comm).__name__}: {e}")
                
    def broadcast_command(self, command: str, parameters: Dict):
        """Broadcast command to all connected robots"""
        message = Message(
            sender_id=self.robot_id,
            message_type="command",
            data={"command": command, "parameters": parameters},
            priority=3
        )
        self.send_to_all(message)
        
    def _handle_ping(self, message: Message):
        """Handle ping messages"""
        response = Message(
            sender_id=self.robot_id,
            receiver_id=message.sender_id,
            message_type="pong",
            data={"timestamp": time.time()},
            priority=1
        )
        
        # Send response through appropriate channel
        for comm in self.communication_channels.values():
            comm.send_message(response)
            
    def _handle_command(self, message: Message):
        """Handle command messages"""
        # This would integrate with robot control system
        command_data = message.data
        print(f"Received command: {command_data}")
        
        # Send acknowledgment
        response = Message(
            sender_id=self.robot_id,
            receiver_id=message.sender_id,
            message_type="ack",
            data={"command": command_data, "status": "received"},
            priority=2
        )
        
        for comm in self.communication_channels.values():
            comm.send_message(response)
            
    def _handle_data(self, message: Message):
        """Handle data messages"""
        print(f"Received data from {message.sender_id}: {message.data}")
        
    def get_network_status(self) -> Dict[str, Any]:
        """Get network status for all channels"""
        status = {}
        
        for name, comm in self.communication_channels.items():
            status[name] = {
                'type': type(comm).__name__,
                'connected': comm.is_connected,
                'config': comm.config
            }
            
        return status


# Example usage and testing
if __name__ == "__main__":
    print("Testing Robot Communication Framework...")
    
    # Test Serial Communication
    print("\n1. Testing Serial Communication...")
    serial_comm = SerialCommunication({
        'port': '/dev/ttyUSB0',
        'baudrate': 115200,
        'timeout': 1.0
    })
    
    if serial_comm.connect():
        # Test message sending
        test_message = Message(
            sender_id="test_sender",
            receiver_id="test_receiver",
            message_type="command",
            data={"move": "forward", "speed": 0.5}
        )
        serial_comm.send_message(test_message)
        
        # Test receiving
        for i in range(10):
            received = serial_comm.receive_message()
            if received:
                print(f"Received: {received.message_type} from {received.sender_id}")
            time.sleep(0.5)
            
        serial_comm.disconnect()
        
    # Test WiFi Communication
    print("\n2. Testing WiFi Communication...")
    wifi_server = WiFiCommunication({
        'role': 'server',
        'host': 'localhost',
        'port': 8080
    })
    
    wifi_client = WiFiCommunication({
        'role': 'client',
        'host': 'localhost',
        'port': 8080
    })
    
    # Start server
    if wifi_server.connect():
        time.sleep(1)  # Give server time to start
        
        # Connect client
        if wifi_client.connect():
            # Test messaging
            client_message = Message(
                sender_id="client",
                receiver_id="server",
                message_type="data",
                data={"sensor_reading": "temperature: 25C"}
            )
            wifi_client.send_message(client_message)
            
            time.sleep(2)  # Allow time for message processing
            
            # Cleanup
            wifi_client.disconnect()
        wifi_server.disconnect()
        
    # Test Robot Network
    print("\n3. Testing Robot Network...")
    
    async def test_network():
        network = RobotNetwork("test_robot_1")
        
        # Add communication channels
        # network.add_communication_channel("wifi", wifi_client)
        # network.add_communication_channel("serial", serial_comm)
        
        # Start network
        await network.start_network()
        
        # Broadcast command
        network.broadcast_command("update_firmware", {"version": "1.0.1"})
        
        # Wait for responses
        await asyncio.sleep(2)
        
        # Check status
        status = network.get_network_status()
        print(f"Network status: {status}")
        
        # Stop network
        network.stop_network()
        
    # Run async test
    try:
        asyncio.run(test_network())
    except Exception as e:
        print(f"Network test error: {e}")
        
    print("\nCommunication framework testing complete!")
