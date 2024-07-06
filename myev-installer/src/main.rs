use evdev::{enumerate, Device};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const SERVICE_FILE_PATH: &str = "/etc/systemd/system/myev.service";
const BINARY_PATH: &str = "/usr/local/bin/myev";

fn main() -> io::Result<()> {
    println!("Building project...");
    build_project()?;

    println!("Selecting input device...");
    let device_path = select_physical_device()?;

    println!("Creating service file...");
    create_service_file(&device_path)?;

    println!("Installing binary...");
    install_binary()?;

    println!("Configuring systemd service...");
    configure_systemd_service()?;

    println!("Installation complete!");
    Ok(())
}

fn build_project() -> io::Result<()> {
    let output = Command::new("cargo")
        .args(&["build", "--release"])
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to build project",
        ));
    }
    Ok(())
}

fn select_physical_device() -> io::Result<String> {
    let mut devices: Vec<_> = enumerate().map(|(_, d)| d).collect();
    devices.reverse();

    for (i, d) in devices.iter().enumerate() {
        println!(
            "{}: {} ({:?})",
            i,
            d.name().unwrap_or("Unnamed device"),
            d.phys()
        );
    }

    print!("Select the device [0-{}]: ", devices.len() - 1);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let index: usize = input
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid input"))?;

    if index >= devices.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid device index",
        ));
    }

    let selected_device = &devices[index];
    Ok(selected_device.phys().unwrap_or_default().to_string())
}

fn create_service_file(device_path: &str) -> io::Result<()> {
    let service_content = format!(
        r#"[Unit]
Description=Dual Function Keys Service
After=network.target

[Service]
ExecStart={} {}
User=root
Restart=always
RestartSec=5s

[Install]
WantedBy=multi-user.target
"#,
        BINARY_PATH, device_path
    );

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
