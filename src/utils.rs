pub fn validate_node_id(input: &str) -> () {

    // Try to parse as a number
    if let Ok(val) = input.parse::<isize>() {

        //TODO check if node exists

        //TODO add sender
    }
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
