use std::rc::Rc;
use std::path::Path;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};
use once_cell::sync::OnceCell;
use slint::{Image, SharedString, VecModel, Weak};
use wg_internal::network::NodeId;
use common::types::Message;
use crate::{Client, Drone, Server, SimulationController};
use crate::{ MainWindow };

static LOGGER: OnceCell<Box<dyn Fn(SharedString) + Send + Sync + 'static>> = OnceCell::new();

// Register the logger callback (called from main.rs).
pub fn set_logger(cb: Box<dyn Fn(SharedString) + Send + Sync + 'static>) {
    let _ = LOGGER.set(cb);
}

//TODO log events
pub fn log<S: Into<SharedString>>(msg: S) {
    if let Some(cb) = LOGGER.get() {
        cb(msg.into());
    }
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

pub fn handle_registered_clients(notification_from: &u8, list: &Vec<u8>, main_window: Weak<MainWindow>, nodes: (Vec<(NodeId, String)>, Vec<(NodeId, String)>)) -> () {
    if let Some(mw) = main_window.upgrade() {
        let (clients_nodes, servers_nodes) = nodes;
        if clients_nodes.iter().any(|(node_id, _)| node_id == notification_from) {
            let clients = Rc::new(VecModel::from(clients_nodes.iter().map(|(node_id, node_type)| Client { title: format!("Client {}. Can reach:  {:?}", node_id, list).into(), subtitle: node_type.into(), id: node_id.to_string().into() }).collect::<Vec<_>>()));
            mw.set_clients(clients.into());
        }
        if servers_nodes.iter().any(|(node_id, _)| node_id == notification_from) {
            let servers = Rc::new(VecModel::from(servers_nodes.iter().map(|(node_id, node_type)| Server { title: format!("Server {}. Subscribed: {:?}", node_id, list).into(), subtitle: node_type.into(), id: node_id.to_string().into() }).collect::<Vec<_>>()));
            mw.set_servers(servers.into());
        }
    }
}

pub fn validate_node_id(input: &str) -> bool {

    // Try to parse as a number
    if let Ok(val) = input.parse::<isize>() {

        //TODO check if node exists

        //TODO add sender

        return true;
    }

    return false;
}

pub fn validate_pdr(input: &str, current: &str) {

    // Try to parse as a number
    if let Ok(val) = input.parse::<f64>() {

        // Range check
        if val >= 0.0 && val <= 100.0 {

            // If different from current
            if input != current {

                //TODO set PDR
            }

        }
    }
}

pub fn remove_node(input: &str) -> () {

    // Try to parse as a number
    if let Ok(val) = input.parse::<isize>() {

        //TODO check if node exists

        //TODO remove node
    }
} 

pub fn crash_node(input: &str) -> () {

    // Try to parse as a number
    if let Ok(val) = input.parse::<isize>() {

        //TODO check if node exists

        //TODO crash node
    }
} 

pub fn update_graph(
    drones: Rc<VecModel<Drone>>,
    clients: Rc<VecModel<Client>>,
    servers: Rc<VecModel<Server>>) -> Image {
    
    //TODO replace this with the actual graph
    Image::load_from_path(Path::new("assets/images/placeholder.png")).unwrap_or_default()
}
