//! Message Passing Interface (MPI) compatibility layer for education
//!
//! This module provides an educational implementation of MPI concepts,
//! making parallel programming concepts more accessible to students.

use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, mpsc, oneshot, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cluster::{Cluster, NodeId};
use crate::common::NodeMetrics;

/// MPI rank identifier (equivalent to process ID)
pub type Rank = usize;

/// Number of processes in MPI communication
pub type Size = usize;

/// MPI communication tags
pub type Tag = i32;

/// Standard MPI tags
pub const MPI_ANY_TAG: Tag = -1;
pub const MPI_TAG_UB: Tag = 32767;

/// Error types specific to MPI operations
#[derive(thiserror::Error, Debug)]
pub enum MPIError {
    #[error("Invalid rank: {0} (valid range: 0..{1})")]
    InvalidRank(Rank, Size),
    
    #[error("Communication timeout after {0:?}")]
    Timeout(Duration),
    
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    
    #[error("Buffer too small: required {0}, available {1}")]
    BufferTooSmall(usize, usize),
    
    #[error("No matching message found")]
    NoMessage,
    
    #[error("Communication failed: {0}")]
    Communication(String),
}

/// MPI status structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub source: Rank,
    pub tag: Tag,
    pub count: usize,
    pub error: Option<MPIError>,
}

/// MPI data types (simplified for education)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MPIType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Char,
    Bool,
}

/// MPI communicator - manages process groups and communication
#[derive(Debug, Clone)]
pub struct Communicator {
    pub id: Uuid,
    pub name: String,
    pub rank: Rank,
    pub size: Size,
    pub processes: HashMap<Rank, NodeId>,
    pub created_at: SystemTime,
}

/// MPI request handle for non-blocking operations
#[derive(Debug)]
pub struct Request {
    pub id: Uuid,
    pub source_rank: Rank,
    pub target_rank: Option<Rank>,
    pub tag: Tag,
    pub operation_type: MPIOperationType,
    pub buffer: Option<Vec<u8>>,
    pub callback: Option<oneshot::Sender<Result<Status>>>,
    pub start_time: SystemTime,
}

/// Types of MPI operations
#[derive(Debug, Clone)]
pub enum MPIOperationType {
    Send,
    Receive,
    Broadcast,
    Scatter,
    Gather,
    Reduce,
    AllReduce,
    Barrier,
}

/// MPI operations
#[derive(Debug)]
pub enum MPIOperation {
    Send {
        data: Vec<u8>,
        dest: Rank,
        tag: Tag,
    },
    Receive {
        source: Rank,
        tag: Tag,
        buffer: Vec<u8>,
    },
    Broadcast {
        data: Vec<u8>,
        root: Rank,
    },
    Scatter {
        sendbuf: Vec<u8>,
        recvbuf: Vec<u8>,
        root: Rank,
    },
    Gather {
        sendbuf: Vec<u8>,
        recvbuf: Vec<u8>,
        root: Rank,
    },
    Reduce {
        sendbuf: Vec<u8>,
        recvbuf: Vec<u8>,
        root: Rank,
        op: ReductionOp,
    },
    AllReduce {
        sendbuf: Vec<u8>,
        recvbuf: Vec<u8>,
        op: ReductionOp,
    },
    Barrier,
}

/// Reduction operations for collective communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReductionOp {
    Sum,
    Product,
    Max,
    Min,
    BitwiseAnd,
    BitwiseOr,
    LogicalAnd,
    LogicalOr,
}

/// MPI barrier synchronization
pub struct Barrier {
    communicator_id: Uuid,
    participating_ranks: Vec<Rank>,
    arrival_times: HashMap<Rank, SystemTime>,
    coordinator: Rank,
}

/// Main MPI runtime environment
pub struct MPITimeEnvironment {
    communicator: Communicator,
    cluster: Arc<Cluster>,
    
    // Communication infrastructure
    message_queue: Arc<RwLock<VecDeque<MPIOperation>>>,
    active_requests: Arc<RwLock<HashMap<Uuid, Request>>>,
    completed_requests: Arc<RwLock<VecDeque<Request>>>,
    
