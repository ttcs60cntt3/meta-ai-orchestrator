//! Task scheduling implementation

use async_trait::async_trait;
use meta_ai_common::{
    error::{Error, Result},
    types::{Task, TaskId, Priority},
};
use meta_ai_core::orchestrator::{TaskScheduler, QueueStats};
use priority_queue::PriorityQueue;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use parking_lot::Mutex;
use tracing::{debug, instrument};

/// Priority-based task scheduler
pub struct PriorityScheduler {
    queue: Arc<Mutex<PriorityQueue<TaskId, PriorityWrapper>>>,
    tasks: Arc<Mutex<HashMap<TaskId, ScheduledTask>>>,
    max_queue_size: usize,
    stats: Arc<Mutex<SchedulerStats>>,
}

/// Wrapper for priority to implement Ord
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PriorityWrapper {
    priority: Priority,
    timestamp: u64,
}

impl Ord for PriorityWrapper {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // First compare by priority, then by timestamp (older first)
        match self.priority.cmp(&other.priority) {
            std::cmp::Ordering::Equal => other.timestamp.cmp(&self.timestamp),
            other => other,
        }
    }
}

impl PartialOrd for PriorityWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Scheduled task with metadata
#[derive(Debug, Clone)]
struct ScheduledTask {
    task: Task,
    scheduled_at: Instant,
    attempt_count: u32,
}

/// Scheduler statistics
#[derive(Debug, Default)]
struct SchedulerStats {
    total_scheduled: u64,
    total_completed: u64,
    total_failed: u64,
    total_requeued: u64,
    cumulative_wait_time: Duration,
    cumulative_execution_time: Duration,
}

impl PriorityScheduler {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(PriorityQueue::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            max_queue_size,
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
        }
    }
    
    fn create_priority_wrapper(priority: Priority) -> PriorityWrapper {
        PriorityWrapper {
            priority,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        }
    }
}

#[async_trait]
impl TaskScheduler for PriorityScheduler {
    #[instrument(skip(self, task))]
    async fn schedule_task(&mut self, task: Task) -> Result<()> {
        let task_id = task.id;
        let priority = task.priority;
        
        // Check queue size
        {
            let queue = self.queue.lock();
            if queue.len() >= self.max_queue_size {
                return Err(Error::Internal("Task queue is full".to_string()));
            }
        }
        
        // Add to queue
        {
            let mut queue = self.queue.lock();
            let mut tasks = self.tasks.lock();
            
            let scheduled_task = ScheduledTask {
                task,
                scheduled_at: Instant::now(),
                attempt_count: 0,
            };
            
            tasks.insert(task_id, scheduled_task);
            queue.push(task_id, Self::create_priority_wrapper(priority));
            
            let mut stats = self.stats.lock();
            stats.total_scheduled += 1;
        }
        
        debug!("Scheduled task {} with priority {:?}", task_id, priority);
        Ok(())
    }
    
    #[instrument(skip(self))]
    async fn next_task(&mut self) -> Result<Option<Task>> {
        let mut queue = self.queue.lock();
        let mut tasks = self.tasks.lock();
        
        if let Some((task_id, _)) = queue.pop() {
            if let Some(mut scheduled_task) = tasks.remove(&task_id) {
                scheduled_task.attempt_count += 1;
                
                // Update stats
                let wait_time = scheduled_task.scheduled_at.elapsed();
                let mut stats = self.stats.lock();
                stats.cumulative_wait_time += wait_time;
                
                debug!("Dequeued task {} after {:?} wait", task_id, wait_time);
                return Ok(Some(scheduled_task.task));
            }
        }
        
        Ok(None)
    }
    
