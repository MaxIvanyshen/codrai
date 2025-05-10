pub mod tools;

use openai::Tool as OpenAITool;

use crate::tool_box::tools::{Tool, file_tools::{
    new_append_to_file_tool, new_create_folder_tool, new_get_folder_files_tool,
    new_read_file_tool, new_replace_file_tool, new_write_file_tool,
}};

pub struct ToolBox {
    tools: Vec<Box<Tool>>,
}

pub fn status_success() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({"status": "success"}))
}

pub fn err(message: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({"status": "error", "message": message}))
}

impl ToolBox {
    pub fn new() -> Self {
        ToolBox {
            tools: vec![
                Box::new(new_write_file_tool()),
                Box::new(new_replace_file_tool()),
                Box::new(new_read_file_tool()),
                Box::new(new_append_to_file_tool()),
                Box::new(new_create_folder_tool()),
                Box::new(new_get_folder_files_tool()),
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
