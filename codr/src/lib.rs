use std::{env, fs};

pub struct Codr {
    openai_client: openai::OpenAIClient,
    messages: Vec<openai::Message>,
    toolbox: tools::ToolBox,
}

impl Codr {
    pub fn new() -> Self {
        let base_url = env::var("CODR_BASE_URL").expect("CODR_BASE_URL must be set");
        let api_key = env::var("CODR_API_KEY").expect("CODR_API_KEY must be set");
        let model = env::var("CODR_MODEL").expect("CODR_MODEL must be set");

        let system_prompt = fs::read_to_string("./system_prompt.md")
            .expect("Unable to read system prompt file");

        Codr {
            openai_client: openai::OpenAIClient::new(base_url, api_key, model),
            messages: vec![
                openai::simple_message(system_prompt, openai::Role::System),
            ],
            toolbox: tools::ToolBox::new(),
        }
    }

    pub async fn message(&mut self, message: String) -> Result<Vec<Option<String>>, Box<dyn std::error::Error>> {
        self.messages.push(openai::simple_message(message, openai::Role::User));
        let mut results = Vec::new();
        
        loop {
            let response = match self.openai_client.chat_completion(
                &self.messages, 
                Some(Box::new(self.toolbox.get_tools()))
            ).await {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("API Error: {}", e);
                    return Err(e);
                }
            };
            
            if response.choices.is_empty() {
                return Err("No choices returned from API".into());
            }
            
            let choice = &response.choices[0];
            
            let has_tool_calls = choice.message.tool_calls.as_ref()
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);
                
            self.messages.push(choice.message.clone());
            
            if has_tool_calls {
                let tool_calls = choice.message.tool_calls.as_ref().unwrap();
                
                for tool_call in tool_calls {
                    println!("Processing tool call: {} (id: {})", 
                             tool_call.function.name, tool_call.id);
                    println!("Arguments: {}", tool_call.function.arguments);
                    
                    let args = match serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments) {
                        Ok(args) => args,
                        Err(e) => {
                            eprintln!("Error parsing arguments: {}", e);

                            // Add error message as tool result
                            let error_result = serde_json::json!({"error": format!("Failed to parse arguments: {}", e)});
                            self.messages.push(openai::tool_call_result(
                                tool_call.id.clone(), 
                                error_result.to_string()
                            ));
                            continue;
                        }
                    };
                    
                    let result = match self.toolbox.run_tool(&tool_call.function.name, args) {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("Error running tool: {}", e);
                            serde_json::json!({"error": e.to_string()})
                        }
                    };
                    
                    self.messages.push(openai::tool_call_result(
                        tool_call.id.clone(), 
                        result.to_string()
                    ));
                }
                
                // Continue the loop to get the final response
                continue;
            } else {
                // If there are no tool calls, add the content to results
                results.push(choice.message.content.clone());
                break;
            }
        }
        
        Ok(results)
    }
}
