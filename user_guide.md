# ECHONET Lite Project - User Guide

## Project Overview

A Rust-based **ECHONET Lite** smart home control system with two components:

- **Controller**: Interactive CLI to discover and control ECHONET Lite devices on your network
- **Monolight**: Simulated mono-functional lighting device for testing purposes

Both services communicate via ECHONET Lite protocol over UDP port 3610 and run in Docker containers.

---

## Quick Start

### Prerequisites

- Docker and Docker Compose installed
- Port 3610 (UDP) accessible on your network

### Run Both Services

Start monolight in background and controller in interactive mode:

```bash
docker compose down --remove-orphans
docker compose up -d monolight
docker compose run -it controller
```

Then in the controller:

```
search              # Discover devices
list                # Show discovered devices
write 2 029101 80 30    # Turn on light
write 2 029101 80 31    # Turn off light
exit                # Stop controller
```

### Run Only Controller

```bash
docker compose up controller
```

### Run Only Monolight

```bash
docker compose up -d monolight
```

---

## Controller Guide

The **Controller** is an interactive command-line tool that discovers and controls ECHONET Lite devices.

### Starting the Controller

```bash
# Interactive mode (recommended)
docker compose run -it controller

# Background mode
docker compose up controller

# Standalone (requires Rust)
cd controller && cargo run --release
```

### Available Commands

#### `search`

Discovers all ECHONET Lite devices on your network (takes ~3 seconds).

```bash
> search
Searching for devices (this may take a few seconds)...
Search complete.
```

**Run this first** before using `list` or other commands.

#### `list`

Displays all discovered devices with their objects and properties.

```bash
> list
Discovered ECHONET Lite Devices:
====================================================================================================
Device [1]: 192.168.65.3 (192.168.65.3:3610) - Experimental
  Object [0]: 0EF001 - Node profile
    Property: 80 - Operation status (Read-only: false)
    Property: 8A - Manufacturer code (Read-only: true)

Device [2]: 192.168.65.6 (192.168.65.6:3610) - Experimental
  Object [0]: 029101 - Mono functional lighting
    Property: 80 - Operation status (Read-only: false)
```

Shows device IP addresses, object codes, and property codes with access rights.

#### `read <device> <object> <property>`

Reads a property value from a device.

**Syntax:**

- `device`: Device index from `list` output (1-based)
- `object`: Object code in hex (6 digits)
- `property`: Property code in hex (2 digits)

**Examples:**

```bash
# Read light operating status
> read 2 029101 80
Reading property 80 from device 2 object 029101...
Property 80: 31
# 30 = On, 31 = Off

# Read manufacturer code
> read 2 0ef001 8a
Property 8A: 00000b
```

#### `write <device> <object> <property> <data>`

Writes a property value to control a device.

**Syntax:**

- `device`: Device index from `list` output (1-based)
- `object`: Object code in hex (6 digits)
- `property`: Property code in hex (2 digits)
- `data`: Property value in hex

**Examples:**

```bash
# Turn on light
> write 2 029101 80 30
Writing property 80=30 to device 2 object 029101...
Property write successful

# Turn off light
> write 2 029101 80 31
Property write successful
```

#### `help`

Shows all available commands.

```bash
> help
Available commands:
  search              - Search for ECHONET Lite devices on the network
  list                - List all discovered devices and their properties
  read <dev> <obj> <prop> - Read a property from a device
  write <dev> <obj> <prop> <data> - Write a property value
  help                - Show this help message
  exit                - Stop controller and exit
```

#### `exit` or `quit`

Stops the controller and exits.

### Property Reference

For this project, the main properties are:

| Property Code | Name              | Values           | Example Usage  |
| ------------- | ----------------- | ---------------- | -------------- |
| 80            | Operation status  | 30h=On, 31h=Off  | Control lights |
| 8A            | Manufacturer code | 3-byte hex value | Device info    |
| 0EF001        | Node profile      | System object    | Device details |

### Typical Workflow

```bash
1. Run 'search' to discover devices on the network
2. Run 'list' to see their codes and properties
3. Use 'read 2 029101 80' to check current light status
4. Use 'write 2 029101 80 30' to turn light on
5. Use 'write 2 029101 80 31' to turn light off
6. Use 'exit' to stop
```

### Control the Test Light

With monolight running in background:

```bash
# Check status (31 = off, 30 = on)
> read 2 029101 80

# Turn on
> write 2 029101 80 30

# Turn off
> write 2 029101 80 31

# Verify the change
> read 2 029101 80
```

---

## Monolight Guide

**Monolight** is a simulated ECHONET Lite lighting device for testing the controller.

### Starting Monolight

```bash
# Background mode (recommended)
docker compose up -d monolight

# Foreground mode
docker compose up monolight

# Standalone (requires Rust)
cd monolight && cargo run --release
```

### Device Information

- **Device Code**: 029101 (Mono functional lighting)
- **Port**: 3610 (UDP)
- **Status Property**: 0x80 (30h=On, 31h=Off)
- **Default State**: Off (31h)

### Testing with Monolight

Start both services:

```bash
docker compose up -d monolight
docker compose run -it controller
```

In the controller:

