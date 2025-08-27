#![allow(warnings)]

mod utils;
mod graph_utils;

//mod graph_utils;
mod test;
use chrono::{Datelike, Local, Timelike};

use common::file_conversion;
use common::network::{Network, Node};
use common::types::{ChatCommand, ChatEvent, Command, Event, MediaReference, Message, NodeCommand, NodeEvent, NodeType, TextFile, WebCommand, WebEvent};

use crossbeam::channel::{Receiver, Sender};
use crossbeam::select;

use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use regex::Regex;
use uuid::Uuid;

use network_initializer::network_initializer::{NetworkInitializer, Running, Uninitialized};
use wg_internal::controller::{DroneCommand, DroneEvent};
use wg_internal::network::NodeId;

use slint::{Color, ComponentHandle, Image, Model, ModelRc, PhysicalSize, SharedString, SharedVector, VecModel, Weak};
slint::include_modules!();


//? SIMULATION CONTROLLER DEFINITION

#[derive(Default)]
pub(crate) struct SimulationController {
    is_running: Arc<RwLock<bool>>,
    network_view: Network,
    clients: HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    servers: HashMap<NodeId, (NodeType, Sender<Box<dyn Command>>)>,
    drones: HashMap<NodeId, (f32, Sender<DroneCommand>)>,
    network_initializer: Option<NetworkInitializer<Running>>,
    listener: Option<std::thread::JoinHandle<()>>,
}

impl SimulationController {

    fn start_simulation(&mut self, path: &str, ui_handle: Weak<MainWindow>) {
        if *self.is_running.read().unwrap() {
            return;
        }
        self.is_running = Arc::new(RwLock::new(true));

        let initializer = NetworkInitializer::<Uninitialized>::new(path)
            .initialize()
            .start_simulation();

        self.clients = initializer.get_clients();
        self.servers = initializer.get_servers();
        self.drones = initializer.get_drones();
        self.network_view = initializer.get_network_view();

        let node_event_receiver = initializer.get_nodes_event_receiver();
        let drone_event_receiver = initializer.get_drones_event_receiver();
        let is_running = self.is_running.clone();

        self.network_initializer = Some(initializer);
        let nodes = self.get_nodes_with_type();
        self.listener = Some(std::thread::spawn(move || {
            Self::listen_to_events(
                node_event_receiver,
                drone_event_receiver,
                is_running,
                ui_handle,
                nodes
            )
        }));
    }

    fn stop_simulation(&mut self) {
        if !*self.is_running.read().unwrap() {
            return;
        }
        *self.is_running.write().unwrap() = false;

        if let Some(initializer) = &mut self.network_initializer {
            initializer.stop_simulation();
        }

        if let Some(handle) = self.listener.take() {
            handle.join().expect("Failed to join listener thread");
        }

        self.clients.clear();
        self.servers.clear();
        self.drones.clear();
        self.network_initializer = None;
    }

