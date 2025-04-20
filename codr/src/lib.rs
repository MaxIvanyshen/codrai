use std::{env, fs};

pub struct Codr {
    openai_client: openai::Client,
    messages: Vec<openai::Message>,
}

impl Codr {
    pub fn new() -> Self {
        let base_url = env::var("CODR_BASE_URL").expect("CODR_BASE_URL must be set");
        let api_key = env::var("CODR_API_KEY").expect("CODR_API_KEY must be set");
        let model = env::var("CODR_MODEL").expect("CODR_MODEL must be set");

        let system_prompt = fs::read_to_string("./system_prompt.md")
            .expect("Unable to read system prompt file");

        Codr {
            openai_client: openai::Client::new(base_url, api_key, model),
            messages: vec![
                openai::simple_message(system_prompt, openai::Role::System),
            ],
        }
    }

    pub async fn message(&mut self, message: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.messages.push(openai::simple_message(message, openai::Role::User));
        let mut result = vec![];
        match self.openai_client.chat_completion(&self.messages, None).await {
            Ok(response) => {
                for choice in response.choices {
                    result.push(choice.message.content);
                }
                self.messages.push(
                    openai::simple_message(result.join("\n"), openai::Role::Assistant)
                );
                Ok(result)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }
    }
}
