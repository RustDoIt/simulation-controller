use std::rc::Rc;
use std::path::Path;

use slint::{Image, VecModel};

use crate::{Drone, Client, Server};

pub fn validate_node_id(input: &str) -> bool {

    // Try to parse as a number
    if let Ok(val) = input.parse::<isize>() {

        //TODO check if node exists

        //TODO add sender

        return true;
    }

    return false;
}

pub fn validate_pdr(input: &str, current: &str) -> String {

    // Try to parse as a number
    if let Ok(val) = input.parse::<f64>() {

        // Range check
        if val >= 0.0 && val <= 100.0 {

            // If different from current
            if input != current {

                //TODO set PDR

                return input.to_string();
            }

        }
    }

    // Fallback: keep current value
    current.to_string()
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