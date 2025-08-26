


#[cfg(test)]
mod tests {
    use common::types::{WebEvent, MediaFile};
    use std::fs;
    use std::path::Path;
    use uuid::Uuid;
    use crate::{MainWindow, SimulationController};
    use std::sync::Arc;
    use slint::Weak;
    use common::file_conversion;

    #[test]
    fn test_simulation_controller_mediafile_save() {
        // Setup
        let mut simulation_controller = SimulationController::default();
    
         
        let file_id = Uuid::new_v4();
        let file_data = vec![10, 20, 30, 40, 50];
        let file_name = "test_mediafile.bin";

        // Create a dummy MediaFile (adjust fields as needed)
        let media_file = MediaFile {
            id: file_id,
            title: file_name.to_string(),
            content: vec![file_data.clone()],
        };
        let server_id = &1;
        // atomic testing save media files
        file_conversion::save_media_file(server_id,&media_file).expect("Failed to save media file");

        // Testable when UI active
        // Simulate receiving a WebEvent::MediaFile
        // let event = WebEvent::MediaFile {
        //     notification_from: 2,
        //     file: media_file.clone(),
        // };
        // Call the handler directly
        // SimulationController::handle_node_event(Box::new(event), Weak::default());

        // Check that the file was saved
        let fmt_path = format!("./cached_files_{}/{}_{}", server_id,media_file.id, media_file.title);
        println!("Checking saved media file at: {}", fmt_path);
        let path = Path::new(&fmt_path);
        println!("Reading saved media file at: {}", path.display());
        assert!(path.exists(), "Media file was not saved by SimulationController");
        let saved_data = fs::read(&path).expect("Failed to read saved media file");
        assert_eq!(saved_data, file_data, "Saved media file contents do not match");

        // Clean up
        let _ = fs::remove_file(&path);
    }

    
    }