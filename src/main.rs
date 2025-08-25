#![allow(warnings)]

mod utils;

use chrono::{Datelike, Local, Timelike};

use common::network::Network;
use common::types::{ChatEvent, Command, Event, NodeEvent, WebEvent, NodeType};

use crossbeam::channel::{Receiver, Sender};
use crossbeam::select;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use network_initializer::network_initializer::{NetworkInitializer, Running, Uninitialized};
use wg_internal::controller::{DroneCommand, DroneEvent};
use wg_internal::network::NodeId;

use slint::{Model, ModelRc, SharedString, VecModel, Weak, SharedVector, ComponentHandle};
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
        self.listener = Some(std::thread::spawn(move || {
            
            Self::listen_to_events(
                node_event_receiver,
                drone_event_receiver,
                is_running,
                ui_handle,
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
    ) {
        while *is_running.read().unwrap() {
            select! {
                recv(nodes_event_receiver) -> msg => {
                    match msg {
                        Ok(event) => {
                            Self::handle_node_event(event, ui_handle.clone());
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

    fn handle_node_event(event: Box<dyn Event>, ui_handle: Weak<MainWindow>) {
        slint::invoke_from_event_loop(move || {
            if let Some(main_window) = ui_handle.upgrade() {
                let event = event.into_any();
                if let Some(event) = event.downcast_ref::<WebEvent>() {
                    match event {
                        WebEvent::CachedFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, CACHED FILES RECEIVED: {} files", files.len()));
                            todo!("Save files")
                        },
                        WebEvent::File {
                            notification_from,
                            file,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILE RECEIVED: {}", file.id.to_string()));
                            todo!("Save file")
                        },
                        WebEvent::TextFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILES RECEIVED: {} files", files.len()));
                            todo!("Save files")
                        },
                        WebEvent::TextFile {
                            notification_from,
                            file,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE RECEIVED: {}", file.id.to_string()));
                            todo!("Save file")
                        },
                        WebEvent::MediaFiles {
                            notification_from,
                            files,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILES RECEIVED: {} files", files.len()));
                            todo!("Save files")
                        },
                        WebEvent::MediaFile {
                            notification_from,
                            file,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE RECEIVED: {}", file.id.to_string()));
                            todo!("Save file")

                        },
                        WebEvent::FilesListQueried {
                            notification_from,
                            from,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILES LIST QUERIED FROM: {from}"));
                        },
                        WebEvent::FileNotFound {
                            notification_from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILE NOT FOUND: {uuid}"));
                        },
                        WebEvent::TextFileAdded {
                            notification_from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE ADDED: {uuid}"));
                        },
                        WebEvent::MediaFileAdded {
                            notification_from,
                            uuid,
                        } =>{
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE ADDED: {uuid}"));
                        },
                        
                        WebEvent::TextFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE REMOVED: {uuid}"));
                        },
                        WebEvent::TextFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, TEXT FILE REMOVED: {uuid}"));
                        },
                        WebEvent::MediaFileRemoved {
                            notification_from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, MEDIA FILE REMOVED: {uuid}"));
                        },
                        
                        WebEvent::FileOperationError {
                            notification_from,
                            msg,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILE OPERATION ERROR: {msg}"));
                        },
                        WebEvent::FileRequested {
                            notification_from,
                            from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILE REQUESTED FROM: {from}, UUID: {uuid}"));
                        },

                        WebEvent::BadUuid {
                            notification_from,
                            from,
                            uuid,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, BAD UUID FROM: {from}, UUID: {uuid}"));
                        },
                        WebEvent::FileServed {
                            notification_from,
                            file,
                        } => {
                            utils::log(&format!("NOTIFICATION FROM: {notification_from}, FILE SERVED: {file}"));
                        },
                    
                    }
                } else if let Some(event) = event.downcast_ref::<ChatEvent>() {
                    match event {
                        ChatEvent::ChatHistory {
                            notification_from,
                            history,
                        } => todo!(),
                        ChatEvent::RegisteredClients {
                            notification_from,
                            list,
                        } => todo!(),
                        ChatEvent::MessageSent {
                            notification_from,
                            to,
                        } => todo!(),
                        ChatEvent::MessageReceived {
                            notification_from,
                            msg,
                        } => todo!(),
                        ChatEvent::ClientRegistered { client, server } => todo!(),
                        ChatEvent::ClientListQueried {
                            notification_from,
                            from,
                        } => todo!(),
                        ChatEvent::ClientNotInList {
                            notification_from,
                            id,
                        } => todo!(),
                        ChatEvent::ErrorClientNotFound {
                            notification_from,
                            location,
                            not_found,
                        } => todo!(),
                        ChatEvent::RegistrationSucceeded {
                            notification_from,
                            to,
                        } => todo!(),
                    }
                } else if let Some(event) = event.downcast_ref::<NodeEvent>() {
                    match event {
                        NodeEvent::PacketSent(packet) => todo!(),
                        NodeEvent::FloodStarted(_, _) => todo!(),
                        NodeEvent::NodeRemoved(_) => todo!(),
                        NodeEvent::MessageReceived {
                            notification_from,
                            from,
                        } => todo!(),
                        NodeEvent::MessageSent {
                            notification_from,
                            to,
                        } => todo!(),
                        NodeEvent::ServerTypeQueried {
                            notification_from,
                            from,
                        } => todo!(),
                    }
                }
            }
        });
    }

    fn handle_drone_event(even: DroneEvent, ui_handle: Weak<MainWindow>) {
        unimplemented!();
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

    // Drones
    let mut drones = simulation_controller
        .get_drones_pdr();
    
    let drones = Rc::new(VecModel::from(drones.iter().map(|(id, pdr)| Drone { title: format!("Drone {id}").into(), id: id.to_string().into(), pdr: (pdr * 100.0).to_string().into() }).collect::<Vec<_>>()));

    main_window.set_drones(drones.clone().into());

    // Clients & Servers
    let (clients, servers) = simulation_controller.get_nodes_with_type();

    // Clients    
    let clients = Rc::new(VecModel::from(clients.iter().map(|(node_id, node_type)| Client { title: format!("Client {node_id}").into(), subtitle: node_type.into(), id: node_id.to_string().into() }).collect::<Vec<_>>()));
    main_window.set_clients(clients.clone().into());

    // Servers
    let servers = Rc::new(VecModel::from(servers.iter().map(|(node_id, _)| Server { title: format!("Server {node_id}").into(), subtitle: "".into(), id: node_id.to_string().into() }).collect::<Vec<_>>()));
    main_window.set_servers(servers.clone().into());

    // Log
    let logs_model: Rc<VecModel<SharedString>> =Rc::new(VecModel::from(Vec::<SharedString>::new()));
    main_window.set_logs(logs_model.clone().into());

    {
        let logs_model = logs_model.clone();
        main_window.on_add_log(move |line: SharedString| {
            logs_model.push(line);
        });
    }

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
    
    main_window.on_add_sender({
        move |node_command: SimulationControllerCommand, 
            node_type: SimulationControllerType, 
            node_id: SharedString,
            args: AddSender| {
            println!("add_sender {:?}", node_id);

            match node_type {
                SimulationControllerType::Drone => {

                    let (_, sender1) = simulation_controller.drones.get_mut(&node_id.parse::<NodeId>().unwrap()).unwrap();
                    let (_, sender2) = simulation_controller.drones.get_mut(&node_id.parse::<NodeId>().unwrap()).unwrap();
                    
                    sender1.send(DroneCommand::AddSender(args.node_id.clone(), sender2.clone()));
                    
                }
                SimulationControllerType::ChatClient => {}
                SimulationControllerType::WebBrowser => {}
                SimulationControllerType::ChatServer => {}
                SimulationControllerType::WebServer => {}
            }
        }
    });
    
    main_window.on_remove_sender({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveSender| {
            println!("remove_sender {:?}", node_id);
        }
    });

    main_window.on_shutdown({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("shutdown {:?}", node_id);
        }
    });

    main_window.on_get_chats_history({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("get_chats_history {:?}", node_id);
        }
    });

    main_window.on_get_registered_clients({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("get_registered_clients {:?}", node_id);
        }
    });

    main_window.on_send_message({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: SendMessage| {
            println!("send_message {:?}", node_id);
        }
    });

    main_window.on_get_cached_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("get_cached_files {:?}", node_id);
        }
    });

    main_window.on_get_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: GetFile| {
            println!("get_file {:?}", node_id);
        }
    });

    main_window.on_get_text_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("get_text_files {:?}", node_id);
        }
    });

    main_window.on_get_media_files({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString| {
            println!("get_media_files {:?}", node_id);
        }
    });

    main_window.on_get_media_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: GetMediaFile| {
            println!("get_media_file {:?}", node_id);
        }
    });

    main_window.on_add_text_file_from_path({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: AddTextFileFromPath| {
            println!("add_text_file_from_path {:?}", node_id);
        }
    });

    main_window.on_add_media_file_from_path({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: AddMediaFileFromPath| {
            println!("add_media_file_from_path {:?}", node_id);
        }
    });

    main_window.on_remove_text_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveTextFile| {
            println!("remove_text_file {:?}", node_id);
        }
    });

    main_window.on_remove_media_file({
        move |node_command: SimulationControllerCommand,
            node_type: SimulationControllerType,
            node_id: SharedString,
            args: RemoveMediaFile| {
            println!("remove_media_file {:?}", node_id);
        }
    });


    // Initial log
    utils::log("Simulation Controller started");

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
