#!/bin/bash
# Run this script on the Raspberry Pi with sudo!

if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (use sudo)"
  exit 1
fi

echo "[*] Setting up Echonet Lite Controller service..."

# 1. Build the Rust binary (assuming rustc and cargo are installed on the Pi)
echo "[*] Building the controller..."
cargo build --release

# 2. Create the target directory and copy files
echo "[*] Copying binary and targets to /opt/echonet-controller/..."
mkdir -p /opt/echonet-controller
chown cyres:cyres /opt/echonet-controller

# Copy the compiled executable and the targets.json configuration
cp target/release/controller /opt/echonet-controller/
cp targets.json /opt/echonet-controller/

# Ensure proper permissions
chown cyres:cyres /opt/echonet-controller/controller
chown cyres:cyres /opt/echonet-controller/targets.json
chmod +x /opt/echonet-controller/controller

# 3. Install the systemd service file
echo "[*] Installing the systemd service..."
cp echonet-controller.service /etc/systemd/system/
chmod 644 /etc/systemd/system/echonet-controller.service

# 4. Reload systemd, enable the service to start on boot, and start it now
echo "[*] Reloading systemd and enabling the service..."
systemctl daemon-reload
systemctl enable echonet-controller.service
systemctl start echonet-controller.service

echo "[✓] Service installed, enabled, and started!"
echo "    -> Check its status with: systemctl status echonet-controller"
echo "    -> View the live logs with: journalctl -fu echonet-controller"
