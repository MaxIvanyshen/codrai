use std::fs;
use openai::Tool as OpenAITool;

pub struct ToolBox {
    tools: Vec<Box<Tool>>,
}

impl ToolBox {
    pub fn new() -> Self {
        ToolBox {
            tools: vec![
                Box::new(new_write_file_tool()),
                Box::new(new_edit_file_tool()),
                Box::new(new_read_file_tool()),
            ],
        }
    }

    pub fn run_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let tool = self.tools.iter().find(|tool| tool.name() == name).map(|tool| tool.as_ref());
        match tool {
            Some(tool) => {
                tool.run(args).map_err(|e| {
                    eprintln!("Error running tool {}: {}", name, e);
                    e
                })
            }
            None => Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Tool not found"))),
        }
    }

    pub fn get_tools(&self) -> Vec<OpenAITool> {
        self.tools.iter().map(|tool| tool.to_openai_tool()).collect()
    }
}

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

pub fn new_write_file_tool() -> Tool {
    Tool {
        name: "write_file".to_string(),
        description: "Writes content to a file".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        }),
        runner: |args| {
            println!("Using write file tool"); 
            let file_path = args["file_path"].as_str().ok_or("file_path is required")?;
            let content = args["content"].as_str().ok_or("content is required")?;

            fs::write(file_path, content)?;
            Ok(serde_json::json!({"status": "success"}))
        },
    }
}

pub fn new_edit_file_tool() -> Tool {
    Tool {
        name: "edit_file".to_string(),
        description: "Replaces content of a file with a new one".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        }),
        runner: |args| {
            let file_path = args["file_path"].as_str().ok_or("file_path is required")?;
            let content = args["content"].as_str().ok_or("content is required")?;

            fs::write(file_path, content)?;
            Ok(serde_json::json!({"status": "success"}))
        },
    }
}

pub fn new_read_file_tool() -> Tool {
    Tool {
        name: "read_file".to_string(),
        description: "Reads content from a file".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to read"
                }
            },
            "required": ["file_path"]
        }),
        runner: |args| {
            let file_path = args["file_path"].as_str().ok_or("file_path is required")?;
            let content = fs::read_to_string(file_path)?;
            Ok(serde_json::json!({"content": content}))
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations() {
        let args = serde_json::json!({
            "file_path": "./test.txt",
            "content": "Hello, world!"
        });

        let result = new_write_file_tool().run(args).unwrap();
        assert_eq!(result["status"], "success");


        let edit_args = serde_json::json!({
            "file_path": "./test.txt",
            "content": "Hello, Rust!"
        });
        let result = new_edit_file_tool().run(edit_args).unwrap();
        assert_eq!(result["status"], "success");

        let read_result = new_read_file_tool().run(serde_json::json!({
            "file_path": "./test.txt"
        })).unwrap();
        assert_eq!(read_result["content"], "Hello, Rust!");
    }

    #[test]
    fn test_to_openai_tool() {
        let tool = new_write_file_tool();
        let openai_tool = tool.to_openai_tool();

        assert_eq!(openai_tool.tool_type, "function");
        assert_eq!(openai_tool.function.name, tool.name());
        assert_eq!(openai_tool.function.description, tool.description());
    }

    #[test]
    fn test_tool_box() {
        let toolbox = ToolBox::new();
        let tools = toolbox.get_tools();

        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].function.name, "write_file");
        assert_eq!(tools[1].function.name, "edit_file");
        assert_eq!(tools[2].function.name, "read_file");

        let args = serde_json::json!({
            "file_path": "./toolbox.txt",
            "content": "Hello, world!"
        });
        let result = toolbox.run_tool("write_file", args).unwrap();
        assert_eq!(result["status"], "success");
        let read_result = toolbox.run_tool("read_file", serde_json::json!({
            "file_path": "./toolbox.txt"
        })).unwrap();
        assert_eq!(read_result["content"], "Hello, world!");
    }
}