    fn listen_to_events(
        nodes_event_receiver: Receiver<Box<dyn Event>>,
        drone_event_receiver: Receiver<DroneEvent>,
        is_running: Arc<RwLock<bool>>,
        ui_handle: Weak<MainWindow>,
        nodes: (Vec<(NodeId, String)>, Vec<(NodeId, String)>),
    ) {
        while *is_running.read().unwrap() {
            select! {
                recv(nodes_event_receiver) -> msg => {
                    match msg {
                        Ok(event) => {
                            Self::handle_node_event(event, ui_handle.clone(), nodes.clone());
                        }
                        Err(e) => {
                            break;
                        }
                    }
                }
                recv(drone_event_receiver) -> msg => {
                    match msg {
                        Ok(event) => {
                            Self::handle_drone_event(event, ui_handle.clone());
                        }
                        Err(e) => {
                            eprintln!("Error receiving drone event: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
        std::process::exit(0);
    }

    fn handle_node_event(event: Box<dyn Event>, ui_handle: Weak<MainWindow>, nodes: (Vec<(NodeId, String)>, Vec<(NodeId, String)>)) {
        slint::invoke_from_event_loop(move || {
            if let Some(main_window) = ui_handle.upgrade() {
                let event = event.into_any();
                if let Some(event) = event.downcast_ref::<WebEvent>() {
                    match event {
                        WebEvent::CachedFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, CACHED FILES RECEIVED: {} files", files.len()));
                            file_conversion::save_files(notification_from, files);
                        },
                        WebEvent::File {
                            notification_from,
                            file,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE RECEIVED: {}", file.id.to_string()));
                            file_conversion::save_file(notification_from, file);
                        },
                        WebEvent::TextFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILES RECEIVED: {} files", files.len()));
                            file_conversion::save_text_files(notification_from, files);
                        },
                        WebEvent::TextFile {
                            notification_from,
                            file,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE RECEIVED: {}", file.id.to_string()));
                            file_conversion::save_text_file(notification_from, file);
                        },
                        WebEvent::MediaFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILES RECEIVED: {} files", files.len()));
                            file_conversion::save_media_files(notification_from, files);
                        },
                        WebEvent::MediaFile {
                            notification_from,
                            file,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE RECEIVED: {}", file.id.to_string()));
                            file_conversion::save_media_file(notification_from, file);
                        },
                        WebEvent::FilesListQueried {
                            notification_from,
                            from,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILES LIST QUERIED FROM: {from}"));
                        },
                        WebEvent::FileNotFound {
                            notification_from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE NOT FOUND: {uuid}"));
                        },
                        WebEvent::TextFileAdded {
                            notification_from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE ADDED: {uuid}"));
                        },
                        WebEvent::MediaFileAdded {
                            notification_from,
                            uuid,
                        } =>{
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE ADDED: {uuid}"));
                        },
                        
                        WebEvent::TextFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE REMOVED: {uuid}"));
                        },
                        WebEvent::TextFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE REMOVED: {uuid}"));
                        },
                        WebEvent::MediaFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE REMOVED: {uuid}"));
                        },
                        
                        WebEvent::FileOperationError {
                            notification_from,
                            msg,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE OPERATION ERROR: {msg}"));
                        },
                        WebEvent::FileRequested {
                            notification_from,
                            from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE REQUESTED FROM: {from}, UUID: {uuid}"));
                        },

                        WebEvent::BadUuid {
                            notification_from,
                            from,
                            uuid,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, BAD UUID FROM: {from}, UUID: {uuid}"));
                        },
                        WebEvent::FileServed {
                            notification_from,
                            file,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE SERVED: {file}"));
                        },
                        WebEvent::FilesLists { 
                            notification_from, 
                            files_map 
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, FILE LISTS"));
                        }
                    
                    }
                } else if let Some(event) = event.downcast_ref::<ChatEvent>() {
                    match event {
                        ChatEvent::ChatHistory {
                            notification_from,
                            history,
                        } => {
                            utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, CHAT HISTORY RECEIVED"));
                            utils::save_chat_history(notification_from, history);
                        },
                        ChatEvent::RegisteredClients {
                            notification_from,
                            list,
                        } => {
                            utils::handle_registered_clients(notification_from, list, ui_handle, nodes.clone());
                            // TODO to be tested
                        },
                        ChatEvent::MessageSent {
                            notification_from,
                            to,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MESSAGE SENT TO: {to}")),
                        ChatEvent::MessageReceived {
                            notification_from,
                            msg,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, RECEIVED MESSAGE {:?}", msg)),
                        ChatEvent::ClientRegistered {
                            client,
                            server
                        } => utils::log_default(&format!("NOTIFICATION FROM: {server}, REGISTERED CLIENT {client}")),
                        ChatEvent::ClientListQueried {
                            notification_from,
                            from,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, CLIENT LIST QUERIED BY {from}")),
                        ChatEvent::ClientNotInList {
                            notification_from,
                            id,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, CLIENT {id} NOT IN REGISTERED CLIENTS")),
                        ChatEvent::ErrorClientNotFound {
                            notification_from,
                            not_found,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, CLIENT {not_found} IS NOT REGISTERED IN SERVER")),
                        ChatEvent::RegistrationSucceeded {
                            notification_from,
                            to,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, SUCCESSFULLY REGISTERED TO SERVER {to}")),
                    }
                } else if let Some(event) = event.downcast_ref::<NodeEvent>() {
                    match event {
                        NodeEvent::PacketSent(packet) => {
                            utils::log(&format!("PACKET SENT: {}", packet), Color::from_rgb_u8( 123, 132, 150));
                        },
                        NodeEvent::FloodStarted(flood_counter,node_id) => {
                            utils::log_default(&format!("NOTIFICATION FROM: {}, FLOOD STARTED {} FLOOD", node_id, flood_counter));
                        },
                        NodeEvent::NodeRemoved(node_id) => {
                            utils::log_default(&format!("REMOVED {} NODE", node_id));
                        },
                        NodeEvent::MessageReceived {
                            notification_from,
                            from,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MESSAGE RECEIVED FROM: {from}")),
                        NodeEvent::MessageSent {
                            notification_from,
                            to,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, MESSAGE SENT TO: {to}")),
                        NodeEvent::ServerTypeQueried {
                            notification_from,
                            from,
                        } => utils::log_default(&format!("NOTIFICATION FROM: {notification_from}, SERVER TYPE QUERIED FROM: {from}")),
                    }
                }
            }
        });
    }

    fn handle_drone_event(even: DroneEvent, ui_handle: Weak<MainWindow>) {
        //unimplemented!();
    }

    fn get_drones_pdr(&self) -> Vec<(NodeId, f32)> {
        let mut unsorted = self.drones.iter().map(|(id, (pdr, _))| (*id, *pdr)).collect::<Vec<_>>();
        unsorted.sort_by(|(id_a, _), (id_b, _)| id_a.cmp(id_b));
        let sorted = unsorted.iter().map(|(id, pdr)| (*id, *pdr)).collect::<Vec<_>>();
        sorted
    }

    fn get_nodes_with_type(&self) -> (Vec<(NodeId, String)>, Vec<(NodeId, String)>){
        let mut unsorted = self.clients
            .iter()
            .map(|(id, (nt, _))| (*id, *nt))
            .collect::<Vec<_>>();

        unsorted.sort_by(|(id_a, _), (id_b, _)| id_a.cmp(id_b));

        let clients_sorted = unsorted
            .iter()
            .map(|(id, nt)| (*id, nt.to_string()))
            .collect::<Vec<_>>();

        let mut unsorted = self.servers
            .iter()
            .map(|(id, (nt, _))| (*id, *nt))
            .collect::<Vec<_>>();

        unsorted.sort_by(|(id_a, _), (id_b, _)| id_a.cmp(id_b));

        let servers_sorted = unsorted
            .iter()
            .map(|(id, nt)| (*id, nt.to_string()))
            .collect::<Vec<_>>();

        (clients_sorted, servers_sorted)
    }
}

//? END SIMULATION CONTROLLER DEFINITION

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let main_window = MainWindow::new()?;

    let mut simulation_controller = SimulationController::default();

    simulation_controller.start_simulation(
        "../network-initializer/config/butterfly.toml",
        main_window.as_weak(),
    );

    graph_utils::generate_graph(&main_window, &simulation_controller.network_view, &simulation_controller.clients, &simulation_controller.servers, &simulation_controller.drones);
    
    // Drones
    let mut drones = simulation_controller
        .get_drones_pdr();
    
    let drones = Rc::new(VecModel::from(drones.iter().map(|(id, pdr)| Drone { title: format!("Drone {id}").into(), id: id.to_string().into(), pdr: (pdr * 100.0).to_string().into() }).collect::<Vec<_>>()));

    main_window.set_drones(drones.clone().into());

    // Clients & Servers
    let (clients, servers) = simulation_controller.get_nodes_with_type();

    println!("{:?}", clients);
    println!("{:?}", servers);

    // Clients    
    let clients = Rc::new(VecModel::from(clients.iter().map(|(node_id, node_type)| Client { title: format!("Client {node_id}").into(), subtitle: node_type.into(), id: node_id.to_string().into(), kind: node_type.into() }).collect::<Vec<_>>()));
    main_window.set_clients(clients.clone().into());

    // Servers
    let servers = Rc::new(VecModel::from(servers.iter().map(|(node_id, node_type)| Server { title: format!("Server {node_id}").into(), subtitle: node_type.into(), id: node_id.to_string().into(), kind: node_type.into() }).collect::<Vec<_>>()));
    main_window.set_servers(servers.clone().into());

    // Log
    let logs_model: Rc<VecModel<LogMessage>> = Rc::new(VecModel::from(Vec::<LogMessage>::new()));
    main_window.set_logs(logs_model.clone().into());

    {
        let logs_model = logs_model.clone();
        main_window.on_add_log(move |entry: LogMessage| {
            logs_model.push(entry);
        });
    }

    {
        let mw_weak = main_window.as_weak();
        utils::set_logger(Box::new(move |entry: LogMessage| {
            let mw_weak = mw_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(mw) = mw_weak.upgrade() {
                    mw.invoke_add_log(entry);
                }
            });
        }));
    }

    {
        let logs_model = logs_model.clone();
        main_window.on_clear_logs(move || {
            logs_model.clear();
        });
    }

    let simulation_controller = Arc::new(simulation_controller);
    let sc_add_sender = Arc::clone(&simulation_controller);
    let sc_remove_sender = Arc::clone(&simulation_controller);
    let sc_shutdown = Arc::clone(&simulation_controller);
    let sc_crash = Arc::clone(&simulation_controller);
    let sc_set_packet_drop_rate = Arc::clone(&simulation_controller);
    let sc_get_chats_history = Arc::clone(&simulation_controller);
    let sc_get_registered_clients = Arc::clone(&simulation_controller);
    let sc_send_message = Arc::clone(&simulation_controller);
    let sc_register_to_server = Arc::clone(&simulation_controller);
    let sc_get_cached_files = Arc::clone(&simulation_controller);
    let sc_get_file = Arc::clone(&simulation_controller);
    let sc_get_text_files = Arc::clone(&simulation_controller);
    let sc_get_text_file = Arc::clone(&simulation_controller);
    let sc_get_media_files = Arc::clone(&simulation_controller);
    let sc_get_media_file = Arc::clone(&simulation_controller);
    let sc_add_text_file_from_path = Arc::clone(&simulation_controller);
    let sc_add_media_file_from_path = Arc::clone(&simulation_controller);
    let sc_remove_text_file = Arc::clone(&simulation_controller);
    let sc_remove_media_file = Arc::clone(&simulation_controller);
    let sc_query_text_files_list = Arc::clone(&simulation_controller);
    let sc_get_text_files_list = Arc::clone(&simulation_controller);
    let sc_add_text_file = Arc::clone(&simulation_controller);

    main_window.on_add_sender({
        move |node_command: SimulationControllerCommand, 
            node_type: SimulationControllerType, 
            node_id: SharedString,
            args: AddSender| {
                
            println!("add_sender {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            let args_node_id = args.node_id.parse::<NodeId>().unwrap();

            let sender2 = sc_add_sender
                .network_initializer
                .as_ref()
                .unwrap()
                .get_comms_channels()
                .get(&args_node_id)
                .unwrap()
                .get_sender()
                .clone();

            match node_type {
                SimulationControllerType::Drone => {
                    let sender1 = &sc_add_sender
                        .drones
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(DroneCommand::AddSender(args_node_id.clone(), sender2.clone()));
                }

                SimulationControllerType::ChatClient | SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_add_sender
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::AddSender(args_node_id.clone(), sender2.clone())));
                }

                SimulationControllerType::ChatServer | SimulationControllerType::WebServer => {
                    let sender1 = &sc_add_sender
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::AddSender(args_node_id.clone(), sender2.clone())));
                }
            }

            //TODO aggiungere collegamento grafo
        }
    });
    
    main_window.on_remove_sender({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveSender| {

            println!("remove_sender {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            let args_node_id = args.node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::Drone => {
                    let sender1 = &sc_remove_sender
                        .drones
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(DroneCommand::RemoveSender(args_node_id));
                }

                SimulationControllerType::ChatClient | SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_remove_sender
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::RemoveSender(args_node_id.clone())));
                }

                SimulationControllerType::ChatServer | SimulationControllerType::WebServer => {
                    let sender1 = &sc_remove_sender
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::RemoveSender(args_node_id.clone())));
                }
            }

            //TODO rimuovere collegamento grafo

            // for node in sc_remove_sender.network_view.nodes.iter() {
            //     if node_id == node.get_id() {
            //         // remove node
            //     }
            // }

            // for node in sc_remove_sender.network_initializer
        }
    });

    main_window.on_shutdown({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("shutdown {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();

            match node_type {

                SimulationControllerType::ChatClient | SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_shutdown
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::Shutdown));
                }

                SimulationControllerType::ChatServer | SimulationControllerType::WebServer => {
                    let sender1 = &sc_shutdown
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(NodeCommand::Shutdown));
                }

                _ => {}
            }

            //TODO rimuovere collegamento grafo
            //TODO rimuovere dalla lista
        }
    });

    main_window.on_crash({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("crash {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::Drone => {
                    let sender1 = &sc_crash
                        .drones
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(DroneCommand::Crash);
                }

                _ => {}
            }

            //TODO rimuovere collegamento grafo
            //TODO rimuovere dalla lista
        }
    });

    main_window.on_set_packet_drop_rate({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: SetPacketDropRate| {

            println!("set_packet_drop_rate {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            let args_pdr = args.pdr.parse::<f32>().unwrap() / 100.;

            match node_type {
                SimulationControllerType::Drone => {
                    let sender1 = &sc_set_packet_drop_rate
                        .drones
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(DroneCommand::SetPacketDropRate(args_pdr.clone()));
                }

                _ => {}
            }

            //TODO cambiare testo menu
        }
    }); 

    main_window.on_get_chats_history({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_chats_history {:?}", node_id);
            
            let node_id = node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::ChatClient => {
                    let sender1 = &sc_get_chats_history
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::GetChatsHistory));
                }

                SimulationControllerType::ChatServer => {
                    let sender1 = &sc_get_chats_history
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::GetChatsHistory));
                }

                _ => {}
            }

            //TODO creare file di testo
        }
    });

    main_window.on_get_registered_clients({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_registered_clients {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::ChatClient => {
                    let sender1 = &sc_get_registered_clients
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::GetRegisteredClients));
                }

                SimulationControllerType::ChatServer => {
                    let sender1 = &sc_get_registered_clients
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::GetRegisteredClients));
                }

                _ => {}
            }

            //TODO label grafo
        }
    });

    main_window.on_send_message({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: SendMessage| {

            println!("send_message {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let from = args.from.parse::<NodeId>().unwrap();
            let to = args.to.parse::<NodeId>().unwrap();
            let text = args.text.parse::<String>().unwrap();

            match node_type {
                SimulationControllerType::ChatClient => {
                    let sender1 = &sc_send_message
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::SendMessage(Message {from: from, to: to, text: text})));
                }

                SimulationControllerType::ChatServer => {
                    let sender1 = &sc_send_message
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::SendMessage(Message {from: from, to: to, text: text})));
                }

                _ => {}
            }
        }
    });

    main_window.on_register_to_server({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RegisterToServer| {

            println!("register_to_server {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let args_node_id = args.node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::ChatClient => {
                    let sender1 = &sc_register_to_server
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::RegisterToServer(args_node_id)));
                }

                SimulationControllerType::ChatServer => {
                    let sender1 = &sc_register_to_server
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(ChatCommand::RegisterToServer(args_node_id)));
                }

                _ => {}
            }

            //TODO label grafo

        }
    });

    main_window.on_get_cached_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_cached_files {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_cached_files
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetCachedFiles));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_cached_files
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetCachedFiles));
                }

                _ => {}
            }

            //TODO salvare cached files
        }
    });

    main_window.on_get_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: GetFile| {

            println!("get_file {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let uuid = args.uuid.parse::<Uuid>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetFile(uuid)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetFile(uuid)));
                }

                _ => {}
            }

            //TODO salvare file
        }
    });

    main_window.on_get_text_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_text_files {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_text_files
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFiles));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_text_files
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFiles));
                }

                _ => {}
            }

            //TODO salvare text files
        }
    });

    main_window.on_get_text_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: GetTextFile| {

            println!("get_text_file {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let uuid = args.uuid.parse::<Uuid>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_text_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFile(uuid)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_text_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFile(uuid)));
                }

                _ => {}
            }

            //TODO salvare text file
        }
    });

    main_window.on_get_media_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_media_files {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_media_files
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetMediaFiles));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_media_files
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetMediaFiles));
                }

                _ => {}
            }

            //TODO salvare media files
        }
    });

    main_window.on_get_media_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: GetMediaFile| {

            println!("get_media_file {:?}", node_id);
            
            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let media_id = args.media_id.parse::<Uuid>().unwrap();
            let location = args.location.parse::<NodeId>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_media_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetMediaFile { media_id: media_id, location: location }));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_media_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetMediaFile { media_id: media_id, location: location }));
                }

                _ => {}
            }

            //TODO salvare media file
        }
    });

    main_window.on_add_text_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: AddTextFile| {

            println!("add_text_file {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let re = Regex::new(r"\((\d+),([^)]+)\)").unwrap();
            
            let title = args.title.parse::<String>().unwrap();
            let content = args.content.parse::<String>().unwrap();
            let media_refs: Vec<MediaReference> = re
                .captures_iter(&args.media_refs.parse::<String>().unwrap())
                .map(|cap| {
                    let location: NodeId = cap[1].parse().unwrap();
                    let id = Uuid::parse_str(&cap[2]).unwrap();

                    MediaReference { location, id }
                })
                .collect();


            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_add_text_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddTextFile(TextFile::new(title, content, media_refs))));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_add_text_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddTextFile(TextFile::new(title, content, media_refs))));
                }

                _ => {}
            }
        }
    });

    main_window.on_add_text_file_from_path({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: AddTextFileFromPath| {

            println!("add_text_file_from_path {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let file_path = args.file_path.parse::<String>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_add_text_file_from_path
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddTextFileFromPath(file_path)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_add_text_file_from_path
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddTextFileFromPath(file_path)));
                }

                _ => {}
            }
        }
    });

    main_window.on_add_media_file_from_path({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: AddMediaFileFromPath| {

            println!("add_media_file_from_path {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let file_path = args.file_path.parse::<String>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_add_media_file_from_path
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddMediaFileFromPath(file_path)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_add_media_file_from_path
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::AddMediaFileFromPath(file_path)));
                }

                _ => {}
            }
        }
    });

    main_window.on_remove_text_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveTextFile| {

            println!("remove_text_file {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let uuid = args.uuid.parse::<Uuid>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_remove_text_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::RemoveTextFile(uuid)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_remove_text_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::RemoveTextFile(uuid)));
                }

                _ => {}
            }
        }
    });

    main_window.on_remove_media_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveMediaFile| {

            println!("remove_media_file {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            let uuid = args.uuid.parse::<Uuid>().unwrap();

            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_remove_media_file
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::RemoveMediaFile(uuid)));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_remove_media_file
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::RemoveMediaFile(uuid)));
                }

                _ => {}
            }
        }
    });

    main_window.on_query_text_files_list({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("query_text_files_list {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_query_text_files_list
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::QueryTextFilesList));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_query_text_files_list
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::QueryTextFilesList));
                }

                _ => {}
            }
        }
    });

    main_window.on_get_text_files_list({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {

            println!("get_text_files_list {:?}", node_id);

            let node_id = node_id.parse::<NodeId>().unwrap();
            
            match node_type {
                SimulationControllerType::WebBrowser => {
                    let sender1 = &sc_get_text_files_list
                        .clients
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFilesList));
                }

                SimulationControllerType::WebServer => {
                    let sender1 = &sc_get_text_files_list
                        .servers
                        .get(&node_id)
                        .unwrap()
                        .1;

                    sender1.send(Box::new(WebCommand::GetTextFilesList));
                }

                _ => {}
            }

            //TODO salvare file
        }
    });


    // Initial log
    utils::log("Simulation Controller started", Color::from_rgb_u8(123, 132, 150));

    main_window.run()?;
    
    //TODO
    // simulation_controller.stop_simulation();

    Ok(())

    /*
    let network: Network = simulation_controller.network_view;
    let (nodes, edges) = network.to_models();

    // Drones
    let drones_model: Rc<VecModel<Drone>> = Rc::new(VecModel::from(vec![
        Drone { title: "Drone 1".into(), id: "1".into() },
        Drone { title: "Drone 2".into(), id: "2".into() },
    ]));
    main_window.set_drones(ModelRc::from(drones_model.clone()));

    // Clients
    let clients_model: Rc<VecModel<Client>> = Rc::new(VecModel::from(vec![
        Client { title: "Client 1".into(), subtitle: "Web Client".into(), id: "3".into() },
        Client { title: "Client 2".into(), subtitle: "Chat Client".into(), id: "4".into() },
    ]));
    main_window.set_clients(ModelRc::from(clients_model.clone()));

    // Servers
    let servers_model: Rc<VecModel<Server>> = Rc::new(VecModel::from(vec![
        Server { title: "Server 1".into(), id: "5".into() },
        Server { title: "Server 2".into(), id: "6".into() },
    ]));
    main_window.set_servers(ModelRc::from(servers_model.clone()));

    // Current PDR
    main_window.set_initial_pdr(SharedString::from("0"));

    main_window.on_validate_pdr(|input: SharedString, current: SharedString| -> SharedString {
        utils::validate_pdr(&input, &current).into()
    });

    let logs_model: Rc<VecModel<SharedString>> =Rc::new(VecModel::from(Vec::<SharedString>::new()));
    main_window.set_logs(logs_model.clone().into());

    // The .slint callback: when UI calls add_log(...), just append the given line (no formatting here).
    {
        let logs_model = logs_model.clone();
        main_window.on_add_log(move |line: SharedString| {
            logs_model.push(line);
        });
    }

    // Register a global logger in utils that formats a timestamp and calls into the UI via invoke_add_log
    {
        let mw_weak = main_window.as_weak();
        utils::set_logger(Box::new(move |msg: SharedString| {
            let _ = slint::invoke_from_event_loop({
                let mw_weak = mw_weak.clone();
                move || {
                    if let Some(mw) = mw_weak.upgrade() {
                        let now = Local::now();
                        // Example format: [22/08/2025 14:35:12] message
                        let formatted = format!(
                            "[{}/{}/{} {:02}:{:02}:{:02}] {}",
                            now.day(),
                            now.month(),
                            now.year(),
                            now.hour(),
                            now.minute(),
                            now.second(),
                            msg
                        );
                        mw.invoke_add_log(formatted.into());
                    }
                }
            });
        }));
    }

    // Handlers that change the graph after removing/crashing nodes
    {
        let mw_weak = main_window.as_weak();
        let drones_model = drones_model.clone();
        let clients_model = clients_model.clone();
        let servers_model = servers_model.clone();

        main_window.on_remove_node(move |input: SharedString| {
            if let Some(pos) = (0..drones_model.row_count())
                .position(|i| drones_model
                    .row_data(i)
                    .map(|d| d.id == input)
                    .unwrap_or(false))
            {
                utils::remove_node(&input);
                drones_model.remove(pos);

                if let Some(mw) = mw_weak.upgrade() {
                    mw.set_graph_image(utils::update_graph(
                        drones_model.clone(),
                        clients_model.clone(),
                        servers_model.clone(),
                    ));
                }
            }
        });
    }

    {
        let mw_weak = main_window.as_weak();
        let drones_model = drones_model.clone();
        let clients_model = clients_model.clone();
        let servers_model = servers_model.clone();

        main_window.on_crash_node(move |input: SharedString| {
            if let Some(pos) = (0..drones_model.row_count())
                .position(|i| drones_model
                    .row_data(i)
                    .map(|d| d.id == input)
                    .unwrap_or(false))
            {
                utils::crash_node(&input);
                drones_model.remove(pos);

                if let Some(mw) = mw_weak.upgrade() {
                    mw.set_graph_image(utils::update_graph(
                        drones_model.clone(),
                        clients_model.clone(),
                        servers_model.clone(),
                    ));
                }
            }
        });
    }

    utils::log("Simulation Controller started");
    */

}