    // Collective operation state
    barriers: Arc<RwLock<HashMap<Uuid, Barrier>>>,
    collectives: Arc<RwLock<HashMap<Uuid, CollectiveOperation>>>,
    
    // Statistics and monitoring
    statistics: Arc<RwLock<MPIStatistics>>,
}

/// Collective operation state
#[derive(Debug, Clone)]
pub struct CollectiveOperation {
    pub id: Uuid,
    pub operation_type: MPIOperationType,
    pub root: Rank,
    pub participating_ranks: Vec<Rank>,
    pub data: HashMap<Rank, Vec<u8>>,
    pub start_time: SystemTime,
    pub timeout: Duration,
}

/// MPI runtime statistics
#[derive(Debug, Clone)]
pub struct MPIStatistics {
    pub total_messages_sent: u64,
    pub total_bytes_sent: u64,
    pub total_operations: u64,
    pub average_latency: Duration,
    pub barrier_count: u64,
    pub collective_count: u64,
    pub point_to_point_count: u64,
}

impl Default for MPIStatistics {
    fn default() -> Self {
        Self {
            total_messages_sent: 0,
            total_bytes_sent: 0,
            total_operations: 0,
            average_latency: Duration::from_secs(0),
            barrier_count: 0,
            collective_count: 0,
            point_to_point_count: 0,
        }
    }
}

/// Simplified MPI interface for educational use
#[async_trait]
pub trait MPISim {
    /// Initialize MPI environment
    fn init() -> Result<Self>
    where
        Self: Sized;
    
    /// Get current process rank
    fn rank(&self) -> Rank;
    
    /// Get total number of processes
    fn size(&self) -> Size;
    
    /// Finalize MPI environment
    async fn finalize(&self) -> Result<()>;
    
    /// Send data to another process
    async fn send(&self, data: &[u8], dest: Rank, tag: Tag) -> Result<()>;
    
    /// Receive data from another process
    async fn recv(&self, data: &mut [u8], source: Rank, tag: Tag) -> Result<Status>;
    
    /// Non-blocking send
    async fn send_init(&self, data: &[u8], dest: Rank, tag: Tag) -> Result<Request>;
    
    /// Non-blocking receive
    async fn recv_init(&self, data: &mut [u8], source: Rank, tag: Tag) -> Result<Request>;
    
    /// Wait for request completion
    async fn wait(&self, request: Request) -> Result<Status>;
    
    /// Test if request is complete
    async fn test(&self, request: Request) -> Result<(bool, Status)>;
    
    /// Synchronization barrier
    async fn barrier(&self) -> Result<()>;
    
    /// Broadcast data from root to all processes
    async fn broadcast(&self, data: &mut [u8], root: Rank) -> Result<()>;
    
    /// Scatter data from root to all processes
    async fn scatter(&self, sendbuf: &[u8], recvbuf: &mut [u8], root: Rank) -> Result<()>;
    
    /// Gather data from all processes to root
    async fn gather(&self, sendbuf: &[u8], recvbuf: &mut [u8], root: Rank) -> Result<()>;
    
    /// Reduce operation with result to all processes
    async fn all_reduce(&self, sendbuf: &[u8], recvbuf: &mut [u8], op: ReductionOp) -> Result<()>;
}

/// Educational MPI implementation
pub struct EducationalMPI {
    env: Arc<MPITimeEnvironment>,
}

#[async_trait]
impl MPISim for EducationalMPI {
    fn init() -> Result<Self> {
        info!("Initializing educational MPI environment");
        
        // Create a default communicator for educational purposes
        let communicator = Communicator {
            id: Uuid::new_v4(),
            name: "EDU_WORLD".to_string(),
            rank: 0, // Will be set during cluster initialization
            size: 1,
            processes: HashMap::new(),
            created_at: SystemTime::now(),
        };
        
        let cluster = Arc::new(crate::cluster::Cluster::new(&crate::cluster::ClusterConfig::default()).await?);
        
        let env = Arc::new(MPITimeEnvironment {
            communicator,
            cluster,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            completed_requests: Arc::new(RwLock::new(VecDeque::new())),
            barriers: Arc::new(RwLock::new(HashMap::new())),
            collectives: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(MPIStatistics::default())),
        });
        
