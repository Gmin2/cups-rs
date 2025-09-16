use cups_rs::{
    config::{
        get_server, set_server, get_user, set_user, get_encryption, set_encryption,
        get_user_agent, set_user_agent, CupsConfig, EncryptionMode,
    },
    get_all_destinations, Result,
};

fn main() -> Result<()> {
    println!("CUPS Server Configuration Example");

    // Show current configuration
    println!("Server: {}", get_server());
    println!("User: {}", get_user());
    println!("Encryption: {:?}", get_encryption());
    println!("User-Agent: {}", get_user_agent());

    // Test individual configuration functions
    println!("\n--- Testing Individual Configuration ---");
    
    // Test server configuration
    println!("Setting server to 'print.example.com:8631'...");
    set_server(Some("print.example.com:8631"))?;
    println!("Server is now: {}", get_server());

    // Test user configuration
    println!("Setting user to 'testuser'...");
    set_user(Some("testuser"))?;
    println!("User is now: {}", get_user());

    // Test encryption configuration
    println!("Setting encryption to Required...");
    set_encryption(EncryptionMode::Required);
    println!("Encryption is now: {:?}", get_encryption());

    // Test user agent configuration
    println!("Setting User-Agent to 'MyPrintApp/2.0'...");
    set_user_agent(Some("MyPrintApp/2.0"))?;
    println!("User-Agent is now: {}", get_user_agent());

    // Restore defaults
    println!("\n--- Restoring Defaults ---");
    set_server(None)?;
    set_user(None)?;
    set_encryption(EncryptionMode::IfRequested);
    set_user_agent(None)?;
    
    println!("Server restored to: {}", get_server());
    println!("User restored to: {}", get_user());
    println!("Encryption restored to: {:?}", get_encryption());
    println!("User-Agent restored to: {}", get_user_agent());

    // Demonstrate scoped configuration with CupsConfig
    println!("\n--- Scoped Configuration Demo ---");
    println!("Current server before scoped config: {}", get_server());

    {
        let _config = CupsConfig::new()
            .with_server("scoped.example.com")?
            .with_user("scopeduser")?
            .with_encryption(EncryptionMode::Always)
            .with_user_agent("ScopedApp/1.0")?;

        println!("Inside scoped config:");
        let summary = _config.current_config();
        println!("  {}", summary);

        // Try to use CUPS with this configuration
        // Note: This will likely fail since scoped.example.com doesn't exist
        match get_all_destinations() {
            Ok(destinations) => {
                println!("  Successfully connected! Found {} printers", destinations.len());
            }
            Err(e) => {
                println!("  Expected connection failure: {}", e);
                println!("  (This is normal since 'scoped.example.com' doesn't exist)");
            }
        }
    }

    // Configuration should be restored after CupsConfig is dropped
    println!("After scoped config (should be restored):");
    println!("  Server: {}", get_server());
    println!("  User: {}", get_user());
    println!("  Encryption: {:?}", get_encryption());
    println!("  User-Agent: {}", get_user_agent());

    // Demonstrate configuration for different environments
    println!("\n--- Environment-Specific Configurations ---");

    // Development environment
    println!("Development environment config:");
    let _dev_config = CupsConfig::new()
        .with_server("localhost")?
        .with_user("developer")?
        .with_encryption(EncryptionMode::Never)
        .with_user_agent("DevApp/1.0-debug")?;
    
    let dev_summary = _dev_config.current_config();
    println!("  {}", dev_summary);

    // The configuration will be automatically restored when _dev_config is dropped
    println!("\nServer configuration demo completed!");
    println!("Note: All configurations are thread-local in CUPS");
    
    Ok(())
}