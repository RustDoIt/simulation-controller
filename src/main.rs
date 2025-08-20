mod utils;

slint::include_modules!();

use slint::{SharedString, ModelRc, VecModel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    main_window.set_drones(ModelRc::new(VecModel::from(vec![
        Drone { title: "Drone 1".into() },
        Drone { title: "Drone 2".into() },
    ])));

    main_window.set_clients(ModelRc::new(VecModel::from(vec![
        Client { title: "Client 1".into(), subtitle: "Web Client".into() },
        Client { title: "Client 2".into(), subtitle: "Chat Client".into() },
    ])));

    main_window.set_servers(ModelRc::new(VecModel::from(vec![
        Server { title: "Server 1".into() },
        Server { title: "Server 2".into() },
    ])));
    
    main_window.on_validate_pdr(|input: SharedString, current: SharedString| -> SharedString {
        utils::validate_pdr(&input, &current).into()
    });

    main_window.on_validate_node_id(|input: SharedString| -> () {
        utils::validate_node_id(&input).into()
    });

    main_window.run()?;
    Ok(())
}
