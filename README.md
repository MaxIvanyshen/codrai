# Rust-Based AI Coding Agent

An advanced AI-powered coding assistant developed in Rust, designed to enhance software development workflows through intelligent code generation, analysis, and automation.

## Overview

Codr AI is a robust Rust project that leverages artificial intelligence to assist developers in writing, reviewing, and managing code more efficiently. It integrates seamlessly with your development environment, providing intelligent suggestions, auto-completions, and automation utilities powered by state-of-the-art AI models.

## Key Features

- **AI Integration:** Utilizes OpenAI APIs to enable natural language understanding and code generation capabilities, allowing users to instruct the AI with plain language.
- **User-Friendly CLI:** Offers a straightforward command-line interface for easy interaction with various AI functions, file management, and utility tools.
- **Modular and Extensible Architecture:** Designed with clear module separation including core logic, AI communication, utilities, and CLI components, facilitating future enhancements.
- **Asynchronous Performance:** Built with async Rust crates such as `tokio` and `reqwest` to ensure efficient and non-blocking operations.
- **Customization and Utilities:** Includes a suite of tools for automation, file handling, code management, and development support.

## Technology Stack

- **Programming Language:** Rust, with modern syntax and features.
- **Crates and Libraries:** `serde` for serialization, `reqwest` for HTTP requests, `tokio` for async runtime, `hyper` for HTTP server capabilities.
- **Version Control:** Managed with Git for collaborative development.
- **AI Integration:** Connects with OpenAI API for advanced language and code processing.

## Use Cases

- Automate code writing, refactoring, and review processes.
- Perform natural language driven programming commands.
- Automate repetitive developer tasks.
- Educational aid for learning programming concepts.
- Enhance existing IDEs or code editors with AI features.

## Getting Started

To get started, clone the repository, install dependencies, and configure your environment variables:

- **CODR_BASE_URL:** The base URL for your OpenAI-compatible API.
- **CODR_API_KEY:** Your API key for authenticating requests.
- **CODR_MODEL:** The name of the language model supporting tool calls.

Set these environment variables in your shell, for example:

```bash
export CODR_BASE_URL="https://api.openai.com/v1"
export CODR_MODEL="gpt-4"
export CODR_API_KEY="your-api-key-here"
```

Ensure these are correctly configured to allow the program to access the AI services.

## Contributing

Contributions are welcome! Feel free to fork the repo, submit issues, or propose enhancements.

---

This project aims to embed AI deeply into your development setup, making coding faster, smarter, and more enjoyable. For updates and support, refer to the repository documentation.

Happy coding!
