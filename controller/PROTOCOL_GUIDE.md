# ECHONET Lite Protocol Guide

## Overview

ECHONET (Energy Conservation and Homecare Network) Lite is a lightweight open communication protocol standard for home energy management systems and IoT devices in Japan. The controller implements full support for discovering devices and sending/receiving ECHONET Lite messages.

## Architecture

### Communication Model

```
┌─────────────┐                      ┌─────────────┐
│ Controller  │◄──────────────────►  │  Device 1   │
│   (Node A)  │      UDP/Multicast   │  (Node B)   │
└─────────────┘                      └─────────────┘
        │                                    │
        │ Port 3610                          │ Port 3610
        │                                    │
        └────────────── Local Network ──────┘
```

### Message Flow

1. **Device Discovery (Multicast)**
   - Controller sends multicast discovery message to 224.0.23.214:3610
   - All ECHONET devices respond with their node information

2. **Property Reading (Unicast)**
   - Controller sends ReadRequest to device
   - Device responds with property value

3. **Property Writing (Unicast)**
   - Controller sends WriteRequest with data
   - Device acknowledges and applies change

## Protocol Details

### ECHONET Message Structure

```
┌──────────┬─────────┬──────┬────────────┬─────────┬──────┬─────────────┐
│ EHD1/2   │ TID     │ SEOJ │ DEOJ       │ ESV     │ OPC   │ Properties  │
│ (2 bytes)│(2 bytes)│(3b)  │ (3 bytes)  │(1 byte) │(1b)  │ (variable)  │
└──────────┴─────────┴──────┴────────────┴─────────┴──────┴─────────────┘
```

**Field Descriptions:**

