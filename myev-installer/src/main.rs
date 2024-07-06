use evdev::Device;
use std::fs;
use std::io::{self};
use std::path::PathBuf;
use std::process::Command;

const SERVICE_FILE_PATH: &str = "/etc/systemd/system/myev.service";
const BINARY_PATH: &str = "/usr/bin/myev";

fn main() -> io::Result<()> {
    // println!("Building project...");
    // build_project()?;

    println!("Selecting input device...");
    let device = select_physical_device();

    println!("Creating service file...");
    create_service_file(device)?;

    println!("Installing binary...");
    install_binary()?;

    println!("Configuring systemd service...");
    configure_systemd_service()?;

    println!("Installation complete!");
    Ok(())
}

// fn build_project() -> io::Result<()> {
//     let output = Command::new("cargo")
//         .args(&["build", "--release"])
//         .output()?;
//
//     if !output.status.success() {
//         return Err(io::Error::new(
//             io::ErrorKind::Other,
//             "Failed to build project",
//         ));
//     }
//     Ok(())
// }

fn select_physical_device() -> Device {
    use std::io::prelude::*;

    let mut args = std::env::args_os();
    args.next();
    if let Some(dev_file) = args.next() {
        Device::open(dev_file).unwrap()
    } else {
        let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
        devices.reverse();
        for (i, d) in devices.iter().enumerate() {
            println!("{}: {}", i, d.name().unwrap_or("Unnamed device"));
        }
        print!("Select the device [0-{}]: ", devices.len());
        let _ = std::io::stdout().flush();
        let mut chosen = String::new();
        std::io::stdin().read_line(&mut chosen).unwrap();
        let n = chosen.trim().parse::<usize>().unwrap();
        devices.into_iter().nth(n).unwrap()
    }
}

fn create_service_file(device: Device) -> io::Result<()> {
    let service_content = format!(
        r#"[Unit]
Description=Key Remapping Service
After=network.target

[Service]
ExecStart={} {}
User=root
Restart=always
RestartSec=5s

[Install]
WantedBy=multi-user.target
"#,
        BINARY_PATH,
        find_device_path(&device)
            .expect("Expected to be able to find the path to device")
            .to_str()
            .expect("Expected to be able to turn pathbuf into a string")
    );

    println!("{service_content}");

    fs::write(SERVICE_FILE_PATH, service_content)?;
    Ok(())
}

fn install_binary() -> io::Result<()> {
    let source_path = "target/release/myev";
    fs::copy(source_path, BINARY_PATH)?;
    Ok(())
}

fn configure_systemd_service() -> io::Result<()> {
    Command::new("systemctl")
        .args(&["daemon-reload"])
        .status()?;
    Command::new("systemctl")
        .args(&["enable", "myev.service"])
        .status()?;
    Command::new("systemctl")
        .args(&["start", "myev.service"])
        .status()?;
    Ok(())
}

fn find_device_path(device: &Device) -> io::Result<PathBuf> {
    let device_name = device.name().unwrap_or("Unknown");
    let phys_path = device.physical_path().unwrap_or("Unknown");

    // Check in /dev/input/by-path first
    if let Ok(entries) = fs::read_dir("/dev/input/by-path") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(dev) = Device::open(&path) {
                if dev.name() == device.name() && dev.physical_path() == device.physical_path() {
                    return Ok(path);
                }
            }
        }
    }
    // If not found in by-path, check in /dev/input/by-id
    if let Ok(entries) = fs::read_dir("/dev/input/by-id") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Ok(dev) = Device::open(&path) {
                if dev.name() == device.name() && dev.physical_path() == device.physical_path() {
                    return Ok(path);
                }
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        format!("Device path not found for {} ({})", device_name, phys_path),
    ))
}
