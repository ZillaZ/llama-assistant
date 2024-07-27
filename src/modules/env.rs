use dotenvy::dotenv;
use std::env;

pub struct Environment {
    api_key: String,
    ai_model: String
}

impl Environment {
    pub fn new() -> Self {
        init_env();
        let api_key = env::var("API_KEY").unwrap();
        let ai_model = env::var("AI_MODEL").unwrap();
        Self { api_key, ai_model }
    }
    pub fn get_key(&self) -> String {
        self.api_key.clone()
    }
    pub fn get_model(&self) -> String {
        self.ai_model.clone()
    }
}

fn init_env() {
    dotenv().expect("Unable to start environment.");
}
