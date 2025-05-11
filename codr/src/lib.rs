use std::{env, fs, sync::{Arc, Mutex}};
use tools::tool_box::ToolBox as ToolBox;

pub struct Codr {
    openai_client: openai::OpenAIClient,
    messages: Arc<Mutex<Vec<openai::Message>>>,
    toolbox: ToolBox,
}

impl Codr {
    pub fn new() -> Self {
        let base_url = env::var("CODR_BASE_URL").expect("CODR_BASE_URL must be set");
        let api_key = env::var("CODR_API_KEY").expect("CODR_API_KEY must be set");
        let model = env::var("CODR_MODEL").expect("CODR_MODEL must be set");

        let system_prompt = fs::read_to_string("system_prompt.md")
            .expect("Unable to read system prompt file");

        let messages = vec![
            openai::simple_message(system_prompt, openai::Role::System),
        ];

        Codr {
            openai_client: openai::OpenAIClient::new(base_url, api_key, model),
            messages: Arc::new(Mutex::new(messages)),
            toolbox: ToolBox::new(),
        }
    }

    pub async fn message(&mut self, message: String) -> Result<Vec<Option<String>>, Box<dyn std::error::Error>> {
        let mut msg_lock = self.messages.lock().unwrap();
        msg_lock.push(openai::simple_message(message, openai::Role::User));
        let mut results = Vec::new();
        
        loop {
            let response = match self.openai_client.chat_completion(
                &msg_lock, 
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
            
            let has_tool_calls = choice.message.clone().unwrap().tool_calls.as_ref()
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);
                
            msg_lock.push(choice.message.clone().unwrap());
            
            if has_tool_calls {
                let msg = choice.message.clone().unwrap();
                let tool_calls = msg.tool_calls.as_ref().unwrap();
                
                for tool_call in tool_calls {
                    println!("Processing tool call: {}", 
                             tool_call.function.name.clone().unwrap());
                    println!("Arguments: {}", tool_call.function.arguments.clone());
                    
                    let args = match serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments.clone()) {
                        Ok(args) => args,
                        Err(e) => {
                            eprintln!("Error parsing arguments: {}", e);

                            // Add error message as tool result
                            let error_result = serde_json::json!({"error": format!("Failed to parse arguments: {}", e)});
                            msg_lock.push(openai::tool_call_result(
                                tool_call.id.clone().unwrap(), 
                                error_result.to_string()
                            ));
                            continue;
                        }
                    };
                    
                    let result = match self.toolbox.run_tool(&tool_call.function.name.clone().unwrap(), args) {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("Error running tool: {}", e);
                            serde_json::json!({"error": e.to_string()})
                        }
                    };
                    
                    msg_lock.push(openai::tool_call_result(
                        tool_call.id.clone().unwrap(), 
                        result.to_string()
                    ));
                }
                
                // Continue the loop to get the final response
                continue;
            } else {
                // If there are no tool calls, add the content to results
                results.push(choice.message.clone().unwrap().content.clone());
                break;
            }
        }
        
        Ok(results)
    }


    pub async fn message_stream(&self, message: String) -> tokio::sync::mpsc::Receiver<String> {
        let mut msg_lock = self.messages.lock().unwrap();
        msg_lock.push(openai::simple_message(message, openai::Role::User));

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let mut curr_msg = msg_lock.clone();

        drop(msg_lock); // Drop the lock to allow other threads to access it

        let openai_client = self.openai_client.clone();
        let toolbox = self.toolbox.clone();

        let msg_arc = self.messages.clone();

        tokio::spawn(async move {
            'stream: loop {
                let mut chunk_receiver = Box::new(openai_client.chat_completion_stream(
                        &curr_msg, 
                        Some(Box::new(toolbox.get_tools()))
                ).await);

                while let Some(chunk) = chunk_receiver.recv().await {
                    if chunk.finished {
                        curr_msg.push(openai::simple_message(
                            chunk.final_content.unwrap(),
                            openai::Role::Assistant
                        ));

                        let mut msg_lock = msg_arc.lock().unwrap();
                        *msg_lock = curr_msg.clone();
                        break 'stream;
                    }

                    for choice in chunk.choices {
                        if let Some(message) = choice.delta {
                            if let Some(tool_calls) = message.clone().tool_calls {
                                curr_msg.push(message.clone());

                                for tool_call in tool_calls {
                                    println!("Processing tool call: {}", 
                                        tool_call.function.name.clone().unwrap());
                                    println!("Arguments: {}", tool_call.function.arguments.clone());

                                    let args = match serde_json::from_str::<serde_json::Value>(&tool_call.function.arguments.clone()) {
                                        Ok(args) => args,
                                        Err(e) => {
                                            eprintln!("Error parsing arguments: {}", e);
                                            continue;
                                        }
                                    };

                                    let result = match toolbox.run_tool(&tool_call.function.name.clone().unwrap(), args) {
                                        Ok(res) => res,
                                        Err(e) => {
                                            eprintln!("Error running tool: {}", e);
                                            serde_json::json!({"error": e.to_string()})
                                        }
                                    };

                                    curr_msg.push(openai::tool_call_result(
                                            tool_call.id.clone().unwrap(), 
                                            result.to_string()
                                    ));
                                    continue 'stream;
                                }
                            }
                            if let Some(content) = message.content {
                                if let Err(e) = tx.send(content.clone()).await {
                                    eprintln!("Error sending message: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        });

        rx
    }
}
