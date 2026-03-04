use ember_core::world::World;
use std::collections::HashMap;

use crate::context::RenderContext;

/// A node in the render graph. Each node performs a specific rendering operation.
pub trait RenderNode: Send + Sync {
    /// Execute this render pass, encoding GPU commands.
    fn run(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        world: &World,
    );
}

/// A directed acyclic graph of render nodes. Nodes are executed in topological order.
pub struct RenderGraph {
    nodes: HashMap<String, Box<dyn RenderNode>>,
    /// Edges: (from, to) — `from` must execute before `to`.
    edges: Vec<(String, String)>,
    /// Cached execution order after topological sort.
    execution_order: Vec<String>,
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            execution_order: Vec::new(),
        }
    }

    /// Add a named render node to the graph.
    pub fn add_node(&mut self, name: &str, node: impl RenderNode + 'static) {
        self.nodes.insert(name.to_string(), Box::new(node));
        self.rebuild_order();
    }

    /// Add a dependency edge: `from` must execute before `to`.
    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.push((from.to_string(), to.to_string()));
        self.rebuild_order();
    }

    /// Topological sort using Kahn's algorithm. Panics on cycles.
    fn rebuild_order(&mut self) {
        let node_names: Vec<String> = self.nodes.keys().cloned().collect();

        // Build adjacency list and in-degree count
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for name in &node_names {
            in_degree.insert(name.as_str(), 0);
            adjacency.insert(name.as_str(), Vec::new());
        }

        for (from, to) in &self.edges {
            // Only consider edges where both nodes exist
            if self.nodes.contains_key(from) && self.nodes.contains_key(to) {
                adjacency
                    .get_mut(from.as_str())
                    .unwrap()
                    .push(to.as_str());
                *in_degree.get_mut(to.as_str()).unwrap() += 1;
            }
        }

        // Start with nodes that have no incoming edges
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&name, _)| name)
            .collect();
        queue.sort(); // Deterministic ordering

        let mut order: Vec<String> = Vec::new();

        while let Some(node) = queue.pop() {
            order.push(node.to_string());

            if let Some(neighbors) = adjacency.get(node) {
                let mut next_nodes: Vec<&str> = Vec::new();
                for &neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        next_nodes.push(neighbor);
                    }
                }
                next_nodes.sort();
                // Push in reverse order so that sorted order is maintained when popping
                for n in next_nodes.into_iter().rev() {
                    queue.push(n);
                }
            }
        }

        if order.len() != node_names.len() {
            panic!(
                "Render graph has a cycle! Sorted {} of {} nodes.",
                order.len(),
                node_names.len()
            );
        }

        self.execution_order = order;
    }

    /// Execute all nodes in topological order.
    pub fn execute(&self, ctx: &RenderContext, view: &wgpu::TextureView, world: &World) {
        for name in &self.execution_order {
            if let Some(node) = self.nodes.get(name) {
                node.run(&ctx.device, &ctx.queue, view, world);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyNode;

    impl RenderNode for DummyNode {
        fn run(
            &self,
            _device: &wgpu::Device,
            _queue: &wgpu::Queue,
            _view: &wgpu::TextureView,
            _world: &World,
        ) {
            // No-op for testing
        }
    }

    #[test]
    fn topological_sort_linear() {
        let mut graph = RenderGraph::new();
        graph.add_node("clear", DummyNode);
        graph.add_node("sprites", DummyNode);
        graph.add_node("present", DummyNode);
        graph.add_edge("clear", "sprites");
        graph.add_edge("sprites", "present");

        assert_eq!(graph.execution_order, vec!["clear", "sprites", "present"]);
    }

    #[test]
    fn topological_sort_diamond() {
        let mut graph = RenderGraph::new();
        graph.add_node("a", DummyNode);
        graph.add_node("b", DummyNode);
        graph.add_node("c", DummyNode);
        graph.add_node("d", DummyNode);
        graph.add_edge("a", "b");
        graph.add_edge("a", "c");
        graph.add_edge("b", "d");
        graph.add_edge("c", "d");

        // a must come first, d must come last, b and c can be either order
        assert_eq!(graph.execution_order[0], "a");
        assert_eq!(graph.execution_order[3], "d");
    }

    #[test]
    #[should_panic(expected = "cycle")]
    fn topological_sort_cycle_detection() {
        let mut graph = RenderGraph::new();
        graph.add_node("a", DummyNode);
        graph.add_node("b", DummyNode);
        graph.add_edge("a", "b");
        graph.add_edge("b", "a"); // cycle!
    }
}

