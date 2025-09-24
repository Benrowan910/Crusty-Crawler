//Authored by Benjamin Rowan
// Created for Nagios Enterprises LLC During the 2025 Summer Nintern Program
// The goal is to understand and create from the ground up, a server side monitoring agent that posts information
// to a server that can than be used to determine the health of the system you want to monitor.
//
// Ultimately the goal is to hook this up with custom XI plugins.

use eframe::egui;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

// Axum Server Components
use axum::{Router, response::Html, routing::get};
use tower_http::services::ServeDir;

// Includes
include!("network.rs");
include!("components.rs");
include!("disks.rs");
include!("hardware_statistics.rs");

// Shared state between GUI and server
struct ServerState {
    is_running: bool,
    port: u16,
    shutdown_sender: Option<tokio::sync::oneshot::Sender<()>>,
    hardware_state: Arc<Mutex<HardwareMonitorState>>,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            is_running: false,
            port: 3000,
            shutdown_sender: None,
            hardware_state: Arc::new(Mutex::new(HardwareMonitorState::default())),
        }
    }
}

struct MyApp {
    port_input: String,
    server_state: Arc<Mutex<ServerState>>,
    status_message: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            port_input: "3000".to_string(),
            server_state: Arc::new(Mutex::new(ServerState::default())),
            status_message: String::new(),
        }
    }
}

impl MyApp {
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

    //function to send the shutdown signal to the server
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

// Axum apllication and routing of information
fn create_app(server_state: Arc<Mutex<ServerState>>) -> Router {
    Router::new()
        .route("/api/status", get(move || status_handler(server_state)))
        .fallback_service(ServeDir::new("public"))
}

// Endpoint handler
async fn status_handler(server_state: Arc<Mutex<ServerState>>) -> Html<String> {
    Html(status(server_state).await)
}

// Display the system statistics collected
async fn status(server_state: Arc<Mutex<ServerState>>) -> String {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();

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

    out
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Crusty Server Configuration");

            let (is_running, current_port, last_update) = {
                let state = self.server_state.lock().unwrap();
                let hardware_state = state.hardware_state.lock().unwrap();
                let last_update = hardware_state.last_update.elapsed().as_secs();
                (state.is_running, state.port, last_update)
            }; // Releases the lock intrinsically

            // Port configuration
            ui.horizontal(|ui| {
                ui.label("Port:");
                ui.text_edit_singleline(&mut self.port_input).enabled();
            });

            // Control buttons
            ui.horizontal(|ui| {
                if !is_running {
                    if ui.button("🚀 Host Server").clicked() {
                        self.start_server();
                    }
                } else {
                    if ui.button("🛑 Stop Server").clicked() {
                        self.stop_server();
                    }
                }
            });

            // Status display
            if !self.status_message.is_empty() {
                ui.separator();
                ui.label(&self.status_message);
            }

            if is_running {
                ui.separator();
                ui.label("📊 Server Information:");
                ui.label(format!("• Port: {}", current_port));
                ui.label(format!("• Local URL: http://localhost:{}", current_port));
                ui.label(format!("• Network URL: http://[YOUR-IP]:{}", current_port));
                ui.label("• Replace [YOUR-IP] with your computer's IP address");
                ui.label("• Accessible from any device on your network!");

                // Show hardware monitoring status
                ui.separator();
                ui.label("🔧 Hardware Monitoring:");
                ui.label(format!("• Last updated: {} seconds ago", last_update));
                ui.label("• Power and thermal data refreshes every 60s");
            }

            // Instructions
            ui.separator();
            ui.label("💡 Instructions:");
            ui.label("1. Enter a port number (default: 3000)");
            ui.label("2. Click 'Host Server' to start");
            ui.label("3. Access the status page from any browser");
            ui.label("4. Use 'Stop Server' to shut down");
        });
    }
}

fn main() -> Result<(), eframe::Error> {
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
        "Crusty Server",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}
