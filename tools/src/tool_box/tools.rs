pub mod file_tools;

use openai::Tool as OpenAITool;

pub struct Tool {
    name: String,
    description: String,
    parameters: serde_json::Value,
    runner: fn(serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>>,
}

impl Tool {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn parameters(&self) -> &serde_json::Value {
        &self.parameters
    }

    pub fn run(&self, args: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        (self.runner)(args)
    }

    pub fn to_openai_tool(&self) -> OpenAITool {
        OpenAITool {
            tool_type: "function".to_string(),
            function: openai::Function {
                name: self.name.clone(),
                description: self.description.clone(),
                parameters: self.parameters.clone(),
            },
        }
    }
}
