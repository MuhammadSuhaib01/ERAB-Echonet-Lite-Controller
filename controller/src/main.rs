use echonet::protocol::{ESV, Message, Property};
use echonet::util::Bytes;
use echonet::{Controller, StandardDatabase};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub mod targets;

fn get_local_ip() -> Option<String> {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local_addr = socket.local_addr().ok()?;
    Some(local_addr.ip().to_string())
}

fn print_separator(width: usize) {
    println!("{}", "=".repeat(width));
}

fn print_devices(controller: &mut Controller, targets: &targets::Targets) {
    let nodes = controller.nodes();

    if nodes.is_empty() {
        println!("No devices found (you may need to run 'search' first).");
        return;
    }

    println!("\nDiscovered ECHONET Lite Devices:");
    print_separator(100);
    
    let std_db = StandardDatabase::shared();

    for (i, node) in nodes.iter().enumerate() {
        let node_addr = node.addr();
        let ip = node_addr.ip();
        let port = node_addr.port();
        let ip_str = ip.to_string();

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
        
        let primary_name = targets.devices.iter()
            .find(|d| d.ip == ip_str)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| ip_str.clone());

        println!(
            "Device [{}]: {} ({}:{}) - {}",
            i + 1,
            primary_name,
            ip,
            port,
            manufacture_name
        );

        // Fetch properties map from the node to force it to populate its objects
        let mut map_msg = Message::new();
        map_msg.set_esv(ESV::ReadRequest);
        map_msg.set_deoj(0x0EF001);
        let mut map_prop = Property::new();
        map_prop.set_code(0x9F); // Get property map
        map_msg.add_property(map_prop);
        let _ = controller.post_message(&node, &mut map_msg).recv_timeout(Duration::from_secs(1));
        
        // Print objects in the node
        for (j, obj) in node.objects().iter().enumerate() {
            let obj_code = obj.code();
            let class_name = obj.class_name();
            
            let eoj_hex = format!("{:06X}", obj_code);
            let obj_target_name = targets.devices.iter()
                .find(|d| d.ip == ip_str && d.eoj == eoj_hex)
                .map(|d| d.name.clone())
                .unwrap_or_else(|| class_name.to_string());
                
            println!("  Object [{}]: {:06X} - {}", j, obj_code, obj_target_name);

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

fn read_property(controller: &mut Controller, ip_str: &str, obj_code: u32, prop_code: u8) {
    let nodes = controller.nodes();

    let node = match nodes.iter().find(|n| n.addr().ip().to_string() == ip_str) {
        Some(n) => n,
        None => {
            println!("Device with IP {} not found (try running 'search' first)", ip_str);
            return;
        }
    };

    let mut msg = Message::new();
    msg.set_esv(ESV::ReadRequest);
    msg.set_deoj(obj_code);
    let mut prop = Property::new();
    prop.set_code(prop_code);
    msg.add_property(prop);

    println!("Reading property {:02X} from device {} object {:06X}...", prop_code, ip_str, obj_code);

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
    ip_str: &str,
    obj_code: u32,
    prop_code: u8,
    data: &[u8],
) {
    let nodes = controller.nodes();

    let node = match nodes.iter().find(|n| n.addr().ip().to_string() == ip_str) {
        Some(n) => n,
        None => {
            println!("Device with IP {} not found (try running 'search' first)", ip_str);
            return;
        }
    };

    let mut msg = Message::new();
    msg.set_esv(ESV::WriteRequest); // Use SetI (0x60) - Fire and forget, fixes hardware timeout loops
    msg.set_deoj(obj_code);
    let mut prop = Property::new();
    prop.set_code(prop_code);
    prop.set_data(data.to_vec());
    msg.add_property(prop);

    println!(
        "Sending write command {:02X}={} to device {} object {:06X}...",
        prop_code,
        hex::encode(data),
        ip_str,
        obj_code
    );

    // Send without blocking the channel to avoid freezing physical controller queues
    let _ = controller.post_message(&node, &mut msg);
    println!("Command dispatched to network.");
    
    // Give the bridge hardware 200ms to process the physical actuation
    thread::sleep(Duration::from_millis(200));
}

fn print_help() {
    println!("\nAvailable commands:");
    println!("  search              - Search for ECHONET Lite devices on the network");
    println!("  list                - List all discovered devices and their properties");
    println!("  read <ip> <obj> <prop> - Read a property from a device");
    println!("                        ip: device IP address (e.g., 172.16.18.51)");
    println!("                        obj: object code in hex (e.g., 029101)");
    println!("                        prop: property code in hex (e.g., 80)");
    println!("  write <ip> <obj> <prop> <data> - Write a property value");
    println!("                        ip: device IP address (e.g., 172.16.18.51)");
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
                if controller.nodes().len() <= 1 {
                    println!("No external devices known yet. Performing automatic search...");
                    controller.search();
                    thread::sleep(Duration::from_secs(3));
                    println!("Search complete.\n");
                }
                
                let loaded_targets = targets::Targets::load_from_file("targets.json").unwrap_or_default();
                print_devices(&mut controller, &loaded_targets);
            }
            "read" => {
                if parts.len() < 4 {
                    println!("Usage: read <ip> <object_code_hex> <property_code_hex>");
                    println!("Example: read 172.16.18.51 013001 80");
                } else {
                    let ip_str = parts[1];
                    if let Ok(obj_code) = u32::from_str_radix(parts[2], 16) {
                        if let Ok(prop_code) = u8::from_str_radix(parts[3], 16) {
                            read_property(&mut controller, ip_str, obj_code, prop_code);
                        } else {
                            println!("Invalid property code (must be hex)");
                        }
                    } else {
                        println!("Invalid object code (must be hex)");
                    }
                }
            }
            "write" => {
                if parts.len() < 5 {
                    println!("Usage: write <ip> <object_code_hex> <property_code_hex> <data_hex>");
                    println!("Example: write 172.16.18.51 013001 80 30");
                } else {
                    let ip_str = parts[1];
                    if let Ok(obj_code) = u32::from_str_radix(parts[2], 16) {
                        if let Ok(prop_code) = u8::from_str_radix(parts[3], 16) {
                            if let Ok(data) = hex::decode(parts[4]) {
                                write_property(&mut controller, ip_str, obj_code, prop_code, &data);
                            } else {
                                println!("Invalid hex data");
                            }
                        } else {
                            println!("Invalid property code (must be hex)");
                        }
                    } else {
                        println!("Invalid object code (must be hex)");
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
