#[cfg(any(debug_assertions, test))]
mod debug {
    use redb::{Database, ReadableTable, TableDefinition};
    use std::path::Path;

    // Define the table structure - assuming keys and values are strings
    // Adjust these types if your database uses different types
    const LISTS_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("lists");

    pub fn print_lists_data() -> Result<(), Box<dyn std::error::Error>> {
        // Path to the database
        let db_path = Path::new("/home/jyr/.local/share/events/events.db");

        // Check if database exists
        if !db_path.exists() {
            println!("Database file not found at: {:?}", db_path);
            return Ok(());
        }

        // Open the database in read-only mode
        println!("Opening database at: {:?}", db_path);
        let db = Database::open(db_path)?;

        // Start a read transaction
        let read_txn = db.begin_read()?;

        // Open the lists table
        match read_txn.open_table(LISTS_TABLE) {
            Ok(table) => {
                println!("Lists table contents:");
                println!("--------------------");

                // Iterate through all key-value pairs in the table
                let mut count = 0;
                for result in table.iter()? {
                    let (key, value) = result?;

                    // Try to convert keys and values to strings for display
                    let key_str = match std::str::from_utf8(key.value()) {
                        Ok(s) => s.to_string(),
                        Err(_) => format!("<binary data: {:?}>", key.value()),
                    };

                    // Try to parse value as JSON
                    let value_display = match std::str::from_utf8(value.value()) {
                        Ok(s) => match serde_json::from_str::<serde_json::Value>(s) {
                            Ok(json) => format!(
                                "{}",
                                serde_json::to_string_pretty(&json).unwrap_or(s.to_string())
                            ),
                            Err(_) => s.to_string(),
                        },
                        Err(_) => format!("<binary data: {:?}>", value.value()),
                    };

                    println!("Key: {}\nValue: {}\n", key_str, value_display);
                    count += 1;
                }

                if count == 0 {
                    println!("The lists table is empty.");
                } else {
                    println!("Total entries: {}", count);
                }
            }
            Err(e) => {
                println!("Failed to open lists table: {}", e);
            }
        }

        println!("--------------------");
        Ok(())
    }

    // For easy testing, include a main function that can be called
    #[allow(dead_code)]
    pub fn run_debug() {
        println!("Running database debug...");
        if let Err(e) = print_lists_data() {
            eprintln!("Error: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::debug::print_lists_data;

    #[test]
    fn test_print_lists_data() {
        // This test will run the print_lists_data function
        // and check if it runs without errors.
        assert!(print_lists_data().is_ok());
    }
}