use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Device, InputEvent, InputEventKind, Key, MiscType,
};
use std::{
    io,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

static HOLD_TIMEOUT: Duration = Duration::from_millis(200);

struct KeyConfig {
    key: Key,
    on_tap: Key,
    on_hold: Key,
    interrupted: bool,
    time_pressed: Option<SystemTime>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut physical_device = pick_device().await;
    let supported_keys = gather_supported_keys(&physical_device)?;
    let mut virtual_device = create_virtual_device(&supported_keys).await?;

    prevent_physical_device_input(&mut physical_device).await?;

    let key_configs = initialize_key_config();

    event_loop(&mut physical_device, &mut virtual_device, key_configs).await
}

async fn process_custom_key_event(
    event: InputEvent,
    config: &mut KeyConfig,
    device: &mut VirtualDevice,
) -> io::Result<()> {
    match event.value() {
        1 => {
            emit_key_event(device, config.on_hold, 1).await?;
            config.time_pressed = Some(SystemTime::now());
        }
        0 => {
            emit_key_event(device, config.on_hold, 0).await?;

            let duration_held = SystemTime::now()
                .duration_since(config.time_pressed.unwrap())
                .unwrap();

            let condition_for_tap_met = (duration_held < HOLD_TIMEOUT) && !config.interrupted;

            println!("condition_for_tap_met: {condition_for_tap_met}\nduration_held {duration_held:#?}\ninterrupted: {:#?}", config.interrupted);
            if condition_for_tap_met {
                tap(device, config.on_tap).await?;
            }

            config.time_pressed = None;
            config.interrupted = false;
        }
        _ => {}
    }
    // println!("{config:#?}");
    Ok(())
}

async fn tap(device: &mut VirtualDevice, key: Key) -> io::Result<()> {
    emit_key_event(device, key, 1).await?;
    sleep(Duration::from_millis(10)).await;
    emit_key_event(device, key, 0).await?;

    Ok(())
}

fn gather_supported_keys(device: &Device) -> io::Result<AttributeSet<Key>> {
    let mut keys = AttributeSet::<Key>::new();
    if let Some(supported_keys) = device.supported_keys() {
        supported_keys.iter().for_each(|k| {
            keys.insert(Key::new(k.code()));
        });
    }
    Ok(keys)
}

async fn create_virtual_device(supported_keys: &AttributeSet<Key>) -> io::Result<VirtualDevice> {
    let mut virtual_device = VirtualDeviceBuilder::new()?
        .name("Virtual Keyboard")
        .with_keys(supported_keys)?
        .build()?;
    announce_virtual_device(&mut virtual_device)?;
    Ok(virtual_device)
}

fn announce_virtual_device(device: &mut VirtualDevice) -> io::Result<()> {
    for path in device.enumerate_dev_nodes_blocking()? {
        println!("Virtual device available as {}", path?.display());
    }
    Ok(())
}

async fn prevent_physical_device_input(device: &mut Device) -> io::Result<()> {
    device.grab()?;
    sleep(Duration::from_secs(1)).await;
    Ok(())
}

fn initialize_key_config() -> Vec<KeyConfig> {
    vec![KeyConfig {
        key: Key::KEY_CAPSLOCK,
        on_tap: Key::KEY_ESC,
        on_hold: Key::KEY_LEFTMETA,
        interrupted: false,
        time_pressed: None,
    }]
}

async fn event_loop(
    physical_device: &mut Device,
    virtual_device: &mut VirtualDevice,
    mut key_configs: Vec<KeyConfig>,
) -> io::Result<()> {
    loop {
        for event in physical_device.fetch_events()?.into_iter() {
            handle_event(event, &mut key_configs, virtual_device).await?;
        }
        sleep(Duration::from_millis(10)).await;
    }
}

async fn handle_event(
    event: InputEvent,
    key_configs: &mut Vec<KeyConfig>,
    device: &mut VirtualDevice,
) -> io::Result<()> {
    // if event is in KeyConfigs then handle it
    for config in &mut *key_configs {
        if event.code() == config.key.code() {
            process_custom_key_event(event, config, device).await?;
            return Ok(());
        }
    }

    // if event is not in keyconfigs, but a keyconfig is held down, set it to interrupted
    for config in key_configs {
        if config.time_pressed.is_some() {
            let not_sync_or_scan_event = event.kind() != InputEventKind::Misc(MiscType::MSC_SCAN)
                && event.kind()
                    != InputEventKind::Synchronization(evdev::Synchronization::SYN_REPORT);

            if not_sync_or_scan_event {
                println!("EVENT: {event:#?}");
                //FIXME: this should be in a separate function. just me compensating for being an
                //idiot.
                if event.kind() == InputEventKind::Key(Key::KEY_RIGHTSHIFT) {
                    config.interrupted = false;

                    emit_key_event(device, Key::KEY_RIGHTSHIFT, 0).await?;
                    emit_key_event(device, Key::KEY_ESC, 1).await?;
                    sleep(Duration::from_millis(10)).await;
                    emit_key_event(device, Key::KEY_ESC, 0).await?;
                } else if event.kind() == InputEventKind::Key(Key::KEY_ESC) && config.interrupted {
                    config.interrupted = false;
                    emit_key_event(device, Key::KEY_ESC, 1).await?;
                    sleep(Duration::from_millis(10)).await;
                    emit_key_event(device, Key::KEY_ESC, 0).await?;
                } else {
                    config.interrupted = true;
                }
            }
        }
    }

    //otherwise just pass the event through
    device.emit(&[event])?;

    Ok(())
}

async fn emit_key_event(device: &mut VirtualDevice, key: Key, value: i32) -> io::Result<()> {
    device.emit(&[InputEvent::new(evdev::EventType(0x01), key.code(), value)])?;
    Ok(())
}

async fn pick_device() -> evdev::Device {
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

// The select_physical_device and KeyConfig struct definitions remain the same as in the original code.
