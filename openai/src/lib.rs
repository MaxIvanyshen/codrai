use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{ErrorKind};

#[derive(Debug, Serialize, Deserialize)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String, // Changed from Role enum to String for simplicity
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
}

pub struct Client {
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl Client {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let http_client = reqwest::Client::new();

        Client {
            http_client,
            api_key,
            base_url,
            model,
        }
    }

    pub async fn chat_completion(&self, messages: Vec<Message>) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
        });

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)  // This serializes the body to JSON
            .send()
            .await?;

        if response.status().is_success() {
            let chat_completion: ChatCompletion = response.json().await?;
            Ok(chat_completion)
        } else {
            let status = response.status();
            let error_message = response.text().await?;
            Err(Box::new(std::io::Error::new(
                ErrorKind::Other,
                format!("Error {}: {}", status, error_message),
            )))
        }
    }
}
