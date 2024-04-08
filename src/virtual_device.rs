// use nix::fcntl::{open, OFlag};
// use nix::ioctl_write_int;
// use nix::sys::stat::Mode;
use nix::unistd::close;
use std::os::unix::io::RawFd;

// Define necessary ioctl commands
ioctl_write_int!(ui_set_evbit, b'U', 100);
ioctl_write_int!(ui_set_keybit, b'U', 101);
ioctl_write_int!(ui_dev_create, b'U', 1);

// Path to the uinput device
const UINPUT_PATH: &str = "/dev/uinput";

// Represents a virtual device. For simplicity, we only include a file descriptor here.
struct VirtualDevice {
    fd: RawFd,
}

impl VirtualDevice {
    // Creates a new virtual device
    fn new() -> nix::Result<Self> {
        let fd = open(
            UINPUT_PATH,
            OFlag::O_WRONLY | OFlag::O_NONBLOCK,
            Mode::empty(),
        )?;
        Ok(Self { fd })
    }

    // Configures the device to send key events
    fn configure_keyboard(&self) -> nix::Result<()> {
        // Enable key event
        unsafe { ui_set_evbit(self.fd, libc::EV_KEY as i32) }?;

        // Enable specific key: here we enable the 'A' key as an example
        unsafe { ui_set_keybit(self.fd, libc::KEY_A as i32) }?;

        Ok(())
    }

    // Finalizes the device setup
    fn create(&self) -> nix::Result<()> {
        unsafe { ui_dev_create(self.fd) }?;
        Ok(())
    }
}

impl Drop for VirtualDevice {
    // Ensures the device is properly destroyed when it goes out of scope
    fn drop(&mut self) {
        let _ = close(self.fd);
    }
}

fn create_virtual_device() -> nix::Result<()> {
    let device = VirtualDevice::new()?;
    device.configure_keyboard()?;
    device.create()?;

    // The virtual device is now ready to use, you can inject events here.
    // Remember to sleep a bit to ensure the system has time to register the device
    // before attempting to inject events.

    println!("Virtual device created successfully.");
    Ok(())
}
use evdev_rs::enums::*;
use evdev_rs::Device;
// use nix::fcntl::OFlag;
// use nix::sys::stat::Mode;
use nix::unistd::write;
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub fn merge_devices() -> Result<(), Box<dyn std::error::Error>> {
    let device_file = File::open("/dev/input/event17").unwrap();
    // Open the reference device
    let reference_device = Device::new_from_file(device_file)?;

    // Open /dev/uinput
    let uinput_file = File::open("/dev/uinput").expect("Failed to open /dev/uinput");

    // Fetch capabilities from the reference device
    let caps = reference_device.supported_events();

    // For simplicity, this example will only handle key events
    // More complex implementations would handle additional event types
    if let Some(key_events) = caps.get(&EV_KEY) {
        for key_event in key_events.iter() {
            println!("Key: {:?}", key_event);
            // Here, you'd use ioctl to set up the virtual device's capabilities
            // This step is complex and platform-specific; see below for hints
        }
    }

    // Finally, create the virtual device by writing to /dev/uinput
    // In a real implementation, you'd use ioctl calls here
    // This is a placeholder to show where device creation logic would go
    let bytes_written = write(uinput_file.as_raw_fd(), b"placeholder").expect("Failed to write");
    println!("Bytes written: {}", bytes_written);

    Ok(())
}
