#!/usr/bin/env python3
"""
ECHONET Lite Controller Python Integration Examples

This script demonstrates how to interact with the ECHONET Lite controller
programmatically using Python.
"""

import subprocess
import json
import time
import sys
from typing import Optional, Dict, List, Tuple
from dataclasses import dataclass

@dataclass
class EchonetDevice:
    """Represents an ECHONET Lite device"""
    index: int
    ip_address: str
    port: int
    manufacturer: str
    objects: List[Dict]

class EchonetController:
    """Python wrapper for ECHONET Lite Controller"""
    
    def __init__(self, container_name: str = "echonet_controller"):
        """
        Initialize the controller wrapper
        
        Args:
            container_name: Docker container name for the controller
        """
        self.container_name = container_name
    
    def _run_command(self, *commands: str) -> str:
        """
        Run a command in the controller container
        
        Args:
            commands: Commands to execute
            
        Returns:
            Command output
        """
        cmd_input = "\n".join(commands) + "\nexit\n"
        try:
            result = subprocess.run(
                ["docker", "run", "-it", "--network", "host", self.container_name],
                input=cmd_input,
                text=True,
                capture_output=True,
                timeout=10
            )
            return result.stdout
        except subprocess.TimeoutExpired:
            print("Error: Controller command timed out", file=sys.stderr)
            return ""
        except Exception as e:
            print(f"Error running command: {e}", file=sys.stderr)
            return ""
    
    def discover(self) -> bool:
        """
        Discover ECHONET Lite devices on the network
        
        Returns:
            True if devices were found
        """
        print("Searching for ECHONET Lite devices...")
        output = self._run_command("search")
        time.sleep(1)
        return "Search complete" in output
    
    def list_devices(self) -> List[EchonetDevice]:
        """
        Get list of discovered devices
        
        Returns:
            List of EchonetDevice objects
        """
        output = self._run_command("list")
        devices = []
        # Parse the output (simplified - in production, improve parsing)
        return devices
    
    def read_property(self, device_idx: int, obj_code: str, prop_code: str) -> Optional[str]:
        """
        Read a property from a device
        
        Args:
            device_idx: Device index (1-based)
            obj_code: Object code in hex (e.g., '029101')
            prop_code: Property code in hex (e.g., '80')
            
        Returns:
            Property value in hex, or None if failed
        """
        output = self._run_command(f"read {device_idx} {obj_code} {prop_code}")
        if "Property" in output and ":" in output:
            # Extract the property value
            parts = output.split(":")
            if len(parts) >= 2:
                return parts[-1].strip()
        return None
    
    def write_property(self, device_idx: int, obj_code: str, prop_code: str, data: str) -> bool:
        """
        Write a property to a device
        
        Args:
            device_idx: Device index (1-based)
            obj_code: Object code in hex (e.g., '029101')
            prop_code: Property code in hex (e.g., '80')
            data: Property value in hex (e.g., '30')
            
        Returns:
            True if successful
        """
        output = self._run_command(f"write {device_idx} {obj_code} {prop_code} {data}")
        return "successful" in output.lower()


# ============================================================================
# EXAMPLE USAGE
# ============================================================================

def example_basic_discovery():
    """Example 1: Basic device discovery"""
    print("\n=== Example 1: Basic Device Discovery ===\n")
    
    controller = EchonetController()
    
    # Discover devices
    if controller.discover():
        print("✓ Devices discovered successfully")
    else:
        print("✗ No devices found")
    
    # List devices
    devices = controller.list_devices()
    for device in devices:
        print(f"Device {device.index}: {device.ip_address}")


def example_light_control():
    """Example 2: Control a light device"""
    print("\n=== Example 2: Light Control ===\n")
    
    controller = EchonetController()
    
    # First discover
    controller.discover()
    
    # Control parameters
    device_idx = 1
    obj_code = "029101"  # Mono functional lighting
    
    # Check current status
    status = controller.read_property(device_idx, obj_code, "80")
    print(f"Current status: {status}")
    print(f"  30 = On")
    print(f"  31 = Off")
    
    # Turn on
    print("\nTurning on...")
    if controller.write_property(device_idx, obj_code, "80", "30"):
        print("✓ Turned on successfully")
    else:
        print("✗ Failed to turn on")
    
    time.sleep(1)
    
    # Turn off
    print("\nTurning off...")
    if controller.write_property(device_idx, obj_code, "80", "31"):
        print("✓ Turned off successfully")
    else:
        print("✗ Failed to turn off")


