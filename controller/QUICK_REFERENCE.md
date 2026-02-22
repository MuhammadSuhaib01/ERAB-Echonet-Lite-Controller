# ECHONET Lite Controller - Quick Reference Guide

## Command Syntax

### Search Command

```
search
```

Searches the network for ECHONET Lite devices. Takes 2-3 seconds to complete.

### List Command

```
list
```

Displays all discovered devices with:

- IP address and port
- Manufacturer name and code
- All objects and their properties
- Property access rights (read-only indicator)

### Read Property

```
read <device_index> <object_code> <property_code>
```

**Parameters:**

- `device_index`: Number from the list command (1-based)
- `object_code`: 6-digit hex (e.g., `029101`)
- `property_code`: 2-digit hex (e.g., `80`)

**Example:**

```
read 1 029101 80
# Reads property 80 (operating status) from device 1, object 029101
```

### Write Property

```
write <device_index> <object_code> <property_code> <data>
```

**Parameters:**

- `device_index`: Number from the list command (1-based)
- `object_code`: 6-digit hex (e.g., `029101`)
- `property_code`: 2-digit hex (e.g., `80`)
- `data`: Hex data (e.g., `30` for on, `31` for off)

**Example:**

```
write 1 029101 80 30
# Turns on device 1 (sets operating status to 30=On)
```

## Common Device Object Codes

### Lighting

| Code   | Device Name               |
| ------ | ------------------------- |
| 029101 | Mono functional lighting  |
| 029301 | Color functional lighting |
| 029201 | Dimmer switch             |

### Climate Control

| Code   | Device Name        |
| ------ | ------------------ |
| 013001 | Air conditioner    |
| 016701 | Air cleaner        |
| 014201 | Humidity sensor    |
| 014501 | Temperature sensor |

### Kitchen Appliances

| Code   | Device Name       |
| ------ | ----------------- |
| 022701 | Cooking appliance |
| 017301 | Refrigerator      |
| 018E01 | Dishwasher        |

### Smart Metering

| Code   | Device Name               |
| ------ | ------------------------- |
| 028801 | Smart meter (electricity) |
| 028901 | Smart meter (gas)         |
| 028A01 | Smart meter (water)       |

### General/Profile

| Code   | Device Name         |
| ------ | ------------------- |
| 0EF001 | Node Profile Object |

## Common Property Codes

### Universal Properties

| Code | Name                       | Value           |
| ---- | -------------------------- | --------------- |
| 80   | Operating status           | 30h=On, 31h=Off |
| 8A   | Manufacturer code          | 3 bytes         |
| 8B   | Model code                 | Varies          |
| D3   | Status change announcement | True/False      |
| D4   | Set temporary location     | Address data    |
| D5   | Get temporary location     | Address data    |

### Lighting-Specific Properties

| Code | Name                     | Range           |
| ---- | ------------------------ | --------------- |
| 80   | Operating status         | 30h=On, 31h=Off |
| B0   | Light level setting      | 0x00-0xFF       |
| B1   | Brightness setting       | 0x00-0xFF       |
| B2   | Color adjustment setting | RGB data        |

### Air Conditioner Properties

| Code | Name                | Units           |
| ---- | ------------------- | --------------- |
| 80   | Operating status    | 30h=On, 31h=Off |
| B3   | Temperature setting | 0-50°C          |
| A4   | Operating mode      | 00-06           |

### Smart Meter (Electricity) Properties

| Code | Name                         | Unit |
| ---- | ---------------------------- | ---- |
| E0   | Cumulative power consumption | kWh  |
| E7   | Instantaneous power          | W    |

## Step-by-Step Examples

### Example 1: Turn On/Off a Light

1. **Search and find devices:**

   ```
   > search
   > list
   ```

2. **Identify the light (e.g., Device 1, Object 029101)**

3. **Turn on:**

   ```
   > write 1 029101 80 30
   ```

4. **Turn off:**
   ```
   > write 1 029101 80 31
   ```

### Example 2: Adjust Light Brightness

1. **Find the light and device index**

2. **Read current brightness:**

   ```
   > read 1 029101 b0
   # Returns hex value like "80" (represents 128/255 = 50% brightness)
   ```

3. **Set brightness to 75% (0xC0 = 192):**

   ```
   > write 1 029101 b0 c0
   ```

4. **Set brightness to 25% (0x40 = 64):**
   ```
   > write 1 029101 b0 40
   ```

### Example 3: Set Air Conditioner Temperature

1. **Find the air conditioner device (usually object 013001)**

2. **Set temperature to 22°C:**

   ```
   > write 2 013001 b3 16
   # 22 in decimal = 0x16 in hex
   ```

3. **Read current temperature setting:**
   ```
   > read 2 013001 b3
   ```

### Example 4: Read Power Consumption from Smart Meter

1. **Find smart meter (usually object 028801)**

2. **Read cumulative consumption (E0):**

   ```
   > read 3 028801 e0
   # Returns 4-byte value representing kWh
   ```

3. **Read instantaneous power (E7):**
   ```
   > read 3 028801 e7
   # Returns 4-byte value representing watts
   ```

## Hex to Decimal Conversion

Useful for understanding property values:

| Hex | Decimal | Use Case         |
| --- | ------- | ---------------- |
| 30  | 48      | On status        |
| 31  | 49      | Off status       |
| 40  | 64      | 25% brightness   |
| 80  | 128     | 50% brightness   |
| C0  | 192     | 75% brightness   |
| FF  | 255     | 100% brightness  |
| 16  | 22      | 22°C temperature |

## Troubleshooting Commands

### Check if devices are still available:

```
> list
```

### Re-discover devices:

```
> search
> list
```

### Test reading a known property:

```
> read 1 0ef001 8a
# Should return manufacturer code for device 1
```

### Verify manufacturer code:

```
> read 1 0ef001 8a
# Response format: 3-byte manufacturer code in hex
# e.g., "00000B" = Panasonic
```

## Common Manufacturer Codes

| Code   | Manufacturer     |
| ------ | ---------------- |
| 00000B | Panasonic        |
| 000009 | MID (Mitsubishi) |
| 000003 | Daikin           |
| 000013 | Fujitsu          |
| 000CE5 | Haier            |

## Tips & Tricks

✅ **Always search first** - Run `search` before `list` to discover devices

✅ **Use lowercase hex** - Both `FF` and `ff` work, but lowercase is cleaner

✅ **Device index starts at 1** - First device is index 1, not 0

✅ **Check device is on** - Use `list` to verify device still responds

✅ **Timeout is 2 seconds** - If no response, device may be offline

✅ **Write vs Read** - Not all properties are writable, use `list` for access rights

✅ **Batch operations** - Run commands in sequence for multiple device changes

## Getting Help

To see all available commands:

```
> help
```

## Exit the Controller

```
> exit
> quit  # Also accepted
```
