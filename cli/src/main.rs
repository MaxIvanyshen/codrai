use std::io::{self, Write};
use tokio;
use termimad::MadSkin;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "")]
    prompt: String,

    #[arg(short, long, default_value = "false")]
    stream: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut codr = codr::Codr::new();

    // Create a custom skin for regular text
    let text_skin = MadSkin::default();
    
    // Create an enhanced skin for code blocks with syntax highlighting
    let mut code_skin = MadSkin::default();
    // Customize code blocks with vibrant colors
    code_skin.code_block.set_bg(termimad::rgb(40, 44, 52));
    code_skin.code_block.set_fg(termimad::rgb(171, 178, 191));
    
    let mut prompt = String::new();

    if args.prompt.is_empty() {
        text_skin.print_text("Welcome to Codr! Type 'exit' to quit.");
    } else {
        prompt = args.prompt.clone();
    }

    loop {
        if prompt.is_empty() {
            print!("Ask Codr (type 'exit' to quit): ");
            io::stdout().flush().unwrap();

            std::io::stdin().read_line(&mut prompt).unwrap();
            prompt = prompt.trim().to_string();
            if prompt == "exit" {
                break;
            }
        }
        
        if args.stream {
            let mut receiver = codr.message_stream(prompt.clone()).await;
            
            // Track code block state
            let mut in_code_block = false;
            let mut code_block_content = String::new();
            
            while let Some(chunk) = receiver.recv().await {
                if chunk.is_empty() {
                    continue;
                }
                
                if chunk.contains("```") {
                    // Handle chunks that contain code block markers
                    let parts: Vec<&str> = chunk.split("```").collect();
                    
                    for (i, part) in parts.iter().enumerate() {
                        if i % 2 == 0 {
                            // Outside code block
                            if !in_code_block {
                                // Regular text before code block
                                if !part.is_empty() {
                                    text_skin.print_inline(part);
                                }
                            } else {
                                // End of code block
                                code_block_content.push_str(part);
                                
                                // Print the complete code block
                                code_skin.print_text(&format!("```{}", code_block_content));
                                print!("```");
                                
                                // Reset code block tracking
                                code_block_content.clear();
                            }
                        } else {
                            // Code block marker or content
                            if !in_code_block {
                                // Starting a code block
                                in_code_block = true;
                                code_block_content = part.to_string();
                            } else {
                                // Ending a code block
                                in_code_block = false;
                                
                                // Print any text after the code block
                                if !part.is_empty() {
                                    text_skin.print_inline(part);
                                }
                            }
                        }
                    }
                } else if in_code_block {
                    // Inside a code block - accumulate content
                    code_block_content.push_str(&chunk);
                } else {
                    // Regular text - print immediately
                    text_skin.print_inline(&chunk);
                }
                
                // Ensure output is displayed immediately
                io::stdout().flush().unwrap();
            }
            
            // Handle any remaining content
            if in_code_block && !code_block_content.is_empty() {
                // Print any accumulated code block
                code_skin.print_text(&format!("```{}", code_block_content));
                print!("```");
            }
        } else {
            // Non-streaming mode
            match codr.message(prompt.to_string()).await {
                Ok(response) => {
                    let mut full_response = String::new();
                    
                    for line in response {
                        if let Some(content) = line {
                            full_response.push_str(&content);
                        }
                    }
                    
                    // Process the markdown to identify code blocks
                    let segments = full_response.split("```").collect::<Vec<&str>>();
                    
                    for (i, segment) in segments.iter().enumerate() {
                        if i % 2 == 0 {
                            // Regular text
                            text_skin.print_text(segment);
                        } else {
                            // Code block
                            code_skin.print_text(&format!("```{}", segment));
                            print!("```");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error while processing your input: {}", e);
                }
            }
        }

        text_skin.print_text("\n");
        prompt.clear();
    }
}