        Ok(Self { env })
    }
    
    fn rank(&self) -> Rank {
        self.env.communicator.rank
    }
    
    fn size(&self) -> Size {
        self.env.communicator.size
    }
    
    async fn finalize(&self) -> Result<()> {
        info!("Finalizing educational MPI environment");
        
        // Wait for all pending operations to complete
        let mut active_requests = self.env.active_requests.write().await;
        while !active_requests.is_empty() {
            let request = active_requests.values().next().unwrap().clone();
            let _ = self.wait(request).await;
            active_requests.remove(&request.id);
        }
        
        info!("MPI environment finalized");
        Ok(())
    }
    
    async fn send(&self, data: &[u8], dest: Rank, tag: Tag) -> Result<()> {
        debug!("MPI Send: rank {} -> rank {} (tag: {}, {} bytes)", 
               self.rank(), dest, tag, data.len());
        
        if dest >= self.size() {
            return Err(MPIError::InvalidRank(dest, self.size()).into());
        }
        
        // Create MPI operation
        let operation = MPIOperation::Send {
            data: data.to_vec(),
            dest,
            tag,
        };
        
        // Add to message queue
        {
            let mut queue = self.env.message_queue.write().await;
            queue.push_back(operation);
        }
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.total_messages_sent += 1;
            stats.total_bytes_sent += data.len() as u64;
            stats.point_to_point_count += 1;
        }
        
        // Simulate network transmission delay
        tokio::time::sleep(Duration::from_millis(1)).await;
        
        Ok(())
    }
    
    async fn recv(&self, data: &mut [u8], source: Rank, tag: Tag) -> Result<Status> {
        debug!("MPI Receive: rank {} from rank {} (tag: {})", 
               self.rank(), source, tag);
        
        if source >= self.size() && source != Rank::MAX {
            return Err(MPIError::InvalidRank(source, self.size()).into());
        }
        
        // Simulate message reception
        let received_data = vec![0u8; data.len()];
        
        if received_data.len() > data.len() {
            return Err(MPIError::BufferTooSmall(received_data.len(), data.len()).into());
        }
        
        data[..received_data.len()].copy_from_slice(&received_data);
        
        Ok(Status {
            source: if source == Rank::MAX { 0 } else { source },
            tag,
            count: received_data.len(),
            error: None,
        })
    }
    
    async fn send_init(&self, data: &[u8], dest: Rank, tag: Tag) -> Result<Request> {
        let request = Request {
            id: Uuid::new_v4(),
            source_rank: self.rank(),
            target_rank: Some(dest),
            tag,
            operation_type: MPIOperationType::Send,
            buffer: Some(data.to_vec()),
            callback: None,
            start_time: SystemTime::now(),
        };
        
        // Register active request
        {
            let mut active = self.env.active_requests.write().await;
            active.insert(request.id, request.clone());
        }
        
        // Start non-blocking send
        let env = self.env.clone();
        let request_clone = request.clone();
        tokio::spawn(async move {
            let _ = env.send(&data, dest, tag).await;
            
            // Mark request as completed
            let mut active = env.active_requests.write().await;
            active.remove(&request_clone.id);
            
            let mut completed = env.completed_requests.write().await;
            completed.push_back(request_clone);
        });
        
        Ok(request)
    }
    
    async fn recv_init(&self, data: &mut [u8], source: Rank, tag: Tag) -> Result<Request> {
        let request = Request {
            id: Uuid::new_v4(),
            source_rank: self.rank(),
            target_rank: Some(source),
            tag,
            operation_type: MPIOperationType::Receive,
            buffer: Some(data.to_vec()),
            callback: None,
            start_time: SystemTime::now(),
        };
        
        // Register active request
        {
            let mut active = self.env.active_requests.write().await;
            active.insert(request.id, request.clone());
        }
        
        Ok(request)
    }
    
    async fn wait(&self, request: Request) -> Result<Status> {
        debug!("MPI Wait: requesting completion for request {}", request.id);
        
        // Check if request is already completed
        {
            let completed = self.env.completed_requests.read().await;
            if completed.iter().any(|r| r.id == request.id) {
                return Ok(Status {
                    source: request.source_rank,
                    tag: request.tag,
                    count: request.buffer.as_ref().map(|b| b.len()).unwrap_or(0),
                    error: None,
                });
            }
        }
        
        // Wait for completion with timeout
        let timeout_duration = Duration::from_secs(30);
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout_duration {
            {
                let completed = self.env.completed_requests.read().await;
                if completed.iter().any(|r| r.id == request.id) {
                    return Ok(Status {
                        source: request.source_rank,
                        tag: request.tag,
                        count: request.buffer.as_ref().map(|b| b.len()).unwrap_or(0),
                        error: None,
                    });
                }
            }
            
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        
        Err(MPIError::Timeout(timeout_duration).into())
    }
    
    async fn test(&self, request: Request) -> Result<(bool, Status)> {
        {
            let completed = self.env.completed_requests.read().await;
            if completed.iter().any(|r| r.id == request.id) {
                return Ok((true, Status {
                    source: request.source_rank,
                    tag: request.tag,
                    count: request.buffer.as_ref().map(|b| b.len()).unwrap_or(0),
                    error: None,
                }));
            }
        }
        
        Ok((false, Status {
            source: request.source_rank,
            tag: request.tag,
            count: request.buffer.as_ref().map(|b| b.len()).unwrap_or(0),
            error: None,
        }))
    }
    
    async fn barrier(&self) -> Result<()> {
        debug!("MPI Barrier: rank {} entering barrier", self.rank());
        
        let barrier_id = self.env.communicator.id;
        let my_rank = self.rank();
        let total_ranks = self.size();
        
        // Check if barrier already exists
        {
            let barriers = self.env.barriers.read().await;
            if let Some(barrier) = barriers.get(&barrier_id) {
                // Join existing barrier
                let mut barriers_mut = self.env.barriers.write().await;
                if let Some(existing) = barriers_mut.get_mut(&barrier_id) {
                    existing.arrival_times.insert(my_rank, SystemTime::now());
                    
                    // Check if all processes have arrived
                    if existing.arrival_times.len() == total_ranks {
                        info!("All processes arrived at barrier, releasing");
                        barriers_mut.remove(&barrier_id);
                        return Ok(());
                    }
                }
            }
        }
        
        // Create new barrier
        let barrier = Barrier {
            communicator_id: barrier_id,
            participating_ranks: (0..total_ranks).collect(),
            arrival_times: HashMap::new(),
            coordinator: 0,
        };
        
        barrier.arrival_times.insert(my_rank, SystemTime::now());
        
        {
            let mut barriers = self.env.barriers.write().await;
            barriers.insert(barrier_id, barrier);
        }
        
        // Wait for other processes (simplified simulation)
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.barrier_count += 1;
        }
        
        info!("MPI Barrier: rank {} exited barrier", self.rank());
        Ok(())
    }
    
    async fn broadcast(&self, data: &mut [u8], root: Rank) -> Result<()> {
        debug!("MPI Broadcast: rank {} broadcasting {} bytes from root {}", 
               self.rank(), data.len(), root);
        
        if root >= self.size() {
            return Err(MPIError::InvalidRank(root, self.size()).into());
        }
        
        if self.rank() == root {
            // This process is the root - send data to all others
            for rank in 0..self.size() {
                if rank != root {
                    self.send(data, rank, 0).await?;
                }
            }
        } else {
            // This process is a receiver - get data from root
            let mut recv_data = vec![0u8; data.len()];
            let status = self.recv(&mut recv_data, root, 0).await?;
            
            if status.count <= data.len() {
                data[..status.count].copy_from_slice(&recv_data[..status.count]);
            }
        }
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.collective_count += 1;
            if self.rank() == root {
                stats.total_bytes_sent += (data.len() * (self.size() - 1)) as u64;
            }
        }
        
        Ok(())
    }
    
    async fn scatter(&self, sendbuf: &[u8], recvbuf: &mut [u8], root: Rank) -> Result<()> {
        debug!("MPI Scatter: rank {} scattering from root {}", self.rank(), root);
        
        if root >= self.size() {
            return Err(MPIError::InvalidRank(root, self.size()).into());
        }
        
        if self.rank() == root {
            // Root process distributes data
            let chunk_size = sendbuf.len() / self.size();
            let remaining = sendbuf.len() % self.size();
            
            let mut offset = 0;
            for rank in 0..self.size() {
                let mut chunk_size = chunk_size;
                if rank < remaining {
                    chunk_size += 1;
                }
                
                let chunk = &sendbuf[offset..offset + chunk_size];
                
                if rank != root {
                    self.send(chunk, rank, 0).await?;
                } else {
                    // Copy to own buffer
                    if recvbuf.len() >= chunk.len() {
                        recvbuf[..chunk.len()].copy_from_slice(chunk);
                    }
                }
                
                offset += chunk_size;
            }
        } else {
            // Non-root processes receive their portion
            let chunk_size = sendbuf.len() / self.size();
            let remaining = sendbuf.len() % self.size();
            let my_chunk_size = chunk_size + if self.rank() < remaining { 1 } else { 0 };
            
            let mut recv_data = vec![0u8; my_chunk_size];
            let status = self.recv(&mut recv_data, root, 0).await?;
            
            if status.count <= recvbuf.len() {
                recvbuf[..status.count].copy_from_slice(&recv_data[..status.count]);
            }
        }
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.collective_count += 1;
        }
        
        Ok(())
    }
    
    async fn gather(&self, sendbuf: &[u8], recvbuf: &mut [u8], root: Rank) -> Result<()> {
        debug!("MPI Gather: rank {} gathering to root {}", self.rank(), root);
        
        if root >= self.size() {
            return Err(MPIError::InvalidRank(root, self.size()).into());
        }
        
        if self.rank() == root {
            // Root process collects data from all processes
            let chunk_size = sendbuf.len() / self.size();
            
            // Copy own data first
            recvbuf[..sendbuf.len()].copy_from_slice(sendbuf);
            
            // Receive data from other processes
            for rank in 0..self.size() {
                if rank != root {
                    let mut chunk = vec![0u8; chunk_size];
                    let status = self.recv(&mut chunk, rank, 0).await?;
                    
                    let offset = rank * chunk_size;
                    if offset + status.count <= recvbuf.len() {
                        recvbuf[offset..offset + status.count].copy_from_slice(&chunk[..status.count]);
                    }
                }
            }
        } else {
            // Non-root processes send their data
            self.send(sendbuf, root, 0).await?;
        }
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.collective_count += 1;
        }
        
        Ok(())
    }
    
    async fn all_reduce(&self, sendbuf: &[u8], recvbuf: &mut [u8], op: ReductionOp) -> Result<()> {
        debug!("MPI AllReduce: rank {} performing {} operation", self.rank(), format!("{:?}", op));
        
        // Simple implementation: root collects, reduces, then broadcasts
        let root = 0;
        
        if self.rank() == root {
            // Root gathers all data
            let mut gathered_data = Vec::new();
            gathered_data.extend_from_slice(sendbuf);
            
            for rank in 1..self.size() {
                let mut chunk = vec![0u8; sendbuf.len()];
                let status = self.recv(&mut chunk, rank, 0).await?;
                if status.count == sendbuf.len() {
                    gathered_data.extend_from_slice(&chunk);
                }
            }
            
            // Perform reduction (simplified)
            let result = perform_reduction(&gathered_data, &op)?;
            let result_slice = &result[..recvbuf.len()];
            recvbuf[..result_slice.len()].copy_from_slice(result_slice);
            
            // Broadcast result to all processes
            for rank in 1..self.size() {
                self.send(&result, rank, 0).await?;
            }
        } else {
            // Send data to root
            self.send(sendbuf, root, 0).await?;
            
            // Receive reduced result
            let mut result = vec![0u8; recvbuf.len()];
            let status = self.recv(&mut result, root, 0).await?;
            
            if status.count <= recvbuf.len() {
                recvbuf[..status.count].copy_from_slice(&result[..status.count]);
            }
        }
        
        // Update statistics
        {
            let mut stats = self.env.statistics.write().await;
            stats.collective_count += 1;
        }
        
        Ok(())
    }
}

