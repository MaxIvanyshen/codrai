use openai;
use tokio;
use std::env;
use termimad::MadSkin;

#[tokio::main]
async fn main() {

    let base_url = "https://openrouter.ai/api/v1";
    let api_key = env::var("OPENROUTER_KEY").unwrap();
    let model = "deepseek/deepseek-chat-v3-0324:free";
    let client = openai::Client::new(base_url.to_string(), api_key, model.to_string());

    let messages = vec![
        openai::Message {
            role: "system".to_string(),
            content: "Your name is Chatty and you're my best friend".to_string(),
        },
        openai::Message {
            role: "user".to_string(),
            content: "Who are you?".to_string(),
        },
    ];

    let skin = MadSkin::default();

    match client.chat_completion(messages).await {
        Ok(response) => {
            for choice in response.choices {
                skin.print_text(&choice.message.content);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