    #[instrument(skip(self, task))]
    async fn requeue_task(&mut self, task: Task) -> Result<()> {
        let task_id = task.id;
        let priority = task.priority;
        
        {
            let mut queue = self.queue.lock();
            let mut tasks = self.tasks.lock();
            
            // Get existing scheduled task or create new one
            let scheduled_task = tasks.entry(task_id).or_insert_with(|| {
                ScheduledTask {
                    task: task.clone(),
                    scheduled_at: Instant::now(),
                    attempt_count: 0,
                }
            });
            
            scheduled_task.task = task;
            scheduled_task.attempt_count += 1;
            
            // Re-add to queue with slightly lower priority for fairness
            queue.push(task_id, Self::create_priority_wrapper(priority));
            
            let mut stats = self.stats.lock();
            stats.total_requeued += 1;
        }
        
        debug!("Requeued task {} with priority {:?}", task_id, priority);
        Ok(())
    }
    
    async fn queue_stats(&self) -> QueueStats {
        let queue = self.queue.lock();
        let tasks = self.tasks.lock();
        let stats = self.stats.lock();
        
        let pending_tasks = queue.len();
        let running_tasks = 0; // Would need to track this separately
        
        let average_wait_time_ms = if stats.total_scheduled > 0 {
            stats.cumulative_wait_time.as_millis() as f64 / stats.total_scheduled as f64
        } else {
            0.0
        };
        
        let average_execution_time_ms = if stats.total_completed > 0 {
            stats.cumulative_execution_time.as_millis() as f64 / stats.total_completed as f64
        } else {
            0.0
        };
        
        QueueStats {
            pending_tasks,
            running_tasks,
            completed_tasks: stats.total_completed as usize,
            failed_tasks: stats.total_failed as usize,
            average_wait_time_ms,
            average_execution_time_ms,
        }
    }
}

/// Round-robin scheduler for comparison
pub struct RoundRobinScheduler {
    queue: Arc<Mutex<VecDeque<Task>>>,
    max_queue_size: usize,
    stats: Arc<Mutex<SchedulerStats>>,
}

impl RoundRobinScheduler {
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            max_queue_size,
            stats: Arc::new(Mutex::new(SchedulerStats::default())),
        }
    }
}

#[async_trait]
impl TaskScheduler for RoundRobinScheduler {
    async fn schedule_task(&mut self, task: Task) -> Result<()> {
        let mut queue = self.queue.lock();
        
        if queue.len() >= self.max_queue_size {
            return Err(Error::Internal("Task queue is full".to_string()));
        }
        
        queue.push_back(task);
        
        let mut stats = self.stats.lock();
        stats.total_scheduled += 1;
        
        Ok(())
    }
    
    async fn next_task(&mut self) -> Result<Option<Task>> {
        let mut queue = self.queue.lock();
        Ok(queue.pop_front())
    }
    
    async fn requeue_task(&mut self, task: Task) -> Result<()> {
        let mut queue = self.queue.lock();
        queue.push_back(task);
        
        let mut stats = self.stats.lock();
        stats.total_requeued += 1;
        
        Ok(())
    }
    
    async fn queue_stats(&self) -> QueueStats {
        let queue = self.queue.lock();
        let stats = self.stats.lock();
        
        QueueStats {
            pending_tasks: queue.len(),
            running_tasks: 0,
            completed_tasks: stats.total_completed as usize,
            failed_tasks: stats.total_failed as usize,
            average_wait_time_ms: 0.0,
            average_execution_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[tokio::test]
    async fn test_priority_scheduling() {
        let mut scheduler = PriorityScheduler::new(100);
        
        // Create tasks with different priorities
        let high_priority_task = Task {
            id: Uuid::new_v4(),
            name: "High priority".to_string(),
            priority: Priority::High,
            ..Default::default()
        };
        
        let low_priority_task = Task {
            id: Uuid::new_v4(),
            name: "Low priority".to_string(),
            priority: Priority::Low,
            ..Default::default()
        };
        
        // Schedule tasks
        scheduler.schedule_task(low_priority_task.clone()).await.unwrap();
        scheduler.schedule_task(high_priority_task.clone()).await.unwrap();
        
        // High priority should come first
        let next = scheduler.next_task().await.unwrap().unwrap();
        assert_eq!(next.id, high_priority_task.id);
        
        let next = scheduler.next_task().await.unwrap().unwrap();
        assert_eq!(next.id, low_priority_task.id);
    }
}

use std::collections::VecDeque;