/// Perform reduction operation on data
fn perform_reduction(data: &[u8], op: &ReductionOp) -> Result<Vec<u8>> {
    // This is a simplified reduction implementation
    // In a real MPI implementation, this would handle different data types
    
    match op {
        ReductionOp::Sum => {
            // Sum all values (assuming 4-byte integers)
            let mut sum = 0i32;
            for chunk in data.chunks_exact(4) {
                if let Ok(value) = chunk.try_into() {
                    sum += i32::from_le_bytes(value);
                }
            }
            Ok(sum.to_le_bytes().to_vec())
        }
        ReductionOp::Product => {
            // Product of all values
            let mut product = 1i32;
            for chunk in data.chunks_exact(4) {
                if let Ok(value) = chunk.try_into() {
                    product *= i32::from_le_bytes(value);
                }
            }
            Ok(product.to_le_bytes().to_vec())
        }
        ReductionOp::Max => {
            // Maximum value
            let mut max_val = i32::MIN;
            for chunk in data.chunks_exact(4) {
                if let Ok(value) = chunk.try_into() {
                    let val = i32::from_le_bytes(value);
                    if val > max_val {
                        max_val = val;
                    }
                }
            }
            Ok(max_val.to_le_bytes().to_vec())
        }
        ReductionOp::Min => {
            // Minimum value
            let mut min_val = i32::MAX;
            for chunk in data.chunks_exact(4) {
                if let Ok(value) = chunk.try_into() {
                    let val = i32::from_le_bytes(value);
                    if val < min_val {
                        min_val = val;
                    }
                }
            }
            Ok(min_val.to_le_bytes().to_vec())
        }
        _ => Err(MPIError::NotSupported(format!("Reduction operation {:?} not implemented", op)).into()),
    }
}

