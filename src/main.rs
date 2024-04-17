use evdev::{uinput::VirtualDeviceBuilder, AttributeSet, InputEvent, Key};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut physical_device = pick_device();

    // Initialize an empty AttributeSet<Key> to store the supported keys
    let mut keys = AttributeSet::<Key>::new();

    // If the device supports keys, iterate over them and add to the 'keys' set
    if let Some(supported_keys) = physical_device.supported_keys() {
        supported_keys.iter().for_each(|device_key| {
            // Insert a new Key based on the numeric value of the device_key
            keys.insert(Key::new(device_key.code()));
        });
    }

    // For demonstration, print out the numeric values of the keys we just added
    println!("Available Keys:");
    for key in keys.iter() {
        println!("  {:#?}", key);
    }

    let mut virtual_device = VirtualDeviceBuilder::new()?
        .name("Virtual Keyboard")
        .with_keys(&keys)?
        .build()
        .unwrap();

    for path in virtual_device.enumerate_dev_nodes_blocking()? {
        let path = path?;
        println!("Virtual device available as {}", path.display());
    }

    //a second to process new device
    sleep(Duration::from_secs(1)).await;

    //prevent physical device from sending events so that the virtual one can handle everything
    physical_device.grab()?;

    let mut caps_remap = KeyConfig {
        key: Key::KEY_CAPSLOCK,
        on_tap: Key::KEY_ESC,
        on_hold: Key::KEY_LEFTMETA,
        held: false,
    };

    loop {
        for event in physical_device.fetch_events()?.into_iter() {
            if event.code() == caps_remap.key.code() {
                let code = match event.value() {
                    2 => {
                        if !caps_remap.held {
                            caps_remap.held = true;
                            caps_remap.on_hold
                        } else {
                            Key::KEY_RESERVED
                        }
                    }
                    0 => {
                        if caps_remap.held {
                            caps_remap.held = false;
                            caps_remap.on_hold
                        } else {
                            virtual_device.emit(&[InputEvent::new(
                                event.event_type(),
                                caps_remap.on_hold.code(),
                                0,
                            )])?;

                            virtual_device.emit(&[InputEvent::new(
                                event.event_type(),
                                caps_remap.on_tap.code(),
                                1,
                            )])?;

                            sleep(Duration::from_millis(10)).await;
                            caps_remap.on_tap
                        }
                    }
                    1 => caps_remap.on_hold,
                    _ => Key::KEY_RESERVED,
                };

                virtual_device.emit(&[InputEvent::new(
                    event.event_type(),
                    code.code(),
                    event.value(),
                )])?;
            } else {
                virtual_device.emit(&[event])?;
            }
        }
        sleep(Duration::from_millis(10)).await;
    }
}

pub fn pick_device() -> evdev::Device {
    use std::io::prelude::*;

    let mut args = std::env::args_os();
    args.next();
    if let Some(dev_file) = args.next() {
        evdev::Device::open(dev_file).unwrap()
    } else {
        let mut devices = evdev::enumerate().map(|t| t.1).collect::<Vec<_>>();
        // readdir returns them in reverse order from their eventN names for some reason
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

#[derive(Debug)]
struct KeyConfig {
    key: Key,
    on_tap: Key,
    on_hold: Key,
    held: bool,
}
