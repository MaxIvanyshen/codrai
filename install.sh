#!/bin/bash

# Build the Rust project
cargo build --release

# Create the codrai folder in a standard location for executables
CODRAI_DIR="/usr/local/bin/codrai"
sudo mkdir -p "$CODRAI_DIR"

# Move the compiled binary to the codrai directory
sudo mv ./target/release/cli "$CODRAI_DIR/codr"

# Copy the system_prompt.md to codrai directory
sudo cp ./system_prompt.md "$CODRAI_DIR"

# Ensure the codrai directory is in PATH
SHELL_CONFIG="$HOME/.zshrc"
if ! grep -q "$CODRAI_DIR" "$SHELL_CONFIG"; then
  echo "export PATH=\"$CODRAI_DIR:\$PATH\"" >> "$SHELL_CONFIG"
  echo "Added $CODRAI_DIR to PATH in $SHELL_CONFIG"
  echo "Please run 'source $SHELL_CONFIG' or start a new terminal to update your PATH"
fi

# Make sure the binary is executable
sudo chmod +x "$CODRAI_DIR/codr"

# Create a symlink in /usr/local/bin (which is typically in PATH)
sudo ln -sf "$CODRAI_DIR/codr" /usr/local/bin/codr

echo "Installation complete. You can now use 'codr' from anywhere."
echo "If it doesn't work immediately, please run: source $SHELL_CONFIG"

