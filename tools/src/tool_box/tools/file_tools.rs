use std::{fs, io::Write};
use crate::tool_box::{tools::Tool, status_success};

pub fn new_write_file_tool() -> Tool {
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

pub fn new_replace_file_tool() -> Tool {
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

pub fn new_append_to_file_tool() -> Tool {
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

pub fn new_create_folder_tool() -> Tool {
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

pub fn new_get_folder_files_tool() -> Tool {
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
