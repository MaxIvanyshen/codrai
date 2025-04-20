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
                openai::Message {
                    role: openai::Role::System,
                    content: system_prompt,
                },
            ],
        }
    }

    pub async fn message(&mut self, message: String) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.messages.push(openai::Message {
            role: openai::Role::User,
            content: message,
        });
        let mut result = vec![];
        match self.openai_client.chat_completion(&self.messages).await {
            Ok(response) => {
                for choice in response.choices {
                    result.push(choice.message.content);
                }
                self.messages.push(openai::Message {
                    role: openai::Role::Assistant,
                    content: result.join("\n"),
                });
                Ok(result)
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(e)
            }
        }

    }
}
