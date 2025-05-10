use tokio;
use termimad::MadSkin;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    /// Prompt
    #[arg(short, long)]
    prompt: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut codr = codr::Codr::new();

    // Markdown parser skin
    let skin = MadSkin::default();

    match codr.message(args.prompt).await {
        Ok(response) => {
            for line in response {
                skin.print_text(&line.unwrap());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
