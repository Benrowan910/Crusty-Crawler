//Authored by Benjamin Rowan
//
// Created for Nagios Enterprises LLC During the 2025 Summer Nintern Program
// The goal is to understand and create from the ground up, a server side monitoring agent that posts information
// to a server that can than be used to determine the health of the system you want to monitor.
//
// Ultimately the goal is to hook this up with custom XI plugins, and to get it to work on multiple operating systems.
//
// I only plan on working on this until Blake returns from his vacation.

use eframe::egui;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use std::env;

// Axum Server Components
use axum::{Router, extract::Query, http::StatusCode, response::Html, routing::get};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;

// Includes
include!("network.rs");
include!("components.rs");
include!("disks.rs");
include!("hardware_statistics.rs");
include!("auth.rs");
include!("cli.rs");

// Web parameters query
#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

// Shared state between GUI and server
struct ServerState {
    is_running: bool,
    port: u16,
    shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,
    hardware_state: Arc<Mutex<HardwareMonitorState>>,
    auth_manager: Arc<Mutex<AuthManager>>,
}

impl Default for ServerState {
    fn default() -> Self {
        let auth_manager = AuthManager::new("crusty_auth.json")
            .unwrap_or_else(|_| AuthManager::new("crust_auth.json").unwrap());

        Self {
            is_running: false,
            port: 3000,
            shutdown_sender: None,
            hardware_state: Arc::new(Mutex::new(HardwareMonitorState::default())),
            auth_manager: Arc::new(Mutex::new(auth_manager)),
        }
    }
}

enum AppState {
    Setup(SetupState),
    Login(LoginState),
    Main(MainState),
    Recovery(RecoveryState),
    SmtpConfig(SmtpConfigState),
}

struct SetupState {
    username: String,
    password: String,
    confirm_password: String,
    email: String,
    access_token: String,
    error_message: String,
    show_token_suggestion: bool,
}

struct LoginState {
    username: String,
    password: String,
    email: String,
    error_message: String,
    show_recovery: bool,
}

struct RecoveryState {
    email: String,
    message: String,
    is_success: bool,
}

struct SmtpConfigState {
    server: String,
    port: String,
    username: String,
    password: String,
    use_tls: bool,
    message: String,
}

struct MainState {
    port_input: String,
    server_state: Arc<Mutex<ServerState>>,
    status_message: String,
    current_user: String,
}

