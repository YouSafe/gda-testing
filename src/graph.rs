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

#[derive(Serialize, Deserialize, Debug)]
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
        let mut max_crossings_per_edge = 0;

        for (i1, e1) in edges.iter().enumerate() {
            let mut crossings = 0;
            for e2 in edges.iter().skip(i1 + 1) {
                if is_crossing(e1.source, e1.target, e2.source, e2.target) {
                    crossings += 1;
                }
            }

            max_crossings_per_edge = max_crossings_per_edge.max(crossings);
            total_crossings += crossings;
        }

        CrossingCountingResult {
            total: total_crossings,
            max_per_edge: max_crossings_per_edge,
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

                let from = &self.nodes[edge.source];
                let to = &self.nodes[edge.target];

                if !is_between((from.x, from.y), (node.x, node.y), (to.x, to.y)) {
                    continue;
                }

                if is_collinear((from.x, from.y), (node.x, node.y), (to.x, to.y)) {
                    return Err(serde_json::Error::custom(format!(
                        "Node {} is collinear with edge from node {} to node {} at coordinates ({}, {})",
                        node.id, edge.source, edge.target, node.x, node.y
                    )));
                }
            }
        }

        Ok(())
    }
}

fn is_between((a, b): (u32, u32), (n, m): (u32, u32), (x, y): (u32, u32)) -> bool {
    let [min_x, max_x] = minmax(a, x);
    let [min_y, max_y] = minmax(b, y);

    (min_x < n && n < max_x) && (min_y < m && m < max_y)
}

fn ccw((a, b): (u32, u32), (n, m): (u32, u32), (x, y): (u32, u32)) -> std::cmp::Ordering {
    // FIXME: this easily overflows
    (y * n + b * x + m * a).cmp(&(m * x + y * a + b * n))
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
