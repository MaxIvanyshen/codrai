use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{Error, ErrorKind};

pub trait ModelClient {
    fn new(base_url: String, api_key: String, model: String) -> Self;
    fn chat_completion(&self, messages: &Vec<Message>, tools: Option<Box<Vec<Tool>>>) -> impl std::future::Future<Output = Result<ChatCompletion, Box<dyn std::error::Error>>> + Send;
}

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
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
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

impl ModelClient for  Client {
    fn new(base_url: String, api_key: String, model: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        let http_client = reqwest::Client::new();

        Client {
            http_client,
            api_key,
            base_url,
            model,
        }
    }

    async fn chat_completion(&self, messages: &Vec<Message>, tools: Option<Box<Vec<Tool>>>) -> Result<ChatCompletion, Box<dyn std::error::Error>> {
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
            .await?;

        if response.status().is_success() {
            let chat_completion: ChatCompletion = response.json().await?;
            Ok(chat_completion)
        } else {
            let status = response.status();
            let error_message = response.text().await?;
            Err(Box::new(Error::new(
                ErrorKind::Other,
                format!("Error {}: {}", status, error_message),
            )))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "tool")]
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
    pub id: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

pub fn simple_message(message: String, role: Role) -> Message {
    Message {
        role,
        content: message,
        tool_calls: None,
        tool_call_id: None,
    }
}

pub fn tool_call_result(id: String, result: String) -> Message {
    Message {
        role: Role::Tool,
        content: result,
        tool_calls: None,
        tool_call_id: Some(id),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    struct MockClient {
        tools: bool,
        tools_args: Option<HashMap<String, String>>,
    }

    impl ModelClient for MockClient {
        fn new(_: String, _: String, _: String) -> Self {
            MockClient { tools: false, tools_args: None }
        }

        fn chat_completion(&self, _: &Vec<Message>, tools: Option<Box<Vec<Tool>>>) -> impl std::future::Future<Output = Result<ChatCompletion, Box<dyn std::error::Error>>> + Send {
            let choices = if self.tools {
                // Create a tool call response
                let tool_call = ToolCall {
                    id: "call_123456789".to_string(),
                    function: FunctionCall {
                        name: tools.as_ref().unwrap()[0].function.name.clone(),
                        arguments: self.tools_args.as_ref().unwrap()[&tools.as_ref().unwrap()[0].function.name].clone(),
                    },
                };

                let message = Message {
                    role: Role::Assistant,
                    content: "".to_string(),
                    tool_calls: Some(vec![tool_call]),
                    tool_call_id: None,
                };

                vec![Choice { message }]
            } else {
                // Create a text response
                let message = Message {
                    role: Role::Assistant,
                    content: "This is a mock response from the assistant.".to_string(),
                    tool_calls: None,
                    tool_call_id: None,
                };

                vec![Choice { message }]
            };

            // Return the future with the prepared choices
            async { Ok(ChatCompletion { choices }) }}
    }

    fn setup_client(tools: bool, tools_args: Option<HashMap<String, String>>) -> impl ModelClient {
        MockClient {
            tools,
            tools_args,
        }
    }

    #[tokio::test]
    async fn test_chat_completion() {
        let messages = vec![
            simple_message("Hello, how are you?".to_string(), Role::User),
            simple_message("I'm fine, thank you!".to_string(), Role::Assistant),
        ];

        match setup_client(false, None).chat_completion(&messages, None).await {
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

        let mut tools_args = HashMap::new();
        tools_args.insert("get_weather".to_string(), "{\"location\":\"london\"}".to_string());

        match setup_client(true, Some(tools_args)).chat_completion(&messages, Some(Box::new(vec![weather_tool]))).await {
            Ok(response) => {
                assert!(!response.choices.is_empty());
                assert!(!response.choices[0].message.tool_calls.is_none());
                for choice in response.choices {
                    if let Some(tool_calls) = choice.message.tool_calls {
                        for tool_call in tool_calls {
                            assert_eq!(tool_call.function.name, "get_weather");
                            assert_eq!(tool_call.function.arguments.to_lowercase(), "{\"location\":\"london\"}");
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Error: {}", e);
            }
        }
    }
}