def example_brightness_control():
    """Example 3: Control light brightness"""
    print("\n=== Example 3: Brightness Control ===\n")
    
    controller = EchonetController()
    
    # First discover
    controller.discover()
    
    device_idx = 1
    obj_code = "029101"  # Mono functional lighting
    
    # Set different brightness levels
    brightness_levels = {
        "25%": 0x40,   # 64
        "50%": 0x80,   # 128
        "75%": 0xC0,   # 192
        "100%": 0xFF,  # 255
    }
    
    for name, hex_value in brightness_levels.items():
        hex_str = f"{hex_value:02x}"
        print(f"Setting brightness to {name} ({hex_str})...")
        controller.write_property(device_idx, obj_code, "b0", hex_str)
        time.sleep(0.5)
    
    print("✓ Brightness control complete")


def example_temperature_control():
    """Example 4: Control air conditioner temperature"""
    print("\n=== Example 4: Air Conditioner Temperature Control ===\n")
    
    controller = EchonetController()
    
    # First discover
    controller.discover()
    
    device_idx = 2  # Assuming 2nd device is A/C
    obj_code = "013001"  # Air conditioner
    
    # Set temperature to 22°C
    temp_celsius = 22
    hex_value = f"{temp_celsius:02x}"
    
    print(f"Setting temperature to {temp_celsius}°C...")
    if controller.write_property(device_idx, obj_code, "b3", hex_value):
        print("✓ Temperature set successfully")
    else:
        print("✗ Failed to set temperature")


def example_smart_meter():
    """Example 5: Read smart meter data"""
    print("\n=== Example 5: Smart Meter Reading ===\n")
    
    controller = EchonetController()
    
    # First discover
    controller.discover()
    
    device_idx = 3  # Assuming 3rd device is smart meter
    obj_code = "028801"  # Smart meter (electricity)
    
    # Read cumulative power consumption
    consumption = controller.read_property(device_idx, obj_code, "e0")
    print(f"Cumulative power consumption: {consumption} kWh")
    
    # Read instantaneous power
    power = controller.read_property(device_idx, obj_code, "e7")
    print(f"Instantaneous power: {power} W")


def example_batch_operations():
    """Example 6: Batch operations with multiple devices"""
    print("\n=== Example 6: Batch Operations ===\n")
    
    controller = EchonetController()
    
    # Discover devices
    controller.discover()
    
    # Define operations to perform
    operations = [
        {
            "device": 1,
            "object": "029101",
            "action": "Turn on",
            "property": "80",
            "value": "30"
        },
        {
            "device": 2,
            "object": "029101",
            "action": "Set to 50%",
            "property": "b0",
            "value": "80"
        },
        {
            "device": 3,
            "object": "013001",
            "action": "Set to 20°C",
            "property": "b3",
            "value": "14"
        },
    ]
    
    # Execute operations
    for op in operations:
        print(f"Device {op['device']}: {op['action']}...", end=" ")
        if controller.write_property(
            op['device'],
            op['object'],
            op['property'],
            op['value']
        ):
            print("✓ Success")
        else:
            print("✗ Failed")
        time.sleep(0.5)


def example_monitoring_loop():
    """Example 7: Continuous monitoring loop"""
    print("\n=== Example 7: Monitoring Loop ===\n")
    
    controller = EchonetController()
    
    # Discover devices
    controller.discover()
    
    device_idx = 3  # Smart meter
    obj_code = "028801"
    
    print("Monitoring power consumption for 60 seconds...")
    print("Time (s) | Power (W)")
    print("---------|----------")
    
    start_time = time.time()
    while time.time() - start_time < 60:
        elapsed = int(time.time() - start_time)
        power = controller.read_property(device_idx, obj_code, "e7")
        if power:
            print(f"{elapsed:7d} | {power}")
        time.sleep(5)  # Read every 5 seconds
    
    print("✓ Monitoring complete")


if __name__ == "__main__":
    # Run examples
    import argparse
    
    parser = argparse.ArgumentParser(description="ECHONET Controller Python Examples")
    parser.add_argument(
        "--example",
        type=int,
        choices=[1, 2, 3, 4, 5, 6, 7],
        help="Run specific example (1-7)"
    )
    
    args = parser.parse_args()
    
    examples = {
        1: example_basic_discovery,
        2: example_light_control,
        3: example_brightness_control,
        4: example_temperature_control,
        5: example_smart_meter,
        6: example_batch_operations,
        7: example_monitoring_loop,
    }
    
    if args.example:
        examples[args.example]()
    else:
        # Run all examples
        for i in range(1, 8):
            try:
                examples[i]()
            except Exception as e:
                print(f"Example {i} failed: {e}")
            print()
