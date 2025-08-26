use std::{fs, path::Path};

use common::{file_conversion, types::{ChatEvent, WebEvent}};

use crate::{utils, SimulationController};




#[cfg(test)]
mod tests {
    use common::types::{WebEvent, MediaFile, ChatEvent};
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

    #[test]
    fn test_simulation_chat_history_saving() {
        // Setup
        let mut simulation_controller = SimulationController::default();

        let server_id = 1;
        let mut history = std::collections::HashMap::new();
        let messages = vec![
            common::types::Message::new(server_id, 3, "Hello".to_string()),
            common::types::Message::new(server_id, 3, "World".to_string()),
        ];
        let other_client = 2;
        history.insert(other_client, messages.clone());


        //IF USING UI
        // Simulate receiving a ChatEvent::ChatHistory
        // let event = ChatEvent::ChatHistory {
        //     notification_from: server_id,
        //     history: history.clone(),
        // };
        // Call the handler directly
        //SimulationController::handle_node_event(Box::new(event), Weak::default());


        //atomic test
        utils::save_chat_history(&server_id, &history).expect("Failed to save chat history");

        // Check that the chat history was saved in the correct file
        let chat_file_path = format!("./chats_history_{}/clients_{}_{}.txt",server_id, server_id, other_client);
        println!("Checking saved chat history at: {}", chat_file_path);
        let path = Path::new(&chat_file_path);
        println!("Reading saved chat history at: {}", path.display());
        assert!(path.exists(), "Chat history file was not saved by SimulationController");
        let saved_data = fs::read_to_string(&path).expect("Failed to read saved chat history file");
        let expected = messages.iter().map(|m| format!("From {} to {}: {}\n", m.from, m.to, m.text)).collect::<String>();
        assert_eq!(saved_data, expected, "Saved chat history contents do not match");

        // Clean up
        let _ = fs::remove_file(&path);
    }