- **EHD1/2**: Header (0x10 0x81 for ECHONET Lite)
- **TID**: Transaction ID (auto-managed by library)
- **SEOJ**: Source ECHONET Object (controller's node profile)
- **DEOJ**: Destination ECHONET Object (device target)
- **ESV**: ECHONET Service code (request type)
- **OPC**: Number of properties
- **Properties**: Property code + data array

### ECHONET Service Codes (ESV)

The ESV field determines the type of operation:

| Code | Name              | Direction         | Description                   |
| ---- | ----------------- | ----------------- | ----------------------------- |
| 0x62 | ReadRequest       | Controller→Device | Request property value        |
| 0x72 | ReadResponse      | Device→Controller | Property value response       |
| 0x61 | WriteRequest      | Controller→Device | Change property value         |
| 0x71 | WriteResponse     | Device→Controller | Write confirmation            |
| 0x6E | WriteReadRequest  | Controller→Device | Write then read               |
| 0x7E | WriteReadResponse | Device→Controller | Write/read confirmation       |
| 0x73 | SetEvent          | Device→Controller | Property changed notification |

### Object Code Structure

Object codes are 6-digit hexadecimal values representing:

```
AABBCC
││└── Instance Code (device number, usually 01)
│└─── Class Code (device type sub-category)
└──── Class Group Code (device category)
```

**Examples:**

- `0EF001` - Node Profile Object (group 0E, class F0, instance 01)
- `029101` - Mono-functional lighting (group 02, class 91, instance 01)
- `013001` - Air conditioner (group 01, class 30, instance 01)
- `028801` - Smart meter (group 02, class 88, instance 01)

### Property Code Structure

Property codes are 2-digit hexadecimal values:

```
PP
└── Property Number (0x00-0xFF)
```

**Standard Property Codes (applicable to most devices):**

| Code | Name                               | Type    | Read | Write |
| ---- | ---------------------------------- | ------- | ---- | ----- |
| 80   | Operating Status                   | Enum    | ✓    | ✓     |
| 81   | Operation Mode                     | Enum    | ✓    | ✓     |
| 82   | Fault Status                       | Enum    | ✓    | ✗     |
| 8A   | Manufacturer Code                  | Integer | ✓    | ✗     |
| 8B   | Model Code                         | String  | ✓    | ✗     |
| 8C   | Serial Number                      | String  | ✓    | ✗     |
| 9D   | Status Change Announcement         | Bool    | ✓    | ✗     |
| 9E   | Set Property Value After Some Time | Integer | ✗    | ✓     |
| 9F   | Get Property Value After Some Time | Integer | ✗    | ✓     |

## Device Categories

### Lighting (Group 0x02)

| Code     | Device                    | Properties |
| -------- | ------------------------- | ---------- |
| 02 91 01 | Mono functional lighting  | 80, B0, B1 |
| 02 93 01 | Color functional lighting | 80, B0-B4  |
| 02 92 01 | Dimmer switch             | 80, B0     |

**Properties:**

- `80`: Operating status (30=On, 31=Off)
- `B0`: Light level (0x00-0xFF = 0-100%)
- `B1`: Brightness (0x00-0xFF)
- `B2`: Color (RGB data)

### Climate Control (Group 0x01)

| Code     | Device             | Properties |
| -------- | ------------------ | ---------- |
| 01 30 01 | Air conditioner    | 80, B3, A4 |
| 01 67 01 | Air cleaner        | 80, C0     |
| 01 45 01 | Temperature sensor | 7E         |
| 01 42 01 | Humidity sensor    | 7D         |

**Properties:**

- `80`: Operating status
- `B3`: Temperature setting (0-50°C)
- `A4`: Operating mode (0=Auto, 1=Cooling, 2=Heating, etc.)
- `7E`: Temperature value (0-250 = 0-25°C)

### Smart Metering (Group 0x02)

| Code     | Device                    | Properties |
| -------- | ------------------------- | ---------- |
| 02 88 01 | Smart meter (Electricity) | E0, E7     |
| 02 89 01 | Smart meter (Gas)         | E0, E7     |
| 02 8A 01 | Smart meter (Water)       | E0, E7     |

**Properties:**

- `E0`: Cumulative consumption
- `E7`: Instantaneous consumption/power
- `EA`: Measurement date/time

## Common Device Scenarios

### Scenario 1: Control a Light

1. **Find the device**

   ```bash
   > search
   > list
   # Identify device index and object code (e.g., Device 1, Object 029101)
   ```

2. **Turn on**

   ```bash
   > write 1 029101 80 30
   # 30 = On status
   ```

3. **Set brightness to 50%**
   ```bash
   > write 1 029101 b0 80
   # 80 in hex = 128 in decimal = 50% brightness
   ```

### Scenario 2: Control Air Conditioner

1. **Turn on to cooling mode**

   ```bash
   > write 2 013001 80 30     # Turn on
   > write 2 013001 a4 01     # Set to cooling mode
   ```

2. **Set temperature to 22°C**

   ```bash
   > write 2 013001 b3 16
   # 22 decimal = 0x16 hex
   ```

3. **Read current settings**
   ```bash
   > read 2 013001 b3         # Current temperature
   > read 2 013001 a4         # Current mode
   ```

### Scenario 3: Monitor Power Consumption

1. **Read instantaneous power**

   ```bash
   > read 3 028801 e7
   # Returns 4-byte value in watts
   ```

2. **Read monthly/yearly consumption**
   ```bash
   > read 3 028801 e0
   # Returns 4-byte value in kWh
   ```

## Data Types

### Enum (Enumeration)

Fixed set of values (e.g., On/Off)

- `30` = On
- `31` = Off

### Integer

Decimal or byte values

- Single byte: 0x00-0xFF
- 2-byte: 0x0000-0xFFFF
- 4-byte: 0x00000000-0xFFFFFFFF

### String

ASCII or Unicode text

- Device name, model code, etc.

### Binary

Raw binary data

- Timestamps, configuration data

## Standard Manufacturer Codes

```
00 00 0B - Panasonic
00 00 09 - Mitsubishi Electric
00 00 03 - Daikin Industries
00 00 13 - Fujitsu
00 0C E5 - Haier
00 01 04 - Philips
```

Complete list at: https://echonet.jp/spec-en/

## Error Handling

### Common Issues

**No Response (Timeout)**

- Device may be offline
- Check with `list` command
- Verify network connectivity

**Invalid Object Code**

- Object may not exist on device
- Check device properties with `list`
- Object codes are device-specific

**Write Failed (Invalid Property)**

- Property may be read-only
- Check with `list` for access rights
- Verify property code spelling

**Invalid Property Value**

- Data may out of range
- Check property value format
- Verify hex encoding

## Implementation Notes

### Threading & Concurrency

- All ECHONET messages are asynchronous
- Use timeouts for request handling (default: 2 seconds)
- Multiple devices can be queried simultaneously

### Network Requirements

- UDP port 3610 must be open
- Multicast support required (224.0.23.214:3610)
- Devices must be on same network segment

### Performance

- Device queries typically complete in <100ms
- Discovery takes 2-3 seconds (waits for device responses)
- Multiple properties can be read in sequence

## Advanced Usage

### Batch API Calls

```bash
# Read multiple properties
read 1 029101 80    # Status
read 1 029101 b0    # Brightness
read 1 029101 8a    # Manufacturer

# Write multiple properties
write 1 029101 80 30    # Turn on
write 1 029101 b0 ff    # Max brightness
```

### Device Query Pattern

```bash
# Get device info
read 1 0ef001 8a    # Manufacturer code
read 1 0ef001 8b    # Model code

# Get object info
read 1 029101 80    # Operating status
read 1 029101 9d    # Notification enabled

# Control device
write 1 029101 80 30 # Turn on
```

### Monitoring Changes

Use status change notifications (ESV 0x73) to detect when:

- Devices change state
- Properties are modified
- Faults occur on devices

## References

- ECHONET Consortium: https://echonet.jp/english/
- Machine Readable Appendix: https://echonet.jp/spec_mra_rp1_en/
- Manufacturer Code List: https://echonet.jp/spec-en/
- uecho-rs Library: https://github.com/cybergarage/uecho-rs

## Testing Your Setup

Use these commands to verify your ECHONET setup:

```bash
# 1. Discover devices
> search
> list

# 2. Verify device responds
> read 1 0ef001 8a

# 3. Test read operation
> read 1 029101 80

# 4. Test write operation (if device supports it)
> write 1 029101 80 30

# 5. Verify change
> read 1 029101 80
```

All tests should complete within 2 seconds with proper response messages.
