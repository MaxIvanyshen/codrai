use std::io::{self, Write};

use tokio;
use termimad::MadSkin;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = "")]
    prompt: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut codr = codr::Codr::new();

    let skin = MadSkin::default();

    let mut prompt = String::new();

    if args.prompt.is_empty() {
        skin.print_text("Welcome to Codr! Type 'exit' to quit.");
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
        match codr.message(prompt.to_string()).await {
            Ok(response) => {
                for line in response {
                    skin.print_text(&line.unwrap());
                }
            }
            Err(e) => {
                eprintln!("Error while processing your input: {}", e);
            }
        }

        prompt.clear();
    }
}
