use echonet::log::Logger;
use echonet::protocol::{ESV, Message, Property};
use echonet::util::Bytes;
use echonet::{Controller, StandardDatabase};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

fn print_separator(width: usize) {
    println!("{}", "=".repeat(width));
}

fn print_devices(controller: &mut Controller) {
    let nodes = controller.nodes();

    if nodes.is_empty() {
        println!("No devices found.");
        return;
    }

    println!("\nDiscovered ECHONET Lite Devices:");
    print_separator(100);
    
    let std_db = StandardDatabase::shared();

    for (i, node) in nodes.iter().enumerate() {
        let node_addr = node.addr();
        let ip = node_addr.ip();
        let port = node_addr.port();

        // Attempt to get manufacturer code
        let mut manufacture_name = String::from("Unknown");
        let mut msg = Message::new();
        msg.set_esv(ESV::ReadRequest);
        msg.set_deoj(0x0EF001); // Node profile object
        let mut prop = Property::new();
        prop.set_code(0x8A); // Manufacturer code
        msg.add_property(prop);

        let rx = controller.post_message(&node, &mut msg);
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(response) => {
                let props = response.properties();
                if !props.is_empty() {
                    let manufacture_code = Bytes::to_u32(props[0].data()) as u32;
                    if let Some(mfg) = std_db.find_manufacture(manufacture_code) {
                        manufacture_name = mfg.name().to_string();
                    }
                }
            }
            Err(_) => {}
        }

        println!(
            "Device [{}]: {} ({}:{}) - {}",
            i + 1,
            ip,
            ip,
            port,
            manufacture_name
        );

        // Print objects in the node
        for (j, obj) in node.objects().iter().enumerate() {
            let obj_code = obj.code();
            let class_name = obj.class_name();
            println!("  Object [{}]: {:06X} - {}", j, obj_code, class_name);

            // Print mandatory properties for the object
            for prop_def in obj.properties() {
                if prop_def.is_read_required() {
                    println!(
                        "    Property: {:02X} - {} (Read-only: {})",
                        prop_def.code(),
                        prop_def.name(),
                        !prop_def.is_write_required()
                    );
                }
            }
        }
        println!();
    }
    print_separator(100);
}

fn read_property(controller: &mut Controller, device_idx: usize, obj_code: u32, prop_code: u8) {
    let nodes = controller.nodes();

    if device_idx >= nodes.len() {
        println!("Invalid device index");
        return;
    }

    let node = &nodes[device_idx];

    let mut msg = Message::new();
    msg.set_esv(ESV::ReadRequest);
    msg.set_deoj(obj_code);
    let mut prop = Property::new();
    prop.set_code(prop_code);
    msg.add_property(prop);

    println!("Reading property {:02X} from device {} object {:06X}...", prop_code, device_idx + 1, obj_code);

    let rx = controller.post_message(&node, &mut msg);
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(response) => {
            let props = response.properties();
            if !props.is_empty() {
                println!(
                    "Property {:02X}: {}",
                    props[0].code(),
                    hex::encode(props[0].data())
                );
            } else {
                println!("No property data received");
            }
        }
        Err(_) => println!("Timeout: No response from device"),
    }
}

fn write_property(
    controller: &mut Controller,
    device_idx: usize,
    obj_code: u32,
    prop_code: u8,
    data: &[u8],
) {
    let nodes = controller.nodes();

    if device_idx >= nodes.len() {
        println!("Invalid device index");
        return;
    }

    let node = &nodes[device_idx];

    let mut msg = Message::new();
    msg.set_esv(ESV::WriteRequest);
    msg.set_deoj(obj_code);
    let mut prop = Property::new();
    prop.set_code(prop_code);
    prop.set_data(data.to_vec());
    msg.add_property(prop);

    println!(
        "Writing property {:02X}={} to device {} object {:06X}...",
        prop_code,
        hex::encode(data),
        device_idx + 1,
        obj_code
    );

    let rx = controller.post_message(&node, &mut msg);
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(response) => {
            if response.esv() as u8 == 0x71 {
                // Write successful
                println!("Property write successful");
            } else {
                println!("Write completed with ESV: {:02X}", response.esv() as u8);
            }
        }
        Err(_) => println!("Timeout: No response from device"),
    }
}

fn print_help() {
    println!("\nAvailable commands:");
    println!("  search              - Search for ECHONET Lite devices on the network");
    println!("  list                - List all discovered devices and their properties");
    println!("  read <dev> <obj> <prop> - Read a property from a device");
    println!("                        dev: device index (1-based)");
    println!("                        obj: object code in hex (e.g., 029101)");
    println!("                        prop: property code in hex (e.g., 80)");
    println!("  write <dev> <obj> <prop> <data> - Write a property value");
    println!("                        dev: device index (1-based)");
    println!("                        obj: object code in hex");
    println!("                        prop: property code in hex");
    println!("                        data: hex data (e.g., 30)");
    println!("  help                - Show this help message");
    println!("  exit                - Stop controller and exit");
}

fn main() {
    let _ = env_logger::builder().filter_level(log::LevelFilter::Warn).try_init();

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║     ECHONET Lite Controller - uecho-rs powered         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    match get_local_ip() {
        Some(ip) => println!("Controller IP address: {}\n", ip),
        None => println!("Warning: Could not determine IP address\n"),
    }

    let mut controller = Controller::new();

    if !controller.start() {
        eprintln!("Failed to start controller");
        return;
    }

    println!("Controller started successfully");
    println!("Type 'help' for available commands\n");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "exit" | "quit" => {
                println!("Stopping controller...");
                controller.stop();
                println!("Controller stopped. Goodbye!");
                break;
            }
            "search" => {
                println!("Searching for devices (this may take a few seconds)...");
                controller.search();
                thread::sleep(Duration::from_secs(3));
                println!("Search complete.\n");
            }
            "list" => {
                print_devices(&mut controller);
            }
            "read" => {
                if parts.len() < 4 {
                    println!("Usage: read <device_index> <object_code_hex> <property_code_hex>");
                    println!("Example: read 1 029101 80");
                } else {
                    if let Ok(dev_idx) = parts[1].parse::<usize>() {
                        if let Ok(obj_code) = u32::from_str_radix(parts[2], 16) {
                            if let Ok(prop_code) = u8::from_str_radix(parts[3], 16) {
                                read_property(&mut controller, dev_idx - 1, obj_code, prop_code);
                            } else {
                                println!("Invalid property code (must be hex)");
                            }
                        } else {
                            println!("Invalid object code (must be hex)");
                        }
                    } else {
                        println!("Invalid device index (must be numeric)");
                    }
                }
            }
            "write" => {
                if parts.len() < 5 {
                    println!("Usage: write <device_index> <object_code_hex> <property_code_hex> <data_hex>");
                    println!("Example: write 1 029101 80 30");
                } else {
                    if let Ok(dev_idx) = parts[1].parse::<usize>() {
                        if let Ok(obj_code) = u32::from_str_radix(parts[2], 16) {
                            if let Ok(prop_code) = u8::from_str_radix(parts[3], 16) {
                                if let Ok(data) = hex::decode(parts[4]) {
                                    write_property(&mut controller, dev_idx - 1, obj_code, prop_code, &data);
                                } else {
                                    println!("Invalid hex data");
                                }
                            } else {
                                println!("Invalid property code (must be hex)");
                            }
                        } else {
                            println!("Invalid object code (must be hex)");
                        }
                    } else {
                        println!("Invalid device index (must be numeric)");
                    }
                }
            }
            "help" => {
                print_help();
            }
            "" => {}
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", parts[0]);
            }
        }
    }
}