impl MainState {
    fn start_server(&mut self) {
        let port = match self.port_input.parse::<u16>() {
            Ok(p) => p,
            Err(_) => {
                self.status_message = format!("Invalid port number: {}", self.port_input);
                return;
            }
        };

        let server_state = self.server_state.clone();

        {
            let state = server_state.lock().unwrap();
            if state.is_running {
                self.status_message = "Server is already running!".to_string();
                return;
            }
        }

        // Creates a new runtime for the server
        let rt = Runtime::new().unwrap();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        {
            let mut state = server_state.lock().unwrap();
            state.is_running = true;
            state.port = port;
            state.shutdown_sender = Some(shutdown_tx);
        }

        let server_state_clone = server_state.clone();

        // Spawn the server in a separate thread
        std::thread::spawn(move || {
            rt.block_on(async {
                let app = create_app(server_state_clone.clone());
                let addr = SocketAddr::from(([0, 0, 0, 0], port));

                println!("🚀 Server starting on port {}", port);

                let listener = tokio::net::TcpListener::bind(addr).await;
                match listener {
                    Ok(listener) => {
                        println!("✅ Server running at http://0.0.0.0:{}", port);
                        println!("   Accessible from any device on your network!");

                        let server = axum::serve(listener, app);

                        tokio::select! {
                            _ = server => {
                                println!("Server stopped normally");
                            }
                            _ = shutdown_rx => {
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
        self.status_message = format!(
            "✅ Server hosted on port {} (accessible from any device)",
            port
        );
    }

    fn stop_server(&mut self) {
        let shutdown_sender = {
            let mut state = self.server_state.lock().unwrap();
            state.shutdown_sender.take()
        };

        if let Some(sender) = shutdown_sender {
            // Send shutdown signal - ignore error if receiver is dropped
            let _ = sender.send(());
            self.status_message = "🛑 Server shutdown initiated...".to_string();
        } else {
            self.status_message = "❌ Server is not running".to_string();
        }

        // Immediately mark as not running for UI responsiveness
        {
            let mut state = self.server_state.lock().unwrap();
            state.is_running = false;
        }
    }
}

struct MyApp {
    app_state: AppState,
    server_state: Arc<Mutex<ServerState>>,
    // Remove these duplicate fields since they're in MainState:
    // port_input: String,
    // status_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let auth_manager = AuthManager::new("crusty_auth.json")
            .unwrap_or_else(|_| AuthManager::new("crusty_auth.json").unwrap());

        let has_users = auth_manager.has_users();

        let initial_state = if !has_users {
            AppState::Setup(SetupState {
                username: String::new(),
                password: String::new(),
                confirm_password: String::new(),
                email: String::new(),
                access_token: String::new(),
                error_message: String::new(),
                show_token_suggestion: true,
            })
        } else {
            AppState::Login(LoginState {
                username: String::new(),
                password: String::new(),
                email: String::new(),
                error_message: String::new(),
                show_recovery: false,
            })
        };

        Self {
            app_state: initial_state,
            server_state: Arc::new(Mutex::new(ServerState::default())),
            // Remove these:
            // status_message: String::new(),
            // port_input: String::new(),
        }
    }
}

// Axum apllication and routing of information
fn create_app(server_state: Arc<Mutex<ServerState>>) -> Router {
    let server_state_clone = server_state.clone();

    Router::new()
        .route(
            "/api/status",
            get(move |query: Query<TokenQuery>| status_handler(server_state, query)),
        )
        .route(
            "/",
            get(move |query: Query<TokenQuery>| index_handler(server_state_clone, query)),
        )
        .fallback_service(ServeDir::new("public"))
}

// Endpoint handlers with token validation
async fn status_handler(
    server_state: Arc<Mutex<ServerState>>,
    query: Query<TokenQuery>,
) -> Result<Html<String>, StatusCode> {
    // Extract token validation into a separate scope to release the lock
    let is_valid = {
        let state = server_state.lock().unwrap();
        let auth_manager = state.auth_manager.lock().unwrap();

        if let Some(token) = &query.token {
            auth_manager.validate_token(token).is_ok()
        } else {
            false
        }
    };

    if is_valid {
        Ok(Html(status(server_state).await))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn index_handler(
    server_state: Arc<Mutex<ServerState>>,
    query: Query<TokenQuery>,
) -> Result<Html<String>, StatusCode> {
    let state = server_state.lock().unwrap();
    let auth_manager = state.auth_manager.lock().unwrap();

    if let Some(token) = &query.token {
        if auth_manager.validate_token(token).is_ok() {
            let html_content = include_str!("../public/index.html")
                .replace("{{TOKEN}}", token)
                .replace("{{PORT}}", &state.port.to_string());
            Ok(Html(html_content))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        // Return a login page for token entry
        let login_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Crusty Server - Login</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; }
                .container { max-width: 400px; margin: 0 auto; }
                input { width: 100%; padding: 10px; margin: 10px 0; }
                button { width: 100%; padding: 10px; background: #007bff; color: white; border: none; }
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Crusty Server</h1>
                <p>Enter your access token:</p>
                <input type="password" id="token" placeholder="Access Token">
                <button onclick="login()">Access System</button>
            </div>
            <script>
                function login() {
                    const token = document.getElementById('token').value;
                    if (token) {
                        window.location.href = '/?token=' + encodeURIComponent(token);
                    }
                }
            </script>
        </body>
        </html>
        "#;
        Ok(Html(login_html.to_string()))
    }
}

// Display the system statistics collected
async fn status(server_state: Arc<Mutex<ServerState>>) -> String {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let token = {
        let state = server_state.lock().unwrap();
        state
            .auth_manager
            .lock()
            .unwrap()
            .config
            .users
            .values()
            .next()
            .map(|u| u.access_token.clone())
            .unwrap_or_default()
    };
    let mut out = String::new();
    out.push_str(&format!(
        "System name: {:?}\n",
        sysinfo::System::name().unwrap_or_default()
    ));
    out.push_str(&format!(
        "Memory in Use: {} MB\n",
        sys.used_memory() / 1024 / 1024
    ));
    out.push_str(&format!("CPU usage: {:.1}%\n", sys.global_cpu_usage()));

    out.push_str(&get_hardware_status(&server_state));

    // Fetch network info
    match network_info().await {
        Ok(networks) => {
            out.push_str("\nNetwork Statistics (Total):\n");
            for net in networks {
                out.push_str(&format!("  {}\n", net));
            }
        }
        Err(e) => {
            out.push_str(&format!("\nError getting network stats: {}\n", e));
        }
    }

    // Get current network traffic
    match network_traffic().await {
        Ok(traffic) => {
            out.push_str("\nCurrent Network Traffic:\n");
            for net in traffic {
                out.push_str(&format!("  {}\n", net));
            }
        }
        Err(e) => {
            out.push_str(&format!("\nError getting network traffic: {}\n", e));
        }
    }

    match check_components().await {
        Ok(components) => {
            out.push_str("\nComponents:\n");
            if components.is_empty() {
                out.push_str("No Components Found\n");
            }
            for component in components {
                out.push_str(&format!("  {}\n", component));
            }
        }
        Err(e) => {
            out.push_str(&format!("\nError checking components: {}\n", e));
        }
    }

    match check_disks().await {
        Ok(disks) => {
            out.push_str("\nDisks:\n");
            if disks.is_empty() {
                out.push_str("No Disks Found\n");
            }
            for disk in disks {
                out.push_str(&format!("  {}\n", disk));
            }
        }
        Err(e) => {
            out.push_str(&format!("\nError checking disks: {}\n", e));
        }
    }
    out.push_str(&format!(
        "\nAccess URL: http://localhost:3000/?token={}",
        token
    ));
    out
}

enum AppAction {
    None,
    SwitchToLogin(LoginState),
    SwitchToMain(MainState),
    SwitchToRecovery,
    SwitchToSmtpConfig(String), // pass current user for return
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut action = AppAction::None;
        match &mut self.app_state {
            AppState::Setup(setup_state) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("🦀 Crusty Server - First Time Setup");
                    ui.separator();

                    ui.label("Create your administrator account:");

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut setup_state.username);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut setup_state.password).password(true),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Confirm Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut setup_state.confirm_password)
                                .password(true),
                        );
                    });

                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.text_edit_singleline(&mut setup_state.email);
                    });

                    ui.separator();
                    ui.heading("Access Token Configuration");
                    ui.label("This token will be used to access the web interface.");

                    ui.horizontal(|ui| {
                        ui.label("Access Token:");
                        ui.text_edit_singleline(&mut setup_state.access_token);

                        if ui.button("🎲 Suggest Token").clicked() {
                            setup_state.access_token = AuthManager::generate_suggested_token();
                        }
                    });

                    if setup_state.show_token_suggestion && setup_state.access_token.is_empty() {
                        ui.colored_label(
                            egui::Color32::YELLOW,
                            "💡 Click 'Suggest Token' to generate a secure token",
                        );
                    }

                    if !setup_state.error_message.is_empty() {
                        ui.colored_label(egui::Color32::RED, &setup_state.error_message);
                    }

                    ui.separator();

                    if ui.button("✅ Complete Setup").clicked() {
                        // Validate inputs
                        if setup_state.username.len() < 3 {
                            setup_state.error_message =
                                "Username must be at least 3 characters".to_string();
                        } else if setup_state.password.len() < 8 {
                            setup_state.error_message =
                                "Password must be at least 8 characters".to_string();
                        } else if setup_state.password != setup_state.confirm_password {
                            setup_state.error_message = "Passwords do not match".to_string();
                        } else if setup_state.access_token.len() < 8 {
                            setup_state.error_message =
                                "Access token must be at least 8 characters".to_string();
                        } else if !setup_state.email.contains('@') {
                            setup_state.error_message =
                                "Please enter a valid email address".to_string();
                        } else {
                            // Try to register the user
                            let server_state = self.server_state.lock().unwrap();
                            let mut auth_manager = server_state.auth_manager.lock().unwrap();
                            match auth_manager.register_user(
                                &setup_state.username,
                                &setup_state.password,
                                &setup_state.email,
                                &setup_state.access_token,
                            ) {
                                Ok(()) => {
                                    action = AppAction::SwitchToLogin(LoginState {
                                        username: setup_state.username.clone(),
                                        password: String::new(),
                                        email: String::new(),
                                        error_message: String::new(),
                                        show_recovery: false,
                                    });
                                }
                                Err(e) => {
                                    setup_state.error_message = e;
                                }
                            }
                        }
                    }
                });
            }

