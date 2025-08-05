//! DAG execution implementation

use async_trait::async_trait;
use meta_ai_common::{
    error::{Error, Result},
    types::{TaskId, TaskStatus},
};
use meta_ai_core::orchestrator::{
    DagExecutor, TaskDag, DagValidation, DagExecutionResult, DagNode, EdgeCondition,
};
use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::Topo,
    algo::is_cyclic_directed,
    Direction,
};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{info, warn, instrument};

/// DAG executor implementation
pub struct DagExecutorImpl {
    max_depth: usize,
}

impl DagExecutorImpl {
    pub fn new() -> Self {
        Self { max_depth: 10 }
    }
    
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self { max_depth }
    }
    
    /// Build petgraph from TaskDag
    fn build_graph(&self, dag: &TaskDag) -> (DiGraph<TaskId, EdgeCondition>, HashMap<TaskId, NodeIndex>) {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        
        // Add nodes
        for (task_id, _) in &dag.nodes {
            let idx = graph.add_node(*task_id);
            node_map.insert(*task_id, idx);
        }
        
        // Add edges
        for edge in &dag.edges {
            if let (Some(&from_idx), Some(&to_idx)) = (node_map.get(&edge.from), node_map.get(&edge.to)) {
                graph.add_edge(from_idx, to_idx, edge.condition.clone().unwrap_or(EdgeCondition::OnSuccess));
            }
        }
        
        (graph, node_map)
    }
    
    /// Calculate DAG depth
    fn calculate_depth(&self, graph: &DiGraph<TaskId, EdgeCondition>) -> usize {
        let mut max_depth = 0;
        let mut depths = HashMap::new();
        
        // Find root nodes (no incoming edges)
        let roots: Vec<_> = graph.node_indices()
            .filter(|&n| graph.edges_directed(n, Direction::Incoming).count() == 0)
            .collect();
        
        // BFS to calculate depths
        let mut queue = VecDeque::new();
        for root in roots {
            queue.push_back((root, 0));
            depths.insert(root, 0);
        }
        
        while let Some((node, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth);
            
            for edge in graph.edges_directed(node, Direction::Outgoing) {
                let target = edge.target();
                let new_depth = depth + 1;
                
                if !depths.contains_key(&target) || depths[&target] < new_depth {
                    depths.insert(target, new_depth);
                    queue.push_back((target, new_depth));
                }
            }
        }
        
        max_depth
    }
    
    /// Find unreachable nodes
    fn find_unreachable_nodes(&self, graph: &DiGraph<TaskId, EdgeCondition>) -> Vec<TaskId> {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        
        // Start from root nodes
        for node in graph.node_indices() {
            if graph.edges_directed(node, Direction::Incoming).count() == 0 {
                stack.push(node);
            }
        }
        
        // DFS traversal
        while let Some(node) = stack.pop() {
            if visited.insert(node) {
                for edge in graph.edges_directed(node, Direction::Outgoing) {
                    stack.push(edge.target());
                }
            }
        }
        
        // Find unvisited nodes
        graph.node_indices()
            .filter(|&n| !visited.contains(&n))
            .filter_map(|n| graph.node_weight(n).copied())
            .collect()
    }
}

#[async_trait]
impl DagExecutor for DagExecutorImpl {
    #[instrument(skip(self, dag))]
    async fn execute_dag(&self, dag: &TaskDag) -> Result<DagExecutionResult> {
        info!("Executing DAG with {} nodes", dag.nodes.len());
        
        // Validate DAG first
        let validation = self.validate_dag(dag);
        if !validation.valid {
            return Err(Error::Validation("Invalid DAG structure".to_string()));
        }
        
        let start_time = std::time::Instant::now();
        let mut completed_tasks = Vec::new();
        let mut failed_tasks = Vec::new();
        let mut skipped_tasks = Vec::new();
        
        // Get topological order
        let order = self.topological_sort(dag)?;
        
        // Execute tasks in order
        for task_id in order {
            if let Some(node) = dag.nodes.get(&task_id) {
                // Check dependencies
                let deps_satisfied = node.dependencies.iter().all(|dep_id| {
                    dag.nodes.get(dep_id)
                        .map(|dep| matches!(dep.status, TaskStatus::Completed))
                        .unwrap_or(false)
                });
                
                if deps_satisfied {
                    // Execute task (simplified - would call actual executor)
                    info!("Executing task: {}", task_id);
                    
                    // Simulate execution
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    
                    // For now, assume success
                    completed_tasks.push(task_id);
                } else {
                    warn!("Skipping task {} due to failed dependencies", task_id);
                    skipped_tasks.push(task_id);
                }
            }
        }
        
        Ok(DagExecutionResult {
            completed_tasks,
            failed_tasks,
            skipped_tasks,
            total_duration_ms: start_time.elapsed().as_millis() as u64,
        })
    }
    
    fn validate_dag(&self, dag: &TaskDag) -> DagValidation {
        let (graph, _) = self.build_graph(dag);
        
        let has_cycles = is_cyclic_directed(&graph);
        let unreachable_nodes = self.find_unreachable_nodes(&graph);
        let max_depth = self.calculate_depth(&graph);
        
        let valid = !has_cycles && unreachable_nodes.is_empty() && max_depth <= self.max_depth;
        
        DagValidation {
            valid,
            has_cycles,
            unreachable_nodes,
            max_depth,
        }
    }
    
    fn topological_sort(&self, dag: &TaskDag) -> Result<Vec<TaskId>> {
        let (graph, node_map) = self.build_graph(dag);
        
        if is_cyclic_directed(&graph) {
            return Err(Error::Validation("DAG contains cycles".to_string()));
        }
        
        let mut result = Vec::new();
        let mut topo = Topo::new(&graph);
        
        while let Some(node_idx) = topo.next(&graph) {
            if let Some(&task_id) = graph.node_weight(node_idx) {
                result.push(task_id);
            }
        }
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_dag_validation() {
        let executor = DagExecutorImpl::new();
        let mut dag = TaskDag {
            nodes: HashMap::new(),
            edges: Vec::new(),
        };
        
        // Add nodes
        let task1 = Uuid::new_v4();
        let task2 = Uuid::new_v4();
        
        dag.nodes.insert(task1, DagNode {
            task_id: task1,
            task: Default::default(),
            dependencies: vec![],
            dependents: vec![task2],
            status: TaskStatus::Pending,
        });
        
        dag.nodes.insert(task2, DagNode {
            task_id: task2,
            task: Default::default(),
            dependencies: vec![task1],
            dependents: vec![],
            status: TaskStatus::Pending,
        });
        
        // Add edge
        dag.edges.push(meta_ai_core::orchestrator::DagEdge {
            from: task1,
            to: task2,
            condition: Some(EdgeCondition::OnSuccess),
        });
        
        let validation = executor.validate_dag(&dag);
        assert!(validation.valid);
        assert!(!validation.has_cycles);
        assert!(validation.unreachable_nodes.is_empty());
    }
    
    #[tokio::test]
    async fn test_dag_execution() {
        // Test implementation
    }
}