# ECHONET Lite Controller

A comprehensive, interactive Echonet Lite controller implementation in Rust using the `uecho-rs` library.

## Features

✨ **Complete Echonet Lite Control**

- 🔍 Device Discovery: Automatically discover Echonet Lite devices on your network
- 📋 Device Listing: View all discovered devices with detailed information
- 📖 Property Reading: Read properties from Echonet Lite devices
- ✍️ Property Writing: Write/control device properties
- 🏢 Manufacturer Information: Display device manufacturer codes and names
- 📊 Object Information: View all objects and properties in discovered devices
- ⚙️ Standard Database Integration: Uses Echonet standard database for human-readable names

## Building

The controller is designed to run in a Docker container with Linux. To build:

```bash
docker compose build controller
```

## Running

Start the controller using Docker Compose:

```bash
docker compose up controller
```

Or run it directly in Docker:

```bash
docker run -it --network host Echonet_controller
```

## Usage

Once started, the controller presents an interactive command-line interface:

```
> help
Available commands:
  search              - Search for Echonet Lite devices on the network
  list                - List all discovered devices and their properties
  read <dev> <obj> <prop> - Read a property from a device
                        dev: device index (1-based)
                        obj: object code in hex (e.g., 029101)
                        prop: property code in hex (e.g., 80)
  write <dev> <obj> <prop> <data> - Write a property value
                        dev: device index (1-based)
                        obj: object code in hex
                        prop: property code in hex
                        data: hex data (e.g., 30)
  help                - Show this help message
  exit                - Stop controller and exit
```

### Examples

#### 1. Search for Devices

```
> search
Searching for devices (this may take a few seconds)...
Search complete.
```

#### 2. List All Devices

```
> list
Discovered Echonet Lite Devices:
=====================================
Device [1]: 192.168.1.100 (192.168.1.100:3610) - Panasonic
  Object [0]: 0EF001 - Node Profile Object
    Property: 8A - Manufacturer Code (Read-only: False)
    Property: 8B - Model Code (Read-only: False)
  Object [1]: 029101 - Mono functional lighting
    Property: 80 - Operating status (Read-only: True)
    Property: B0 - Light level Setting (Read-only: False)
```

#### 3. Read a Property (e.g., Light Status)

```
> read 1 029101 80
Reading property 80 from device 1 object 029101...
Property 80: 30
```

Where `30` = On, `31` = Off

#### 4. Write a Property (e.g., Turn on a Light)

```
> write 1 029101 80 30
Writing property 80=30 to device 1 object 029101...
Property write successful
```

#### 5. Turn Off the Light

```
> write 1 029101 80 31
Writing property 80=31 to device 1 object 029101...
Property write successful
```

## Echonet Lite Basics

### Object Code Format

Object codes are 6-digit hexadecimal values:

- `0EF001` - Node Profile Object (always present)
- `029101` - Mono functional lighting
- `014200` - Smart meter (electricity meter)
- etc.

### Property Code Format

Property codes are 2-digit hexadecimal values:

- `80` - Operating status (most common)
- `8A` - Manufacturer code
- `8B` - Model code
- etc.

### Common Property Values

- Operating Status (`80`):
  - `30` = On/Operating
  - `31` = Off/Not operating
- Manufacturer Codes:
  - Use the `list` command to see manufacturer information

## Architecture

```
┌─────────────────────────────────────────┐
│    Echonet Lite Controller (Rust)       │
│                                         │
│  ┌──────────────────────────────────┐  │
│  │  Interactive CLI Interface       │  │
│  │  • search                        │  │
│  │  • list                          │  │
│  │  • read                          │  │
│  │  • write                         │  │
│  └──────────────────────────────────┘  │
│               ↓                         │
│  ┌──────────────────────────────────┐  │
│  │  uecho-rs Library                │  │
│  │  • Controller                    │  │
│  │  • Message Handling              │  │
│  │  • Standard Database             │  │
│  └──────────────────────────────────┘  │
│               ↓                         │
│  ┌──────────────────────────────────┐  │
│  │  Echonet Lite Protocol           │  │
│  │  • UDP Multicast Discovery       │  │
│  │  • Unicast Message Exchange      │  │
│  └──────────────────────────────────┘  │
│               ↓                         │
│  ┌──────────────────────────────────┐  │
│  │  Network (UDP Port 3610)         │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

## Dependencies

- `Echonet` - Echonet Lite protocol library
- `log` - Logging framework
- `env_logger` - Environment-based logger configuration
- `hex` - Hex encoding/decoding utilities

## Implementation Details

### Device Discovery Flow

1. Controller starts and binds to local network interfaces
2. `search` command sends Echonet SSDP discovery requests
3. Devices respond with their node profile information
4. Controller stores discovered remote nodes
5. `list` command queries each device for detailed information

### Property Read Flow

1. Parse command: `read 1 029101 80`
2. Create Echonet Message with ESV=ReadRequest
3. Send message to device 1, object 029101, property 80
4. Wait for response (timeout: 2 seconds)
5. Parse and display property value in hexadecimal

### Property Write Flow

1. Parse command: `write 1 029101 80 30`
2. Create Echonet Message with ESV=WriteRequest
3. Set property data to hex value (30)
4. Send message to device 1, object 029101
5. Wait for acknowledgment
6. Display result

## Error Handling

- **Invalid device index**: Checks against available discovered devices
- **Timeout**: 2-second timeout for property read/write operations
- **Invalid hex format**: Validates hex input for object codes and property values
- **Network errors**: Handles communication failures gracefully

## Logging

Set the log level via environment variable:

```bash
docker run -it --network host -e RUST_LOG=debug Echonet_controller
```

Supported levels: `trace`, `debug`, `info`, `warn`, `error`

## Troubleshooting

### No Devices Found

1. Ensure devices are powered on and connected to the network
2. Check that devices support Echonet Lite protocol
3. Verify network connectivity with controller
4. Try searching again with `search`

### Timeout on Read/Write

1. Device may not be responding
2. Check device is still on network with `list`
3. Try reading a different property
4. Check Echonet object and property codes are correct

### Connection Refused

1. Ensure Docker container is running with `--network host`
2. Check firewall allows UDP port 3610
3. Verify device is on same network segment

## Related Resources

- [uecho-rs GitHub](https://github.com/cybergarage/uecho-rs)
- [Echonet Consortium](https://Echonet.jp/english/)
- [Echonet Specification](https://Echonet.jp/spec_mra_rp1_en/)

## License

This project uses the `echonet` library which is licensed under Apache 2.0.
