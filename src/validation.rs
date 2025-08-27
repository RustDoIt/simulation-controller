use std::collections::{HashMap, HashSet};

use common::network::Network;
use crossbeam::channel::Sender;
use common::types::{Command, NodeType, NodeCommand};
use wg_internal::{controller::DroneCommand, network::NodeId};
use wg_internal::packet::NodeType as WGNodeType;


pub fn can_remove_drone(network_graph: &HashMap<(NodeId, String), HashSet<NodeId>>, drone_id: NodeId, servers: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>) -> bool {


    // for each server, check if it has at least two drones after removing the drone id if present
    for (server_id, _) in servers {
        if let Some(adjacent) = network_graph.get(&(*server_id, "server".to_string())) {
            let drone_count = adjacent.iter().filter(|&&adj_id| {
                // the adjacent node should only be drones so i won't check adj_type == "drone"
                if let Some((_, adj_type)) = network_graph.keys().find(|(id, _)| id == &adj_id) {
                    adj_id != drone_id
                } else {
                false
            }
        }).count();
        if drone_count < 2 {
            return false; // This server would have less than two drones
        }}
    }
    true

}

pub fn can_remove_sender_drone(network_graph: &HashMap<(NodeId, String), HashSet<NodeId>>, drone_id: NodeId, sender_id: NodeId, servers: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>) -> bool {
    if let Some((_, sender_type)) = network_graph.keys().find(|(id, _)| id == &sender_id) {
        if sender_type != "server" {
            return true; // if the sender is not a server, we can remove it
        }
    } else {
        return true; // if the sender is not found, we can remove it
    }

    // check if it has at least two drones after removing the drone id if present
    if let Some(adjacent) = network_graph.get(&(sender_id, "server".to_string())) {
        let drone_count = adjacent.iter().filter(|&&adj_id| {
            // the adjacent node should only be drones so i won't check adj_type == "drone"
            if let Some((_, adj_type)) = network_graph.keys().find(|(id, _)| id == &adj_id) {
                adj_id != drone_id
            } else {
                false
            }
        }).count();
        drone_count >= 2
    } else {
        false
    }
}
