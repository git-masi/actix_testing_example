#[derive(Debug)]
pub struct ServerConfig {
    pub host_name: String,
    pub port: u16,
}

impl ServerConfig {
    pub fn new() -> Self {
        Self {
            host_name: std::env::var("HOST_NAME").unwrap_or("127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .unwrap_or("8080".to_string())
                .parse::<u16>()
                .unwrap(),
        }
    }

    pub fn to_address(&self) -> (String, u16) {
        (self.host_name.clone(), self.port)
    }
}
