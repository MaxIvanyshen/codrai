use std::{fs, io::Write};
use openai::Tool as OpenAITool;

pub struct ToolBox {
    tools: Vec<Box<Tool>>,
}

fn status_success() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    Ok(serde_json::json!({"status": "success"}))
}

fn err(message: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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

fn new_write_file_tool() -> Tool {
    Tool {
        name: "write_file".to_string(),
        description: "Writes content to a file. If trying to write a file and the folder does not exist, use create_folder tool to create a folder first".to_string(),
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
            let file_path = args["file_path"].as_str().ok_or("file_path is required")?;
            let content = args["content"].as_str().ok_or("content is required")?;

            fs::write(file_path, content)?;
            status_success()
        },
    }
}

fn new_replace_file_tool() -> Tool {
    Tool {
        name: "replace_file_content".to_string(),
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
            status_success()
        },
    }
}

fn new_read_file_tool() -> Tool {
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

fn new_append_to_file_tool() -> Tool {
    Tool {
        name: "append_to_file".to_string(),
        description: "Appends content to a file".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Path to the file to append to"
                },
                "content": {
                    "type": "string",
                    "description": "Content to append to the file"
                }
            },
            "required": ["file_path", "content"]
        }),
        runner: |args| {
            let file_path = args["file_path"].as_str().ok_or("file_path is required")?;
            let content = args["content"].as_str().ok_or("content is required")?;

            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(file_path)?;
                
            file.write(content.as_bytes())?;
            status_success()
        },
    }
}

fn new_create_folder_tool() -> Tool {
    Tool {
        name: "create_folder".to_string(),
        description: "Creates a new folder".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "folder_path": {
                    "type": "string",
                    "description": "Path to the folder to create"
                }
            },
            "required": ["folder_path"]
        }),
        runner: |args| {
            let folder_path = args["folder_path"].as_str().ok_or("folder_path is required")?;
            fs::create_dir_all(folder_path)?;
            status_success()
        },
    }
}

fn new_get_folder_files_tool() -> Tool {
    Tool {
        name: "get_folder_files".to_string(),
        description: "Gets a list of files and folders in a directory, including nested contents".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "folder_path": {
                    "type": "string",
                    "description": "Path to the folder to list files and subfolders from"
                },
                "recursive": {
                    "type": "boolean",
                    "description": "Whether to recursively list files in subfolders (default: true)",
                    "default": true
                }
            },
            "required": ["folder_path"]
        }),
        runner: |args| {
            let folder_path = args["folder_path"].as_str().ok_or("folder_path is required")?;
            let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(true);
            
            fn scan_directory(path: &str, recursive: bool) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
                let mut files = Vec::new();
                let mut folders = Vec::new();
                
                for entry in fs::read_dir(path)? {
                    if let Ok(entry) = entry {
                        let path_buf = entry.path();
                        let file_name = path_buf.file_name()
                            .and_then(|n| n.to_str())
                            .map(String::from)
                            .ok_or("Invalid filename")?;
                        
                        let file_type = entry.file_type()?;
                        
                        if file_type.is_dir() {
                            if recursive {
                                let subfolder_path = path_buf.to_str()
                                    .ok_or("Invalid path")?;
                                let subfolder_contents = scan_directory(subfolder_path, recursive)?;
                                folders.push(serde_json::json!({
                                    "name": file_name,
                                    "path": subfolder_path,
                                    "contents": subfolder_contents
                                }));
                            } else {
                                folders.push(serde_json::json!({
                                    "name": file_name,
                                    "path": path_buf.to_str().ok_or("Invalid path")?
                                }));
                            }
                        } else if file_type.is_file() {
                            files.push(serde_json::json!({
                                "name": file_name,
                                "path": path_buf.to_str().ok_or("Invalid path")?
                            }));
                        }
                    }
                }
                
                Ok(serde_json::json!({
                    "files": files,
                    "folders": folders
                }))
            }
            
            let result = scan_directory(folder_path, recursive)?;
            Ok(result)
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
        let result = new_replace_file_tool().run(edit_args).unwrap();
        assert_eq!(result["status"], "success");

        let read_result = new_read_file_tool().run(serde_json::json!({
            "file_path": "./test.txt"
        })).unwrap();
        assert_eq!(read_result["content"], "Hello, Rust!");

        fs::remove_file("./test.txt").unwrap();
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

        fs::remove_file("./toolbox.txt").unwrap();
    }

    #[test]
    fn test_create_folder_tool() {
        let args = serde_json::json!({
            "folder_path": "./test_folder"
        });

        let result = new_create_folder_tool().run(args).unwrap();
        assert_eq!(result["status"], "success");

        assert!(fs::metadata("./test_folder").is_ok());

        fs::remove_dir_all("./test_folder").unwrap();
    }

    #[test]
    fn test_get_folder_files_tool() {
        let base_folder = "./test_folder_structure";
        let subfolder = format!("{}/subfolder", base_folder);
        
        // Clean up any existing test folders first (in case previous test failed)
        let _ = fs::remove_dir_all(base_folder);
        
        fs::create_dir_all(&base_folder).unwrap();
        fs::create_dir_all(&subfolder).unwrap();
        
        // Create test files
        fs::write(format!("{}/root_file.txt", base_folder), "Root file content").unwrap();
        fs::write(format!("{}/subfolder/nested_file.txt", base_folder), "Nested file content").unwrap();
        
        // Test recursive listing (default)
        let args = serde_json::json!({
            "folder_path": base_folder
        });
        
        let result = new_get_folder_files_tool().run(args).unwrap();
        
        // Verify structure
        assert!(result.is_object());
        assert!(result.get("files").is_some());
        assert!(result.get("folders").is_some());
        
        // Verify root files
        let files = result["files"].as_array().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["name"], "root_file.txt");
        
        // Verify folders
        let folders = result["folders"].as_array().unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0]["name"], "subfolder");
        
        // Verify subfolder contents
        let subfolder_contents = &folders[0]["contents"];
        assert!(subfolder_contents.is_object());
        let subfolder_files = subfolder_contents["files"].as_array().unwrap();
        assert_eq!(subfolder_files.len(), 1);
        assert_eq!(subfolder_files[0]["name"], "nested_file.txt");
        
        // Test non-recursive listing
        let args_non_recursive = serde_json::json!({
            "folder_path": base_folder,
            "recursive": false
        });
        
        let result_non_recursive = new_get_folder_files_tool().run(args_non_recursive).unwrap();
        
        // Verify structure
        let folders_non_recursive = result_non_recursive["folders"].as_array().unwrap();
        assert_eq!(folders_non_recursive.len(), 1);
        assert_eq!(folders_non_recursive[0]["name"], "subfolder");
        
        // Verify that non-recursive doesn't include contents
        assert!(folders_non_recursive[0].get("contents").is_none());
        
        fs::remove_dir_all(base_folder).unwrap();
    }
}
