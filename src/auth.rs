// auth.rs - Updated with user-defined access token
use bcrypt::{DEFAULT_COST, hash, verify};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub access_token: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthConfig {
    pub users: HashMap<String, User>, // username -> User
    pub smtp_config: Option<SmtpConfig>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SmtpConfig {
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            smtp_config: None,
        }
    }
}

pub struct AuthManager {
    config_path: String,
    pub config: AuthConfig,
}

impl AuthManager {
    pub fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_manager = if Path::new(config_path).exists() {
            let config_data = fs::read_to_string(config_path)?;
            let config: AuthConfig = serde_json::from_str(&config_data)?;
            Self {
                config_path: config_path.to_string(),
                config,
            }
        } else {
            let auth_manager = Self {
                config_path: config_path.to_string(),
                config: AuthConfig::default(),
            };
            auth_manager.save_config()?;
            auth_manager
        };

        Ok(auth_manager)
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_data = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, config_data)?;
        Ok(())
    }

    pub fn register_user(
        &mut self,
        username: &str,
        password: &str,
        email: &str,
        access_token: &str,
    ) -> Result<(), String> {
        if self.config.users.contains_key(username) {
            return Err("Username already exists".to_string());
        }

        if username.len() < 3 {
            return Err("Username must be at least 3 characters".to_string());
        }

        if password.len() < 8 {
            return Err("Password must be at least 8 characters".to_string());
        }

        if access_token.len() < 8 {
            return Err("Access token must be at least 8 characters".to_string());
        }

        // Check if access token is already in use
        for user in self.config.users.values() {
            if user.access_token == access_token {
                return Err("Access token already in use".to_string());
            }
        }

        let password_hash = hash(password, DEFAULT_COST).map_err(|e| e.to_string())?;
        let created_at = chrono::Utc::now().to_rfc3339();

        let user = User {
            username: username.to_string(),
            email: email.to_string(),
            password_hash,
            access_token: access_token.to_string(),
            created_at,
        };

        self.config.users.insert(username.to_string(), user);
        self.save_config().map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn authenticate(&self, username: &str, password: &str) -> Result<String, String> {
        if let Some(user) = self.config.users.get(username) {
            if verify(password, &user.password_hash).map_err(|e| e.to_string())? {
                Ok(user.access_token.clone())
            } else {
                Err("Invalid password".to_string())
            }
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<String, String> {
        for user in self.config.users.values() {
            if user.access_token == token {
                return Ok(user.username.clone());
            }
        }
        Err("Invalid access token".to_string())
    }

    pub fn recover_credentials(&self, email: &str) -> Result<(), String> {
        let user = self
            .config
            .users
            .values()
            .find(|u| u.email == email)
            .ok_or("No user found with that email address")?;

        if let Some(smtp_config) = &self.config.smtp_config {
            self.send_recovery_email(user, smtp_config)
        } else {
            Err("Email configuration not set up. Please contact administrator.".to_string())
        }
    }

    fn send_recovery_email(&self, user: &User, smtp_config: &SmtpConfig) -> Result<(), String> {
        println!("=== RECOVERY EMAIL ===");
        println!("To: {}", user.email);
        println!("Subject: Crusty Server Credentials Recovery");
        println!();
        println!("Hello {},", user.username);
        println!();
        println!("Here are your Crusty Server credentials:");
        println!("Username: {}", user.username);
        println!("Access Token: {}", user.access_token);
        println!();
        println!("Use the username and password to log into the application.");
        println!("Use the access token to access the web interface.");
        println!();
        println!("If you didn't request this, please ignore this message.");
        println!("=== END EMAIL ===");

        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub fn configure_smtp(&mut self, smtp_config: SmtpConfig) -> Result<(), String> {
        self.config.smtp_config = Some(smtp_config);
        self.save_config().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn has_users(&self) -> bool {
        !self.config.users.is_empty()
    }

    pub fn generate_suggested_token() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789";
        let mut rng = rand::rng();

        let token: String = (0..16)
            .map(|_| {
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        token
    }
}
