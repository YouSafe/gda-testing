use std::{cmp::Ordering, collections::HashMap};

use serde::de::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub id: usize,
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Point {
    pub id: usize,
    pub x: u32,
    pub y: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub source: usize,
    pub target: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SimpleEdge {
    pub source: (u32, u32),
    pub target: (u32, u32),
}

pub struct CrossingCountingResult {
    pub total: u32,
    pub max_per_edge: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub nodes: Vec<Node>,

    #[serde(default)]
    pub points: Vec<Point>,

    pub edges: Vec<Edge>,

    #[serde(default = "Graph::default_dimension")]
    pub width: u32,

    #[serde(default = "Graph::default_dimension")]
    pub height: u32,
}

impl Graph {
    fn default_dimension() -> u32 {
        1_000_000
    }

    pub fn crossings(&self) -> CrossingCountingResult {
        let edges = self
            .edges
            .iter()
            .map(|edge| {
                let source = &self.nodes[edge.source];
                let target = &self.nodes[edge.target];

                SimpleEdge {
                    source: (source.x, source.y),
                    target: (target.x, target.y),
                }
            })
            .collect::<Vec<_>>();

        let mut total_crossings = 0;
        let mut crossings_per_edge = vec![0u32; self.edges.len()];

        for (i1, e1) in edges.iter().enumerate() {
            let mut crossings = 0;
            for (i2, e2) in edges.iter().enumerate().skip(i1 + 1) {
                if is_crossing(e1.source, e1.target, e2.source, e2.target) {
                    crossings_per_edge[i1] += 1;
                    crossings_per_edge[i2] += 1;
                    crossings += 1;
                }
            }

            total_crossings += crossings;
        }

        CrossingCountingResult {
            total: total_crossings,
            max_per_edge: crossings_per_edge.into_iter().max().unwrap_or_default(),
        }
    }

    pub fn is_valid(&self) -> Result<(), serde_json::error::Error> {
        let num_nodes = self.nodes.len();

        type NodeID = usize;
        type Coordinate = (u32, u32);

        let mut coordinates: HashMap<Coordinate, NodeID> = HashMap::with_capacity(num_nodes);

        for node in &self.nodes {
            if node.id >= num_nodes {
                return Err(serde_json::Error::custom(format!(
                    "Node ID {} is out of bounds (0 to {})",
                    node.id,
                    num_nodes - 1
                )));
            }
            if node.x > self.width {
                return Err(serde_json::Error::custom(format!(
                    "Node x-coordinate {} exceeds maximum width {}",
                    node.x, self.width
                )));
            }
            if node.y > self.height {
                return Err(serde_json::Error::custom(format!(
                    "Node y-coordinate {} exceeds maximum height {}",
                    node.y, self.height
                )));
            }

            if let Some(duplicate_id) = coordinates.get(&(node.x, node.y)) {
                return Err(serde_json::Error::custom(format!(
                    "Node with ID {} overlaps with node with ID {} at coordinates ({}, {})",
                    duplicate_id, node.id, node.x, node.y
                )));
            }

            coordinates.insert((node.x, node.y), node.id);
        }

        // To deal with a nodes array that is not sorted by the id
        let mut id_to_idx = vec![None; num_nodes];
        let mut nodes_defined = 0;
        for (idx, node) in self.nodes.iter().enumerate() {
            if id_to_idx[node.id].is_some() {
                return Err(serde_json::Error::custom(format!(
                    "Node {} is defined more than once",
                    node.id,
                )));
            }
            id_to_idx[node.id] = Some(idx);
            nodes_defined += 1;
        }

        if nodes_defined != num_nodes {
            return Err(serde_json::Error::custom(format!(
                "Node ID domain mismatch: expected {num_nodes} unique IDs, found {nodes_defined}",
            )));
        }

        for edge in &self.edges {
            if edge.source >= num_nodes {
                return Err(serde_json::Error::custom(format!(
                    "Edge source {} is out of bounds (0 to {})",
                    edge.source,
                    num_nodes - 1
                )));
            }
            if edge.target >= num_nodes {
                return Err(serde_json::Error::custom(format!(
                    "Edge target {} is out of bounds (0 to {})",
                    edge.target,
                    num_nodes - 1
                )));
            }

            for node in &self.nodes {
                if node.id == edge.source || node.id == edge.target {
                    continue;
                }

                // We expect the IDs to be valid at this point, since we have validated that all
                // nodes are uniquely defined and that there are `num_nodes` many nodes.
                let source_idx = id_to_idx[edge.source].expect("Invalid edge source id");
                let target_idx = id_to_idx[edge.target].expect("Invalid edge target id");

                let from = &self.nodes[source_idx];
                let to = &self.nodes[target_idx];

                if !is_between((from.x, from.y), (node.x, node.y), (to.x, to.y)) {
                    continue;
                }

                if is_collinear((from.x, from.y), (node.x, node.y), (to.x, to.y)) {
                    return Err(serde_json::Error::custom(format!(
                        "Node {} is collinear with edge from node {} ({},{}) to node {} ({},{}) at coordinates ({}, {})",
                        node.id,
                        edge.source,
                        from.x,
                        from.y,
                        edge.target,
                        to.x,
                        to.y,
                        node.x,
                        node.y
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn is_isomorphic(&self, graph: &Graph) -> bool {
        if self.nodes.len() != graph.nodes.len() {
            return false;
        }

        if self.edges.len() != graph.edges.len() {
            return false;
        }

        // Hmm, I bet there's a faster option
        let mut input_edges = self.edges.clone();
        let mut output_edges = graph.edges.clone();
        input_edges.sort();
        output_edges.sort();

        return input_edges == output_edges;
    }
}

fn is_between((a, b): (u32, u32), (n, m): (u32, u32), (x, y): (u32, u32)) -> bool {
    let [min_x, max_x] = minmax(a, x);
    let [min_y, max_y] = minmax(b, y);

    (min_x < n && n < max_x) && (min_y < m && m < max_y)
}

fn ccw((a, b): (u32, u32), (n, m): (u32, u32), (x, y): (u32, u32)) -> std::cmp::Ordering {
    let a = a as i64;
    let b = b as i64;
    let n = n as i64;
    let m = m as i64;
    let x = x as i64;
    let y = y as i64;
    return ((n - a) * (y - m)).cmp(&((m - b) * (x - n)));
}

fn is_collinear(p1: (u32, u32), q: (u32, u32), p2: (u32, u32)) -> bool {
    ccw(p1, q, p2) == Ordering::Equal
}

/// This assumes that no three points of p1,q1,p2,p2 are collinear
fn is_crossing(p1: (u32, u32), q1: (u32, u32), p2: (u32, u32), q2: (u32, u32)) -> bool {
    if p1 == p2 || p1 == q2 || q1 == p2 || q1 == q2 {
        return false;
    }

    let o1 = ccw(p1, q1, p2);
    let o2 = ccw(p1, q1, q2);
    let o3 = ccw(p2, q2, p1);
    let o4 = ccw(p2, q2, q1);

    o1 != o2 && o3 != o4
}

#[inline]
#[must_use]
pub fn minmax<T>(v1: T, v2: T) -> [T; 2]
where
    T: Ord,
{
    if v1 <= v2 { [v1, v2] } else { [v2, v1] }
}

#[cfg(test)]
mod test {
    use crate::graph::ccw;
    use std::cmp::Ordering;

    #[test]
    fn ccw_test() {
        let a = (0, 0);
        let b = (0, 1);
        let c = (1, 1);
        assert_eq!(ccw(a, b, c), Ordering::Less);
    }
}
