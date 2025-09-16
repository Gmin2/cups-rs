use cups_rs::{
    auth::{set_password_callback, get_password, do_authentication},
    get_destination, create_job, Result,
};
use std::io::{self, Write};

fn main() -> Result<()> {
    println!("CUPS Authentication Example");

    // Set up a password callback for GUI-style authentication
    println!("Setting up password callback...");
    set_password_callback(Some(Box::new(|prompt, _http, method, resource| {
        println!("Authentication required!");
        println!("Prompt: {}", prompt);
        println!("Method: {}", method);
        println!("Resource: {}", resource);
        
        print!("Enter password (or 'q' to quit): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        if input == "q" || input.is_empty() {
            println!("Authentication cancelled");
            None
        } else {
            Some(input.to_string())
        }
    })))?;

    // Test the password callback directly
    println!("\nTesting password callback directly...");
    match get_password("Test prompt:", None, "GET", "/test") {
        Some(password) => println!("Got password: {}", "*".repeat(password.len())),
        None => println!("No password provided"),
    }

    // Try to access a printer that might require authentication
    let printer_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "PDF".to_string());

    println!("\nTrying to access printer: {}", printer_name);
    
    match get_destination(&printer_name) {
        Ok(destination) => {
            println!("Successfully connected to: {}", destination.full_name());
            
            // Try to create a job (this might trigger authentication)
            println!("Attempting to create a job...");
            match create_job(&destination, "Authentication test job") {
                Ok(job) => {
                    println!("Job created successfully: ID {}", job.id);
                }
                Err(e) => {
                    println!("Job creation failed: {}", e);
                    
                    // Try manual authentication
                    println!("Attempting manual authentication...");
                    match do_authentication(None, "POST", "/") {
                        Ok(_) => println!("Manual authentication successful"),
                        Err(auth_err) => println!("Manual authentication failed: {}", auth_err),
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to access printer: {}", e);
        }
    }

    // Test removing the callback
    println!("\nRemoving password callback...");
    set_password_callback(None)?;
    
    match get_password("Test prompt after removal:", None, "GET", "/test") {
        Some(password) => println!("Unexpected password: {}", password),
        None => println!("Callback correctly removed - no password provided"),
    }

    println!("\nAuthentication example completed!");
    Ok(())
}