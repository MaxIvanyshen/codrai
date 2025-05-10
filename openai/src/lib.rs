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
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
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
    #[serde(rename = "type")]
    pub tool_type: String,
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
        content: Some(message),
        tool_calls: None,
        tool_call_id: None,
    }
}

pub fn tool_call_result(id: String, result: String) -> Message {
    Message {
        role: Role::Tool,
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
                assert!(!response.choices[0].message.tool_calls.is_none());
                for choice in response.choices {
                    if let Some(tool_calls) = choice.message.tool_calls {
                        for tool_call in tool_calls {
                            assert_eq!(tool_call.function.name, "get_weather");
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
    async fn test_unmarshalling() {
        let json = r#"
        {
  "id": "chatcmpl-BVeccwKqpXDrZUzxGnk8HfpEnrmsR",
  "object": "chat.completion",
  "created": 1746884018,
  "model": "gpt-4.1-nano-2025-04-14",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_JzXPC0vOC8ixB8HtD16272lC",
            "type": "function",
            "function": {
              "name": "write_file",
              "arguments": "{\"content\":\"package main\\n\\nimport \\\"fmt\\\"\\n\\nfunc main() {\\n    fmt.Println(\\\"Hello, World!\\\")\\n}\\n\",\"file_path\":\"hello_world.go\"}"
            }
          }
        ],
        "refusal": null,
        "annotations": []
      },
      "logprobs": null,
      "finish_reason": "tool_calls"
    }
  ],
  "usage": {
    "prompt_tokens": 786,
    "completion_tokens": 49,
    "total_tokens": 835,
    "prompt_tokens_details": {
      "cached_tokens": 0,
      "audio_tokens": 0
    },
    "completion_tokens_details": {
      "reasoning_tokens": 0,
      "audio_tokens": 0,
      "accepted_prediction_tokens": 0,
      "rejected_prediction_tokens": 0
    }
  },
  "service_tier": "default",
  "system_fingerprint": "fp_8fd43718b3"
}"#;
    
    let parsed = serde_json::from_str(json);
    match parsed {
        Err(e) => {
            panic!("Error parsing JSON: {}", e);
        }
        Ok(parsed) => {

    let parsed: ChatCompletion = parsed;
    assert_eq!(parsed.choices.len(), 1);
    assert!(parsed.choices[0].message.tool_calls.is_some());
    if let Some(tool_calls) = &parsed.choices[0].message.tool_calls {
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "write_file");
        assert_eq!(tool_calls[0].function.arguments, "{\"content\":\"package main\\n\\nimport \\\"fmt\\\"\\n\\nfunc main() {\\n    fmt.Println(\\\"Hello, World!\\\")\\n}\\n\",\"file_path\":\"hello_world.go\"}");
    } else {
        panic!("No tool calls found");
    }
        }

    }
    }
}