```bash
> search              # Should find monolight at 192.168.65.6
> list                # Should show Device [2]: 029101 - Mono functional lighting
> read 2 029101 80    # Should show 31 (off)
> write 2 029101 80 30 # Turn on
> read 2 029101 80    # Should show 30 (on)
> write 2 029101 80 31 # Turn off
> read 2 029101 80    # Should show 31 (off)
```

### Verify Monolight is Running

```bash
# Check if container is running
docker ps | grep monolight

# View logs
docker logs echonet_monolight

# View network
docker network inspect echonet
```

---

## Docker Setup

### Project Structure

```
├── docker-compose.yml          # Service definitions
├── controller.dockerfile       # Controller image
├── monolight.dockerfile        # Monolight image
├── controller/                 # Controller source code
│   ├── src/main.rs
│   ├── Cargo.toml
│   └── examples.sh
└── monolight/                  # Monolight source code
    ├── src/main.rs
    └── Cargo.toml
```

### Docker Compose Commands

**Build services:**

```bash
docker compose build                # Build all
docker compose build controller     # Build only controller
docker compose build monolight      # Build only monolight
```

**Run services:**

```bash
docker compose up                               # Start all in foreground
docker compose up -d                            # Start all in background
docker compose up -d monolight                  # Start monolight only
docker compose up controller                   # Start controller in foreground
docker compose run -it controller              # Start controller interactive
```

**Stop services:**

```bash
docker compose down                  # Stop all
docker compose stop                  # Stop without removing
docker compose kill                  # Force stop
```

**View logs:**

```bash
docker compose logs controller       # View controller logs
docker compose logs monolight        # View monolight logs
docker compose logs -f               # Follow logs in real-time
```

**Clean up:**

```bash
docker compose down --remove-orphans  # Stop and remove orphaned containers
docker system prune                   # Clean up unused images/containers
```

### Network

Both services connect via a `echonet` Docker bridge network for inter-container communication.

View network details:

```bash
docker network ls | grep echonet
docker network inspect echonet
```

---

## Troubleshooting

### Controller Issues

#### No devices found after search

**Problem**: `search` completes but `list` shows only controller, not monolight

**Solutions**:

1. Verify monolight container is running: `docker ps | grep monolight`
2. Check monolight logs: `docker logs echonet_monolight`
3. Run `search` again and wait the full 3 seconds
4. Verify both containers are on the same network: `docker network inspect echonet`
5. Restart both services: `docker compose down && docker compose up -d monolight && docker compose run -it controller`

#### Timeout on read/write operations

**Problem**: Commands timeout after 2 seconds

**Solutions**:

1. Verify monolight is running: `docker ps`
2. Check device index is correct (first discovered device is 1, not 0)
3. Verify object code is correct (6 hex digits: 029101)
4. Verify property code is correct (2 hex digits: 80)
5. Check monolight logs for errors: `docker logs echonet_monolight`

#### Invalid device index

**Problem**: Error "Invalid device index"

**Solutions**:

1. Run `list` to see available device indices
2. Remember device indexing starts at 1 (not 0)
3. Numbers change if devices go offline/back online
4. Run `search` again to refresh

#### Unknown command error

**Problem**: "Unknown command" when typing a command

**Solutions**:

1. Check command spelling matches exactly
2. Use lowercase for hex values (both work but lowercase preferred)
3. Type `help` to see all available commands
4. Ensure you typed `enter` after the command

### Docker Issues

#### Container fails to start

**Problem**: Container exits immediately after `docker compose up`

**Solutions**:

1. Check logs: `docker compose logs controller`
2. Try rebuilding: `docker compose build --no-cache`
3. Ensure port 3610 isn't in use: `netstat -an | grep 3610`
4. Try `docker compose down --remove-orphans` then restart

#### Build fails

**Problem**: `docker compose build` fails with download errors

**Solutions**:

1. Check internet connection (needs to download Rust dependencies)
2. Try again (network may be temporary)
3. Clear Docker cache: `docker system prune`
4. Check available disk space

#### Connection issues between services

**Problem**: Containers can't communicate with each other

**Solutions**:

1. Verify both are on same network: `docker network inspect echonet`
2. Check firewall isn't blocking port 3610
3. Verify both containers are in `docker ps` output
4. Check service logs for errors

---

## Quick Reference

### Essential Commands

```bash
# Start both services (interactive)
docker compose down --remove-orphans
docker compose up -d monolight
docker compose run -it controller

# Start just controller
docker compose up controller

# In controller terminal
search                              # Discover devices
list                                # Show all devices
read 2 029101 80                   # Read light status
write 2 029101 80 30               # Turn light on
write 2 029101 80 31               # Turn light off
help                                # Show commands
exit                                # Exit controller
```

### Device Codes

```
029101  -  Mono functional lighting (the test device)
0EF001  -  Node profile object (device info)
```

### Property Codes

```
80  -  Operating status (30=On, 31=Off)
8A  -  Manufacturer code (read-only)
D3  -  Number of self-node instances
```

### Hex Values

```
30 (hex)  -  On
31 (hex)  -  Off
```

---

## Support

For issues:

1. Check the logs: `docker compose logs -f`
2. Verify containers are running: `docker ps`
3. Test with a fresh build: `docker compose build --no-cache`
4. Review the implementation details in IMPLEMENTATION.md

For protocol details, see PROTOCOL_GUIDE.md in the controller directory.
