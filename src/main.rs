use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, InputEvent, Key,
};
use std::{
    io,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

static DOUBLE_TAP_TIMEOUT: Duration = Duration::from_millis(150);
static LONG_PRESS_TIMEOUT: Duration = Duration::from_millis(200);

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

    //prevent physical device from sending events so that the virtual one can handle everything
    physical_device.grab()?;

    //a second to process new device
    sleep(Duration::from_secs(1)).await;

    let mut caps_remap = KeyConfig {
        key: Key::KEY_CAPSLOCK,
        on_tap: Key::KEY_ESC,
        on_hold: Key::KEY_LEFTMETA,
        held: false,
        start_time: SystemTime::UNIX_EPOCH,
    };

    let mut interrupted = false;

    loop {
        for event in physical_device.fetch_events()?.into_iter() {
            if event.code() == caps_remap.key.code() {
                if caps_remap.start_time == SystemTime::UNIX_EPOCH {
                    caps_remap.start_time = SystemTime::now();
                }

                if event.value() == 0 && caps_remap.start_time != SystemTime::UNIX_EPOCH {
                    caps_remap.start_time = SystemTime::UNIX_EPOCH;
                    key_up(&mut virtual_device, caps_remap.on_hold)?;
                    key_up(&mut virtual_device, caps_remap.on_tap)?;
                    continue;
                }

                let duration_held = SystemTime::now()
                    .duration_since(caps_remap.start_time)
                    .map_err(|_| io::Error::last_os_error())?;
                println!("{:#?}", duration_held.as_millis());

                let key = if duration_held.as_millis() > LONG_PRESS_TIMEOUT.as_millis() {
                    caps_remap.on_hold
                } else {
                    caps_remap.on_tap
                };

                // let key = match event.value() {
                //     2 => {
                //         if !caps_remap.held {
                //             caps_remap.held = true;
                //             caps_remap.on_hold
                //         } else {
                //             Key::KEY_RESERVED
                //         }
                //     }
                //     0 => {
                //         if caps_remap.held {
                //             caps_remap.held = false;
                //             caps_remap.on_hold
                //         } else {
                //             key_up(&mut virtual_device, caps_remap.on_hold)?;
                //
                //             if !interrupted {
                //                 key_down(&mut virtual_device, caps_remap.on_tap)?;
                //                 sleep(Duration::from_millis(10)).await;
                //                 caps_remap.on_tap
                //             } else {
                //                 println!("not interrupted");
                //                 interrupted = false;
                //                 Key::KEY_RESERVED
                //             }
                //         }
                //     }
                //     1 => {
                //         caps_remap.start_time = event.timestamp();
                //         caps_remap.on_hold
                //     }
                //     _ => Key::KEY_RESERVED,
                // };

                virtual_device.emit(&[InputEvent::new(
                    event.event_type(),
                    key.code(),
                    event.value(),
                )])?;
            } else {
                if caps_remap.held {
                    interrupted = true;
                    println!("{event:#?}");
                    println!("interrupted");
                }

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
    start_time: SystemTime,
}

fn key_down(d: &mut VirtualDevice, key: Key) -> std::io::Result<()> {
    d.emit(&[InputEvent::new(evdev::EventType(0x01), key.code(), 1)])?;
    Ok(())
}
fn key_up(d: &mut VirtualDevice, key: Key) -> std::io::Result<()> {
    d.emit(&[InputEvent::new(evdev::EventType(0x01), key.code(), 0)])?;
    Ok(())
}
