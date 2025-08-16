slint::include_modules!();
use slint::{ModelRc, VecModel};

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
    
    main_window.run()?;
    Ok(())
}
