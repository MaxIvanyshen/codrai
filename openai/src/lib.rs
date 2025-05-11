use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{Error, ErrorKind};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "tool")]
    Tool
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    #[serde(rename = "role")]
    pub role: Option<Role>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Choice {
    pub message: Option<Message>,
    pub delta: Option<Message>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChunk {
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StreamChannelChunk {
    pub finished: bool,
    pub final_content: Option<String>,
    pub choices: Vec<Choice>,
}


#[derive(Debug, Clone)]
pub struct OpenAIClient {
    http_client: reqwest::Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAIClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let http_client = reqwest::Client::new();

        OpenAIClient {
            http_client,
            api_key,
            base_url,
            model,
        }
    }

    pub async fn chat_completion(&self, messages: &Vec<Message>, tools: Option<Box<Vec<Tool>>>) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": tools,
        });

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await;

        if let Err(e) = response {
            eprintln!("Request failed: {}", e);
            return Err(Box::new(Error::new(ErrorKind::Other, "Request failed")));
        }

        let response = response.unwrap();

        if response.status().is_success() {
            let chat_completion = response.json().await;
            if let Err(e) = chat_completion {
                eprintln!("Failed to parse response: {}", e);
                return Err(Box::new(Error::new(ErrorKind::Other, "Failed to parse response")));
            }
            Ok(chat_completion.unwrap())
        } else {
            let status = response.status();
            let error_message = response.text().await?;
            eprintln!("Error: {}", error_message);
            Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Error {}: {}", status, error_message),
            )))
        }
    }

    pub async fn chat_completion_stream(&self, messages: &Vec<Message>, tools: Option<Box<Vec<Tool>>>) -> tokio::sync::mpsc::Receiver<StreamChannelChunk> {
        let url = format!("{}/chat/completions", self.base_url);
        
        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "tools": tools,
            "stream": true,
        });

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await.unwrap();

        let mut response = response;
        let mut all_content = String::new();

        let (tx, rx) = tokio::sync::mpsc::channel::<StreamChannelChunk>(1);

        if response.status() != reqwest::StatusCode::OK {
            panic!("Error ({}): {}", response.status(), response.text().await.unwrap());
        }

        tokio::spawn(async move {
            let mut tool_call: Option<ToolCall> = None;
            loop {
                let chunk = response.chunk().await;
                match chunk {
                    Ok(Some(data)) => {
                        let chunk_str = String::from_utf8_lossy(&data);

                        //println!("Received chunk: {}", chunk_str);
                        
                        for line in chunk_str.lines() {
                            if line.starts_with("data: ") {
                                let json_str = line.trim_start_matches("data: ");
                                
                                // Check if it's the end marker
                                if json_str == "[DONE]" {
                                    continue;
                                }

                                // Parse the JSON chunk
                                match serde_json::from_str::<StreamChunk>(json_str) {
                                    Ok(stream_chunk) => {
                                        for choice in stream_chunk.choices {
                                            all_content.push_str(&choice.delta.clone().unwrap().content.unwrap_or_default());

                                            if choice.finish_reason.is_some() {
                                                match choice.finish_reason.as_deref() {
                                                    Some("stop") => {
                                                        let final_content = all_content.clone();
                                                        tx.send(StreamChannelChunk {
                                                            finished: true,
                                                            final_content: Some(final_content),
                                                            choices: vec![],
                                                        }).await.unwrap();
                                                    }
                                                    Some("tool_calls") => {
                                                        println!("Tool calls detected");
                                                        // Send the tool call to the channel
                                                        tx.send(StreamChannelChunk {
                                                            finished: false,
                                                            final_content: None,
                                                            choices: vec![Choice {
                                                                delta: Some(Message {
                                                                    role: Some(Role::Assistant),
                                                                    content: None,
                                                                    tool_calls: Some(vec![tool_call.clone().unwrap()]),
                                                                    tool_call_id: None,
                                                                }),
                                                                message: None,
                                                                finish_reason: None,
                                                            }],
                                                        }).await.unwrap();
                                                    }
                                                    _ => {}
                                                }
                                                break;
                                            }

                                            if let Some(delta) = choice.clone().delta {
                                                if let Some(tool_calls) = delta.tool_calls.clone() {
                                                    if let Some(curr_call) = tool_calls.get(0) {
                                                        match tool_call {
                                                            Some(ref mut call) => {
                                                                call.function.arguments.push_str(curr_call.function.arguments.as_str());
                                                            }
                                                            None => {
                                                                tool_call = Some(curr_call.clone());
                                                            }
                                                        }
                                                        continue;
                                                    }
                                                }
                                            }

                                            tx.send(StreamChannelChunk {
                                                finished: false,
                                                final_content: None,
                                                choices: vec![choice.clone()],
                                            }).await.unwrap();
                                        }
                                    },
                                    Err(_) => {}
                                }
                            }
                        }
                    },
                    Ok(None) => break, // End of stream
                    Err(e) => {
                        eprintln!("Error reading chunk: {}", e);
                    }
                }
            }
        });

        rx
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: Function, 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Function {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "index", skip_serializing_if = "Option::is_none")]
    pub index: Option<usize>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,

    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: Option<String>,
    pub arguments: String,
}

