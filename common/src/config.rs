use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub database_url: String,
}

impl Config {
    pub fn new() -> Self {
        Envconfig::init().expect("failed to read config from env")
    }

    pub fn verify() {
        Self::new();
    }
}
