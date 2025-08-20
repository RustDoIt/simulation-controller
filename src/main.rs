mod utils;

slint::include_modules!();

use std::rc::Rc;
use slint::{SharedString, ModelRc, VecModel, Model};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = MainWindow::new()?;

    //TODO add existing drones
    let drones_model: Rc<VecModel<Drone>> = Rc::new(VecModel::from(vec![
        Drone { title: "Drone 1".into(), id: "1".into() },
        Drone { title: "Drone 2".into(), id: "2".into() },
    ]));
    main_window.set_drones(ModelRc::from(drones_model.clone()));
    
    //TODO add existing clients
    main_window.set_clients(ModelRc::new(VecModel::from(vec![
        Client { title: "Client 1".into(), subtitle: "Web Client".into(), id: "3".into() },
        Client { title: "Client 2".into(), subtitle: "Chat Client".into(), id: "4".into() },
    ])));
    
    //TODO add existing servers
    main_window.set_servers(ModelRc::new(VecModel::from(vec![
        Server { title: "Server 1".into(), id: "5".into() },
        Server { title: "Server 2".into(), id: "6".into() },
    ])));
    
    //TODO set current PDR
    main_window.set_initial_pdr(SharedString::from("0"));

    main_window.on_validate_pdr(|input: SharedString, current: SharedString| -> SharedString {
        utils::validate_pdr(&input, &current).into()
    });

    {
        let drones_model = drones_model.clone();
        main_window.on_validate_node_id(move |input: SharedString| {

            if(utils::validate_node_id(&input)) {

                //TODO update ui
                // drones_model.push(
                //     Drone {
                //         ...
                //     }
                // );

            }

        });
    }

    {
        let drones_model = drones_model.clone();
        main_window.on_remove_node(move |input: SharedString| {
            if let Some(pos) = (0..drones_model.row_count())
                .position(|i| drones_model
                    .row_data(i)
                    .map(|d| d.id == input)
                    .unwrap_or(false))
            {
                utils::remove_node(&input);

                drones_model.remove(pos);
            }
        });
    }

    {
        let drones_model = drones_model.clone();
        main_window.on_crash_node(move |input: SharedString| {
            if let Some(pos) = (0..drones_model.row_count())
                .position(|i| drones_model
                    .row_data(i)
                    .map(|d| d.id == input)
                    .unwrap_or(false))
            {
                utils::crash_node(&input);

                drones_model.remove(pos);
            }
        });
    }

    main_window.run()?;
    Ok(())
}
