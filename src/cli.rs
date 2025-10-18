// CLI module for Crusty-Crawler
// Provides command-line interface for headless server operation

use std::io::{self, Write};

pub fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Crusty-Crawler CLI Mode");
    println!("==========================\n");

    let server_state = Arc::new(Mutex::new(ServerState::default()));

    // Check if setup is needed
    let needs_setup = {
        let state = server_state.lock().unwrap();
        let auth_manager = state.auth_manager.lock().unwrap();
        !auth_manager.has_users()
    };

    if needs_setup {
        println!("👋 Welcome! First-time setup required.\n");
        setup_wizard(&server_state)?;
    } else {
        println!("✅ Configuration found.\n");
    }

    // Show main menu
    main_menu(server_state)?;

    Ok(())
}

fn setup_wizard(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Setup Wizard");
    println!("---------------\n");

    // Get username
    let username = loop {
        print!("Enter username (min 3 characters): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let username = input.trim().to_string();
        
        if username.len() >= 3 {
            break username;
        }
        println!("❌ Username must be at least 3 characters.\n");
    };

    // Get password
    let password = loop {
        let pass1 = rpassword::prompt_password("Enter password (min 8 characters): ")?;
        if pass1.len() < 8 {
            println!("❌ Password must be at least 8 characters.\n");
            continue;
        }
        
        let pass2 = rpassword::prompt_password("Confirm password: ")?;
        if pass1 != pass2 {
            println!("❌ Passwords do not match.\n");
            continue;
        }
        
        break pass1;
    };

    // Get email
    let email = loop {
        print!("Enter email address: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let email = input.trim().to_string();
        
        if email.contains('@') {
            break email;
        }
        println!("❌ Please enter a valid email address.\n");
    };

    // Generate or enter access token
    print!("\nGenerate random access token? (Y/n): ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    let access_token = if input.trim().eq_ignore_ascii_case("n") {
        loop {
            print!("Enter access token (min 8 characters): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let token = input.trim().to_string();
            
            if token.len() >= 8 {
                break token;
            }
            println!("❌ Access token must be at least 8 characters.\n");
        }
    } else {
        use crate::AuthManager;
        let token = AuthManager::generate_suggested_token();
        println!("Generated token: {}", token);
        token
    };

    // Register the user
    let state = server_state.lock().unwrap();
    let mut auth_manager = state.auth_manager.lock().unwrap();
    
    match auth_manager.register_user(&username, &password, &email, &access_token) {
        Ok(()) => {
            println!("\n✅ User registered successfully!");
            println!("📝 Your access token: {}\n", access_token);
            println!("⚠️  Save this token - you'll need it to access the web interface.\n");
        }
        Err(e) => {
            println!("\n❌ Registration failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn main_menu(server_state: Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        println!("\n📋 Main Menu");
        println!("-------------");
        println!("1. Start Server");
        println!("2. Stop Server");
        println!("3. Server Status");
        println!("4. Change Port");
        println!("5. Configure SMTP");
        println!("6. View Configuration");
        println!("7. Run as Service (daemon mode)");
        println!("8. Exit");
        print!("\nSelect option (1-8): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim() {
            "1" => start_server(&server_state)?,
            "2" => stop_server(&server_state)?,
            "3" => show_status(&server_state)?,
            "4" => change_port(&server_state)?,
            "5" => configure_smtp(&server_state)?,
            "6" => view_config(&server_state)?,
            "7" => run_daemon(&server_state)?,
            "8" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option. Please try again."),
        }
    }

    Ok(())
}

fn start_server(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    let is_running = {
        let state = server_state.lock().unwrap();
        state.is_running
    };

    if is_running {
        println!("⚠️  Server is already running!");
        return Ok(());
    }

    let port = {
        let state = server_state.lock().unwrap();
        state.port
    };

    println!("\n🚀 Starting server on port {}...", port);

    let rt = tokio::runtime::Runtime::new()?;
    let server_state_clone = server_state.clone();
    
    let (tx, rx) = tokio::sync::oneshot::channel();
    
    {
        let mut state = server_state.lock().unwrap();
        state.is_running = true;
        state.shutdown_sender = Some(tx);
    }
    
    std::thread::spawn(move || {
        rt.block_on(async {
            let app = create_app(server_state_clone.clone());
            let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));

            let listener = tokio::net::TcpListener::bind(addr).await;
            match listener {
                Ok(listener) => {
                    println!("✅ Server started successfully!");
                    println!("📍 Access at: http://localhost:{}", port);
                    println!("🌐 Network access: http://[YOUR-IP]:{}", port);

                    let server = axum::serve(listener, app);

                    tokio::select! {
                        _ = server => {
                            println!("Server stopped normally");
                        }
                        _ = rx => {
                            println!("Server received shutdown signal");
                        }
                    };
                }
                Err(e) => {
                    eprintln!("❌ Failed to bind to port {}: {}", port, e);
                    let mut state = server_state_clone.lock().unwrap();
                    state.is_running = false;
                }
            }

            let mut state = server_state_clone.lock().unwrap();
            state.is_running = false;
            state.shutdown_sender = None;
        });
    });

    Ok(())
}

fn stop_server(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    let (is_running, shutdown_sender) = {
        let mut state = server_state.lock().unwrap();
        let is_running = state.is_running;
        let shutdown_sender = state.shutdown_sender.take();
        state.is_running = false;
        (is_running, shutdown_sender)
    };

    if !is_running {
        println!("⚠️  Server is not running!");
        return Ok(());
    }

    println!("\n🛑 Stopping server...");

    if let Some(tx) = shutdown_sender {
        let _ = tx.send(());
    }

    println!("✅ Server stopped successfully!");

    Ok(())
}

fn show_status(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    let (is_running, port) = {
        let state = server_state.lock().unwrap();
        (state.is_running, state.port)
    };

    println!("\n📊 Server Status");
    println!("----------------");
    println!("Status: {}", if is_running { "🟢 Running" } else { "🔴 Stopped" });
    println!("Port: {}", port);
    
    if is_running {
        println!("Local URL: http://localhost:{}", port);
        println!("Network URL: http://[YOUR-IP]:{}", port);
    }

    Ok(())
}

fn change_port(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    let is_running = {
        let state = server_state.lock().unwrap();
        state.is_running
    };

    if is_running {
        println!("⚠️  Please stop the server before changing the port.");
        return Ok(());
    }

    print!("\nEnter new port number (1024-65535): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    match input.trim().parse::<u16>() {
        Ok(port) if port >= 1024 => {
            let mut state = server_state.lock().unwrap();
            state.port = port;
            println!("✅ Port changed to {}", port);
        }
        _ => {
            println!("❌ Invalid port number. Must be between 1024 and 65535.");
        }
    }

    Ok(())
}

fn configure_smtp(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📧 SMTP Configuration");
    println!("---------------------");

    print!("SMTP Server: ");
    io::stdout().flush()?;
    let mut server = String::new();
    io::stdin().read_line(&mut server)?;

    print!("Port (e.g., 587): ");
    io::stdout().flush()?;
    let mut port_str = String::new();
    io::stdin().read_line(&mut port_str)?;
    
    let port: u16 = match port_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            println!("❌ Invalid port number.");
            return Ok(());
        }
    };

    print!("Username: ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;

    let password = rpassword::prompt_password("Password: ")?;

    print!("Use TLS? (Y/n): ");
    io::stdout().flush()?;
    let mut tls_input = String::new();
    io::stdin().read_line(&mut tls_input)?;
    let use_tls = !tls_input.trim().eq_ignore_ascii_case("n");

    let smtp_config = crate::SmtpConfig {
        server: server.trim().to_string(),
        port,
        username: username.trim().to_string(),
        password: password.trim().to_string(),
        use_tls,
    };

    let state = server_state.lock().unwrap();
    let mut auth_manager = state.auth_manager.lock().unwrap();
    
    match auth_manager.configure_smtp(smtp_config) {
        Ok(()) => println!("✅ SMTP configuration saved!"),
        Err(e) => println!("❌ Failed to save configuration: {}", e),
    }

    Ok(())
}

fn view_config(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⚙️  Configuration");
    println!("----------------");

    let (port, user_count, has_smtp) = {
        let state = server_state.lock().unwrap();
        let auth_manager = state.auth_manager.lock().unwrap();
        let user_count = auth_manager.config.users.len();
        let has_smtp = auth_manager.config.smtp_config.is_some();
        (state.port, user_count, has_smtp)
    };

    println!("Port: {}", port);
    println!("Registered Users: {}", user_count);
    println!("SMTP Configured: {}", if has_smtp { "Yes" } else { "No" });

    Ok(())
}

fn run_daemon(server_state: &Arc<Mutex<ServerState>>) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Starting in daemon mode...");
    println!("Press Ctrl+C to stop the server.\n");

    start_server(server_state)?;

    // Keep the server running until Ctrl+C
    let running = Arc::new(Mutex::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        let mut running = r.lock().unwrap();
        *running = false;
    })?;

    println!("Server is running. Press Ctrl+C to stop.\n");

    // Wait for Ctrl+C
    while *running.lock().unwrap() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\n🛑 Shutting down...");
    stop_server(server_state)?;

    Ok(())
}
