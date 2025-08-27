use std::rc::Rc;
use std::path::Path;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};
use once_cell::sync::OnceCell;
use chrono::{Datelike, Local, Timelike};
use slint::{Color, Image, SharedString, VecModel, Weak};
use wg_internal::network::NodeId;
use common::types::Message;
use crate::{Client, Drone, Server, SimulationController};
use crate::{ MainWindow, LogMessage };

use std::collections::{HashMap, HashSet};

use common::network::Network;
use crossbeam::channel::Sender;
use common::types::{Command, NodeType, NodeCommand};
use wg_internal::{controller::DroneCommand};
use wg_internal::packet::NodeType as WGNodeType;
static LOGGER: OnceCell<Box<dyn Fn(LogMessage) + Send + Sync + 'static>> = OnceCell::new();

pub fn set_logger(cb: Box<dyn Fn(LogMessage) + Send + Sync + 'static>) {
    let _ = LOGGER.set(cb);
}

pub fn log<S: Into<SharedString>>(msg: S, color: Color) {
    if let Some(cb) = LOGGER.get() {
        let now = chrono::Local::now();
        let formatted = format!(
            "[{}/{}/{} {:02}:{:02}:{:02}] {}",
            now.day(),
            now.month(),
            now.year(),
            now.hour(),
            now.minute(),
            now.second(),
            msg.into()
        );

        cb(LogMessage {
            message: formatted.into(),
            color,
        });
    }
}

pub fn log_default<S: Into<SharedString>>(msg: S) {
    log(msg, Color::from_rgb_u8(255, 255, 255));
}

/// Saves chat history into `chats_history_{notification_from}`.
///
/// For each pair of clients, creates a file `clients_{client1}_{client2}.txt`
/// containing all messages exchanged.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or any file cannot be created or written to.
pub fn save_chat_history(notification_from: &u8, history: &HashMap<NodeId, Vec<Message>>) -> std::io::Result<()> {
    let dir_name = format!("chats_history_{notification_from}");
    let dir_path = Path::new(&dir_name);
    fs::create_dir_all(dir_path)?;

    for (other_client, messages) in history {
        if other_client == notification_from {
            continue;
        }
        let file_name = format!("clients_{}_{}.txt", notification_from, other_client);
        let file_path = dir_path.join(file_name);
        let mut f = File::create(file_path)?;
        for message in messages {
            writeln!(f, "From {} to {}: {}", message.from, message.to, message.text)?;
        }
    }
    Ok(())
}

pub fn generate_generic_network_view(
    network: &Network,
    clients: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    servers: &HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    drones: &HashMap<NodeId, (f32, Sender<DroneCommand>)>,
) -> HashMap<(NodeId, String), HashSet<NodeId>> {

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

    graph
}


pub fn handle_registered_clients(notification_from: &u8, list: &Vec<u8>, main_window: Weak<MainWindow>, nodes: (Vec<(NodeId, String)>, Vec<(NodeId, String)>)) -> () {
    if let Some(mw) = main_window.upgrade() {

        let (clients_nodes, servers_nodes) = nodes;

        let mut servers = Vec::new();

        for (node_id, node_type) in servers_nodes.iter() {

            if node_id == notification_from {
                servers.push(Server {
                    id: node_id.to_string().into(),
                    title: format!("Server {}", node_id).into(),
                    subtitle: format!("{} | {:?}", node_type, list).into(),
                    kind: node_type.into()
                });
            } else {
                servers.push(Server {
                    id: node_id.to_string().into(),
                    title: format!("Server {}", node_id).into(),
                    subtitle: format!("{}", node_type).into(),
                    kind: node_type.into()
                });
            }
        }

        mw.set_servers(Rc::new(VecModel::from(servers)).into());

        let mut clients = Vec::new();

        for (node_id, node_type) in clients_nodes.iter() {
            clients.push(Client {
                id: node_id.to_string().into(),
                title: format!("Client {}", node_id).into(),
                subtitle: format!("{}", node_type).into(),
                kind: node_type.into()
            });
        }

        mw.set_clients(Rc::new(VecModel::from(clients)).into());
    }
}

pub fn remove_node(node_id: NodeId, sc: &mut SimulationController) {

    sc.network_view.nodes.retain(|n| n.get_id() != node_id);

    for n in sc.network_view.nodes.iter_mut() {
        n.remove_adjacent(node_id);
    }

    sc.clients.retain(|id, _| id != &node_id);
    sc.servers.retain(|id, _| id != &node_id);
    sc.drones.retain(|id, _| id != &node_id);
}

pub fn remove_edge(node_id: NodeId, args_node_id: NodeId, sc: &mut SimulationController) {

    for n in sc.network_view.nodes.iter_mut() {

        if n.get_id() == node_id {
            n.remove_adjacent(args_node_id);
        }
        else if n.get_id() == args_node_id {
            n.remove_adjacent(node_id);
        }
    }

}

pub fn add_edge(node_id: NodeId, args_node_id: NodeId, sc: &mut SimulationController) {

    for n in sc.network_view.nodes.iter_mut() {

        if n.get_id() == node_id {
            n.add_adjacent(args_node_id);
        } 
        else if n.get_id() == args_node_id {
            n.add_adjacent(node_id);
        }
    }
}

pub fn draw_menu(main_window: &MainWindow, sc: &SimulationController) {

    // Drones
    let mut drones = sc.get_drones_pdr();
    let drones = Rc::new(VecModel::from(drones.iter().map(|(id, pdr)| Drone { title: format!("Drone {id}").into(), id: id.to_string().into(), pdr: format!("{:.2}", pdr * 100.0).trim_end_matches('0').trim_end_matches('.').to_string().into() }).collect::<Vec<_>>()));

    main_window.set_drones(drones.clone().into());

    // Clients & Servers
    let (clients, servers) = sc.get_nodes_with_type();

    // Clients    
    let clients = Rc::new(VecModel::from(clients.iter().map(|(node_id, node_type)| Client { title: format!("Client {node_id}").into(), subtitle: node_type.into(), id: node_id.to_string().into(), kind: node_type.into() }).collect::<Vec<_>>()));
    main_window.set_clients(clients.clone().into());

    // Servers
    let servers = Rc::new(VecModel::from(servers.iter().map(|(node_id, node_type)| Server { title: format!("Server {node_id}").into(), subtitle: node_type.into(), id: node_id.to_string().into(), kind: node_type.into() }).collect::<Vec<_>>()));
    main_window.set_servers(servers.clone().into());
}