pub fn simple_message(message: String, role: Role) -> Message {
    Message {
        role: Some(role),
        content: Some(message),
        tool_calls: None,
        tool_call_id: None,
    }
}

pub fn tool_call_result(id: String, result: String) -> Message {
    Message {
        role: Some(Role::Tool),
        content: Some(result),
        tool_calls: None,
        tool_call_id: Some(id),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use super::*;

    fn setup_client() -> OpenAIClient {
        let base_url = env::var("CODR_BASE_URL").expect("CODR_BASE_URL must be set");
        let api_key = env::var("CODR_API_KEY").expect("CODR_API_KEY must be set");
        let model = env::var("CODR_MODEL").expect("CODR_MODEL must be set");

        OpenAIClient::new(base_url, api_key, model)
    }

    #[tokio::test]
    async fn test_chat_completion() {
        let messages = vec![
            simple_message("Hello, how are you?".to_string(), Role::User),
            simple_message("I'm fine, thank you!".to_string(), Role::Assistant),
        ];

        match setup_client().chat_completion(&messages, None).await {
            Ok(response) => {
                assert!(!response.choices.is_empty());
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_tool_calls() {
        let weather_tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: "Get the current weather at the location".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The location to get the weather for"
                        },
                    },
                }),
            },
        };

        let messages = vec![
            simple_message("What is the weather in London?".to_string(), Role::User),
        ];

        match setup_client().chat_completion(&messages, Some(Box::new(vec![weather_tool]))).await {
            Ok(response) => {
                assert!(!response.choices.is_empty());
                assert!(!response.choices[0].message.clone().unwrap().tool_calls.is_none());
                for choice in response.choices {
                    if let Some(tool_calls) = choice.message.unwrap().tool_calls {
                        for tool_call in tool_calls {
                            assert_eq!(tool_call.function.name.unwrap(), "get_weather".to_string());
                            assert_eq!(tool_call.function.arguments.replace(" ", "").to_lowercase(), "{\"location\":\"london\"}");
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_streaming() {
        let messages = vec![
            simple_message("Hello, how are you?".to_string(), Role::User),
        ];

        let client = setup_client();
        let mut stream = client.chat_completion_stream(&messages, None).await;
        let mut all_content = String::new();
        while let Some(chunk) = stream.recv().await {
            if chunk.finished {
                if let Some(content) = chunk.final_content {
                    all_content = content;
                }
                break;
            } else {
                for choice in chunk.choices {
                    println!("Received chunk: {:?}", choice.delta);
                }
            }
        }

        assert!(!all_content.is_empty());
        println!("Streamed content: {}", all_content);
    }

    #[tokio::test]
    async fn test_tool_calls_streaming() {
        let weather_tool = Tool {
            tool_type: "function".to_string(),
            function: Function {
                name: "get_weather".to_string(),
                description: "Get the current weather at the location".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The location to get the weather for"
                        },
                    },
                    "required": ["location"],
                }),
            },
        };

        let messages = vec![
            simple_message("What is the weather in London?".to_string(), Role::User),
        ];

        let client = setup_client();
        let mut stream = client.chat_completion_stream(&messages, Some(Box::new(vec![weather_tool]))).await;

        while let Some(chunk) = stream.recv().await {
            if chunk.finished {
                break;
            } else {
                for choice in chunk.choices {
                    println!("Received chunk: {:?}", choice.delta);
                    if let Some(delta) = choice.delta {
                        if delta.tool_calls.is_some() {
                            for tool_call in delta.tool_calls.unwrap() {
                                println!("Tool call: {:?}", tool_call);
                                assert_eq!(tool_call.function.name.unwrap(), "get_weather".to_string());
                                assert_eq!(tool_call.function.arguments.replace(" ", "").to_lowercase(), "{\"location\":\"london\"}");
                            }
                        }
                    }
                }
            }
        }
    }
}