/// Factory function for creating educational MPI instances
pub fn create_mpi_environment(rank: Rank, size: Size) -> Result<EducationalMPI> {
    let mut mpi = EducationalMPI::init()?;
    
    // Update communicator with provided rank and size
    {
        let env = &mut mpi.env;
        env.communicator.rank = rank;
        env.communicator.size = size;
        
        // Create process mapping
        for r in 0..size {
            env.communicator.processes.insert(r, Uuid::new_v4());
        }
    }
    
    Ok(mpi)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_mpi_initialization() {
        let mpi = EducationalMPI::init();
        assert!(mpi.is_ok());
    }
    
    #[tokio::test]
    async fn test_mpi_send_receive() {
        let mut mpi = EducationalMPI::init().unwrap();
        
        let data = vec![1, 2, 3, 4, 5];
        let mut recv_buffer = vec![0u8; data.len()];
        
        // Send data
        let send_result = mpi.send(&data, 0, 0).await;
        assert!(send_result.is_ok());
        
        // Receive data
        let recv_result = mpi.recv(&mut recv_buffer, 0, 0).await;
        assert!(recv_result.is_ok());
        
        let status = recv_result.unwrap();
        assert_eq!(status.count, data.len());
    }
    
    #[tokio::test]
    async fn test_mpi_barrier() {
        let mpi = EducationalMPI::init().unwrap();
        let barrier_result = mpi.barrier().await;
        assert!(barrier_result.is_ok());
    }
    
    #[tokio::test]
    async fn test_mpi_broadcast() {
        let mut mpi = EducationalMPI::init().unwrap();
        let mut data = vec![1, 2, 3, 4, 5];
        
        let broadcast_result = mpi.broadcast(&mut data, 0).await;
        assert!(broadcast_result.is_ok());
    }
    
    #[test]
    fn test_reduction_operations() {
        let data = vec![1i32, 2i32, 3i32, 4i32]
            .into_iter()
            .flat_map(|i| i.to_le_bytes())
            .collect::<Vec<_>>();
        
        let sum_result = perform_reduction(&data, &ReductionOp::Sum);
        assert!(sum_result.is_ok());
        
        let sum = i32::from_le_bytes(sum_result.unwrap().try_into().unwrap());
        assert_eq!(sum, 10);
    }
    
    #[test]
    fn test_mpi_error_types() {
        let error = MPIError::InvalidRank(10, 5);
        assert!(error.to_string().contains("Invalid rank"));
        
        let error = MPIError::Timeout(Duration::from_secs(5));
        assert!(error.to_string().contains("Communication timeout"));
    }
}