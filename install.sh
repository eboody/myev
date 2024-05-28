#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Build your Rust project in release mode.
echo "Building project..."
cargo build --release

# Replace "my_project_binary" with the name of your binary.
BINARY_NAME="myev"
SERVICE_FILE="myev.service"

# Move the binary to /usr/local/bin.
# You might need to change this path based on your requirements or system conventions.
echo "Moving binary to /usr/local/bin..."
sudo rm "/usr/local/bin/$BINARY_NAME"
sudo mv "./target/release/$BINARY_NAME" "/usr/local/bin/$BINARY_NAME"

# Copy the systemd service file to /etc/systemd/system.
# Ensure the service file name is correct and located in your project directory.
echo "Installing systemd service file..."
sudo cp -f "./$SERVICE_FILE" "/etc/systemd/system/$SERVICE_FILE"

# Reload systemd to recognize the new service.
sudo systemctl daemon-reload

# Optionally, enable and start the service.
# sudo systemctl enable "$SERVICE_FILE"
# sudo systemctl start "$SERVICE_FILE"

echo "Deployment complete."