            AppState::Login(login_state) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("🦀 Crusty Server - Login");
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut login_state.username);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(
                            egui::TextEdit::singleline(&mut login_state.password).password(true),
                        );
                    });

                    if !login_state.error_message.is_empty() {
                        ui.colored_label(egui::Color32::RED, &login_state.error_message);
                    }

                    ui.separator();

                    if ui.button("🔑 Login").clicked() {
                        let server_state = self.server_state.lock().unwrap();
                        let auth_manager = server_state.auth_manager.lock().unwrap();
                        match auth_manager
                            .authenticate(&login_state.username, &login_state.password)
                        {
                            Ok(_token) => {
                                action = AppAction::SwitchToMain(MainState {
                                    port_input: "3000".to_string(),
                                    server_state: self.server_state.clone(),
                                    status_message: String::new(),
                                    current_user: login_state.username.clone(),
                                });
                            }
                            Err(e) => {
                                login_state.error_message = e;
                            }
                        }
                    }

                    if ui.button("🔓 Forgot Credentials?").clicked() {
                        login_state.show_recovery = true;
                    }

                    if login_state.show_recovery {
                        ui.separator();
                        ui.heading("Recover Credentials");
                        ui.label("Enter your email address to receive your credentials:");

                        ui.horizontal(|ui| {
                            ui.label("Email:");
                            ui.text_edit_singleline(&mut login_state.email);
                        });

                        if ui.button("📧 Send Recovery Email").clicked() {
                            let server_state = self.server_state.lock().unwrap();
                            let auth_manager = server_state.auth_manager.lock().unwrap();
                            match auth_manager.recover_credentials(&login_state.email) {
                                Ok(()) => {
                                    login_state.error_message =
                                        "Recovery email sent! Check your inbox.".to_string();
                                    login_state.show_recovery = false;
                                }
                                Err(e) => {
                                    login_state.error_message = e;
                                }
                            }
                        }

                        if ui.button("❌ Cancel").clicked() {
                            login_state.show_recovery = false;
                        }
                    }
                });
            }

            AppState::Main(main_state) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Header section with icon and title
                    ui.horizontal(|ui| {
                        ui.heading("🦀 Crusty Server");
                        ui.label("v1.0.0");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("Logged in as: {}", main_state.current_user));
                            if ui.button("🚪 Logout").clicked() {
                                action = AppAction::SwitchToLogin(LoginState {
                                    username: String::new(),
                                    password: String::new(),
                                    email: String::new(),
                                    error_message: String::new(),
                                    show_recovery: false,
                                });
                            }
                        });
                    });
                    ui.separator();

                    // Server configuration section
                    ui.vertical(|ui| {
                        ui.heading("Server Configuration");

                        egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Port:")
                                        .on_hover_text("Port number for the web server");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut main_state.port_input)
                                            .desired_width(80.0),
                                    );

                                    // Visual port validation
                                    if main_state.port_input.parse::<u16>().is_err() {
                                        ui.colored_label(egui::Color32::RED, "❌ Invalid port");
                                    } else {
                                        ui.colored_label(egui::Color32::GREEN, "✅ Valid");
                                    }
                                });
                            });
                    });
                    ui.separator();

                    // Server control section
                    ui.vertical(|ui| {
                        ui.heading("Server Control");

                        let (is_running, current_port) = {
                            let state = main_state.server_state.lock().unwrap();
                            (state.is_running, state.port)
                        };

                        ui.horizontal(|ui| {
                            if !is_running {
                                if ui
                                    .add(
                                        egui::Button::new("🚀 Start Server")
                                            .fill(egui::Color32::from_rgb(46, 125, 50)),
                                    )
                                    .clicked()
                                {
                                    main_state.start_server();
                                }
                            } else {
                                if ui
                                    .add(
                                        egui::Button::new("🛑 Stop Server")
                                            .fill(egui::Color32::from_rgb(211, 47, 47)),
                                    )
                                    .clicked()
                                {
                                    main_state.stop_server();
                                }
                            }

                            // Status indicator
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if is_running {
                                        ui.colored_label(
                                            egui::Color32::GREEN,
                                            format!("● Running on port {}", current_port),
                                        );
                                    } else {
                                        ui.colored_label(egui::Color32::GRAY, "● Stopped");
                                    }
                                },
                            );
                        });

                        // Status message with better styling
                        if !main_state.status_message.is_empty() {
                            ui.separator();
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(30, 30, 30, 100))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("📢");
                                        ui.label(&main_state.status_message);
                                    });
                                });
                        }
                    });

                    // Server information section (only when running)
                    let (is_running, current_port, last_update) = {
                        let state = main_state.server_state.lock().unwrap();
                        let hardware_state = state.hardware_state.lock().unwrap();
                        let last_update = hardware_state.last_update.elapsed().as_secs();
                        (state.is_running, state.port, last_update)
                    };

                    if is_running {
                        ui.separator();
                        ui.vertical(|ui| {
                            ui.heading("📊 Server Information");

                            egui::Frame::group(ui.style())
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    ui.label("📍 Access URLs:");
                                    ui.indent("urls", |ui| {
                                        ui.monospace(format!(
                                            "Local:    http://localhost:{}",
                                            current_port
                                        ));
                                        ui.monospace(format!(
                                            "Network:  http://[YOUR-IP]:{}",
                                            current_port
                                        ));
                                    });

                                    ui.add_space(5.0);
                                    ui.label(
                                        "💡 Replace [YOUR-IP] with your computer's IP address",
                                    );
                                    ui.colored_label(
                                        egui::Color32::LIGHT_BLUE,
                                        "🌐 Accessible from any device on your network!",
                                    );
                                });

                            ui.add_space(10.0);

                            // Hardware monitoring status
                            ui.heading("🔧 Hardware Monitoring");
                            egui::Frame::group(ui.style())
                                .inner_margin(egui::Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Last updated:");
                                        if last_update < 60 {
                                            ui.colored_label(
                                                egui::Color32::GREEN,
                                                format!("{} seconds ago", last_update),
                                            );
                                        } else {
                                            ui.colored_label(
                                                egui::Color32::YELLOW,
                                                format!("{} seconds ago", last_update),
                                            );
                                        }
                                    });
                                    ui.label("⏱️ Power and thermal data refreshes every 60s");
                                });
                        });
                    }

                    // Instructions section
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.heading("💡 Instructions");

                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(25, 25, 35, 100))
                            .inner_margin(egui::Margin::same(10))
                            .show(ui, |ui| {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("1.");
                                        ui.label("Enter a port number (default: 3000)");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("2.");
                                        ui.label("Click 'Start Server' to begin hosting");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("3.");
                                        ui.label("Access the status page from any browser");
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("4.");
                                        ui.label("Use 'Stop Server' to shut down");
                                    });
                                });
                            });
                    });

                    // Footer
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.small(
                            "Created for Nagios Enterprises LLC • 2025 Summer Nintern Program",
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.small("🦀 Powered by Rust");
                        });
                    });
                });
            }

            AppState::Recovery(recovery_state) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("🔓 Recover Credentials");
                    ui.separator();

                    ui.label("Enter your email address to receive your login credentials:");

                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.text_edit_singleline(&mut recovery_state.email);
                    });

                    if !recovery_state.message.is_empty() {
                        let color = if recovery_state.is_success {
                            egui::Color32::GREEN
                        } else {
                            egui::Color32::RED
                        };
                        ui.colored_label(color, &recovery_state.message);
                    }

                    ui.separator();

                    if ui.button("📧 Send Recovery Email").clicked() {
                        let server_state = self.server_state.lock().unwrap();
                        let auth_manager = server_state.auth_manager.lock().unwrap();
                        match auth_manager.recover_credentials(&recovery_state.email) {
                            Ok(()) => {
                                recovery_state.message =
                                    "Recovery email sent! Check your inbox.".to_string();
                                recovery_state.is_success = true;
                            }
                            Err(e) => {
                                recovery_state.message = e;
                                recovery_state.is_success = false;
                            }
                        }
                    }

                    if ui.button("⬅️ Back to Login").clicked() {
                        action = AppAction::SwitchToLogin(LoginState {
                            username: String::new(),
                            password: String::new(),
                            email: String::new(),
                            error_message: String::new(),
                            show_recovery: false,
                        });
                    }
                });
            }

            AppState::SmtpConfig(smtp_state) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("📧 SMTP Configuration");
                    ui.separator();

                    ui.label("Configure email settings for password recovery:");

                    ui.horizontal(|ui| {
                        ui.label("SMTP Server:");
                        ui.text_edit_singleline(&mut smtp_state.server);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Port:");
                        ui.text_edit_singleline(&mut smtp_state.port);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut smtp_state.username);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut smtp_state.password).password(true));
                    });

                    ui.horizontal(|ui| {
                        ui.checkbox(&mut smtp_state.use_tls, "Use TLS");
                    });

                    if !smtp_state.message.is_empty() {
                        ui.colored_label(egui::Color32::GREEN, &smtp_state.message);
                    }

                    ui.separator();

                    if ui.button("💾 Save Configuration").clicked() {
                        match smtp_state.port.parse::<u16>() {
                            Ok(port) => {
                                let smtp_config = SmtpConfig {
                                    server: smtp_state.server.clone(),
                                    port,
                                    username: smtp_state.username.clone(),
                                    password: smtp_state.password.clone(),
                                    use_tls: smtp_state.use_tls,
                                };

                                let server_state = self.server_state.lock().unwrap();
                                let mut auth_manager = server_state.auth_manager.lock().unwrap();
                                match auth_manager.configure_smtp(smtp_config) {
                                    Ok(()) => {
                                        smtp_state.message =
                                            "SMTP configuration saved successfully!".to_string();
                                    }
                                    Err(e) => {
                                        smtp_state.message = format!("Error: {}", e);
                                    }
                                }
                            }
                            Err(_) => {
                                smtp_state.message = "Invalid port number".to_string();
                            }
                        }
                    }

                    if ui.button("⬅️ Back").clicked() {
                        action = AppAction::SwitchToSmtpConfig("admin".to_string());
                    }
                });
            }
        }
        match action {
            AppAction::SwitchToLogin(login_state) => {
                self.app_state = AppState::Login(login_state);
            }
            AppAction::SwitchToMain(main_state) => {
                self.app_state = AppState::Main(main_state);
            }
            AppAction::SwitchToRecovery => {
                self.app_state = AppState::Recovery(RecoveryState {
                    email: String::new(),
                    message: String::new(),
                    is_success: false,
                });
            }
            AppAction::SwitchToSmtpConfig(current_user) => {
                self.app_state = AppState::Main(MainState {
                    port_input: "3000".to_string(),
                    server_state: self.server_state.clone(),
                    status_message: String::new(),
                    current_user,
                });
            }
            AppAction::None => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for CLI mode flags
    let args: Vec<String> = env::args().collect();
    
    // Check for --cli, --no-gui, or daemon flags
    let cli_mode = args.iter().any(|arg| {
        matches!(arg.as_str(), "--cli" | "--no-gui" | "--daemon" | "daemon" | "start" | "stop" | "status")
    });

    if cli_mode {
        // Run in CLI mode
        run_cli()?;
        Ok(())
    } else {
        // Run in GUI mode
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_icon(std::sync::Arc::new(egui::IconData {
                rgba: image::load_from_memory(include_bytes!("../Assets/icon.png"))
                    .unwrap()
                    .to_rgba8()
                    .to_vec(),
                width: 250,
                height: 325,
            })),
            ..Default::default()
        };

        eframe::run_native(
            "Crusty Crawler",
            options,
            Box::new(|_cc| Ok(Box::<MyApp>::default())),
        )?;
        
        Ok(())
    }
}
