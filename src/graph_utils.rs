use std::collections::{HashMap, HashSet};

use common::network::{Network, Node};
use common::types::{Command, NodeType};

use crossbeam::channel::{Receiver, Sender};

use wg_internal::network::NodeId;
use wg_internal::controller::DroneCommand;

use std::f64::consts::PI;
use slint::{ComponentHandle, ModelRc, VecModel, SharedString};

use wg_internal::packet::NodeType as WGNodeType;

use crate::{MainWindow, Position, Edge};

pub fn generate_graph(
    main_window: &MainWindow,
    network: &Network,
    clients: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    servers: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    drones: &HashMap<NodeId, (f32, Sender<DroneCommand>)>,
) {

    let mut graph: HashMap<(NodeId, String), HashSet<NodeId>> = HashMap::new();

    for node1 in &network.nodes {
        let node1_id = node1.get_id();
        let node1_type = match node1.get_node_type() {
            WGNodeType::Drone => "drone",
            WGNodeType::Client => "client",
            WGNodeType::Server => "server",
        }
        .to_string();

        let key1 = (node1_id, node1_type.clone());
        graph.entry(key1).or_insert_with(HashSet::new);

        for &node2_id in node1.get_adjacents() {
            graph.entry((node1_id, node1_type.clone()))
                .or_insert_with(HashSet::new)
                .insert(node2_id);

            let node2_type = if drones.contains_key(&node2_id) {
                "drone"
            } else if clients.contains_key(&node2_id) {
                "client"
            } else if servers.contains_key(&node2_id) {
                "server"
            } else {
                "unknown"
            };

            graph.entry((node2_id, node2_type.to_string()))
                .or_insert_with(HashSet::new)
                .insert(node1_id);
        }
    }

    let n = graph.len() as f64;
    if n == 0.0 {
        return;
    }

    let d = ((624.0 * PI) / (n + 2.0 * PI)) / 1.5;
    let layout_r = 324.0 - d;

    let mut nodes = Vec::with_capacity(graph.len());
    let mut positions: HashMap<NodeId, Position> = HashMap::new();

    for (i, (node_id, node_type)) in graph.keys().enumerate() {
        let theta = 2.0 * PI * (i as f64) / n;
        let x = 324.0 + layout_r * theta.cos();
        let y = 324.0 + layout_r * theta.sin();

        let position = Position {
            x: x as f32,
            y: y as f32,
            size: d as f32,
            kind: SharedString::from(node_type.as_str()),
            label: SharedString::from(node_id.to_string()),
        };

        positions.insert(*node_id, position.clone());
        nodes.push(position);
    }

    let mut edges = Vec::new();
    for ((node1_id, _), adjs) in &graph {
        if let Some(position1) = positions.get(node1_id) {
            for node2_id in adjs {
                if let Some(position2) = positions.get(node2_id) {
                    edges.push(Edge {
                        from_x: position1.x,
                        from_y: position1.y,
                        to_x: position2.x,
                        to_y: position2.y,
                        offset: 0.,
                    });
                }
            }
        }
    }

    main_window.set_edges(ModelRc::new(VecModel::from(edges)));
    main_window.set_nodes(ModelRc::new(VecModel::from(nodes)));
}
