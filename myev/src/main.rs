use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, Device, FetchEventsSynced, InputEvent, InputEventKind, Key, MiscType,
};
use serde::Deserialize;
use std::str::FromStr;
use std::{
    fs, io,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;
use toml;

const HOLD_TIMEOUT: Duration = Duration::from_millis(200);
const CONFIG_FILE: &str = "/home/eran/code/myev/key_config.toml";

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = load_config()?;
    let mut remapper = KeyRemapper::new(config).await?;
    remapper.run().await
}

fn load_config() -> io::Result<Config> {
    let config_str = fs::read_to_string(CONFIG_FILE)?;
    let config: Config =
        toml::from_str(&config_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(config)
}

#[derive(Deserialize)]
struct Config {
    key_mappings: Vec<KeyMapping>,
}

#[derive(Deserialize)]
struct KeyMapping {
    key: String,
    on_tap: String,
    on_hold: String,
}

struct KeyConfig {
    key: Key,
    on_tap: Key,
    on_hold: Key,
    state: KeyState,
}

#[derive(Debug)]
struct KeyState {
    is_pressed: bool,
    time_pressed: Option<SystemTime>,
    is_interrupted: bool,
}

impl KeyConfig {
    fn new(key: Key, on_tap: Key, on_hold: Key) -> Self {
        Self {
            key,
            on_tap,
            on_hold,
            state: KeyState {
                is_pressed: false,
                time_pressed: None,
                is_interrupted: false,
            },
        }
    }

    async fn process_event(
        &mut self,
        event: InputEvent,
        device: &mut VirtualDevice,
    ) -> io::Result<()> {
        match event.value() {
            1 => self.handle_key_press(device).await,
            0 => self.handle_key_release(device).await,
            _ => Ok(()),
        }
    }

    async fn handle_key_press(&mut self, device: &mut VirtualDevice) -> io::Result<()> {
        self.state.is_pressed = true;
        self.state.time_pressed = Some(SystemTime::now());
        emit_key_event(device, self.on_hold, 1).await
    }

    async fn handle_key_release(&mut self, device: &mut VirtualDevice) -> io::Result<()> {
        self.state.is_pressed = false;
        emit_key_event(device, self.on_hold, 0).await?;

        if let Some(press_time) = self.state.time_pressed {
            let duration_held = SystemTime::now().duration_since(press_time).unwrap();
            if duration_held < HOLD_TIMEOUT && !self.state.is_interrupted {
                tap(device, self.on_tap).await?;
            }
        }

        self.state.time_pressed = None;
        self.state.is_interrupted = false;
        Ok(())
    }

    fn interrupt(&mut self) {
        self.state.is_interrupted = true;
    }

    fn is_pressed(&self) -> bool {
        self.state.is_pressed
    }
}

struct KeyRemapper {
    physical_device: Device,
    virtual_device: VirtualDevice,
    key_configs: Vec<KeyConfig>,
}

impl KeyRemapper {
    async fn new(config: Config) -> io::Result<Self> {
        let mut physical_device = select_physical_device().await;
        let supported_keys = gather_supported_keys(&physical_device)?;
        let virtual_device = create_virtual_device(&supported_keys).await?;

        prevent_physical_device_input(&mut physical_device).await?;

        let key_configs = config
            .key_mappings
            .into_iter()
            .map(|mapping| {
                KeyConfig::new(
                    Key::from_str(&mapping.key).unwrap(),
                    Key::from_str(&mapping.on_tap).unwrap(),
                    Key::from_str(&mapping.on_hold).unwrap(),
                )
            })
            .collect();

        Ok(Self {
            physical_device,
            virtual_device,
            key_configs,
        })
    }

    async fn run(&mut self) -> io::Result<()> {
        loop {
            let events: Vec<InputEvent> = self.fetch_events()?.collect();
            for event in events {
                self.handle_event(event).await?;
            }
            sleep(Duration::from_millis(10)).await;
        }
    }

    fn fetch_events(&mut self) -> io::Result<FetchEventsSynced> {
        self.physical_device.fetch_events()
    }

    async fn handle_event(&mut self, event: InputEvent) -> io::Result<()> {
        println!("EVENT: {:#?}", event);

        let mut handled = false;
        for config in &mut self.key_configs {
            if event.code() == config.key.code() {
                config
                    .process_event(event, &mut self.virtual_device)
                    .await?;
                handled = true;
                break;
            }
        }

        if !handled {
            self.handle_non_configured_event(event).await?;
        }

        Ok(())
    }

    async fn handle_non_configured_event(&mut self, event: InputEvent) -> io::Result<()> {
        let mut interrupt = false;
        let not_sync_or_scan_event = event.kind() != InputEventKind::Misc(MiscType::MSC_SCAN)
            && event.kind() != InputEventKind::Synchronization(evdev::Synchronization::SYN_REPORT);

        if not_sync_or_scan_event {
            for config in &mut self.key_configs {
                if config.is_pressed() {
                    match event.kind() {
                        InputEventKind::Key(Key::KEY_RIGHTSHIFT) => {
                            emit_key_event(&mut self.virtual_device, Key::KEY_RIGHTSHIFT, 0)
                                .await?;
                            tap(&mut self.virtual_device, Key::KEY_ESC).await?;
                            return Ok(());
                        }
                        InputEventKind::Key(Key::KEY_ESC) if config.state.is_interrupted => {
                            tap(&mut self.virtual_device, Key::KEY_ESC).await?;
                            return Ok(());
                        }
                        _ => {
                            interrupt = true;
                        }
                    }
                }
            }
        }

        if interrupt {
            for config in &mut self.key_configs {
                if config.is_pressed() {
                    config.interrupt();
                }
            }
        }

        self.virtual_device.emit(&[event])?;
        Ok(())
    }
}

async fn tap(device: &mut VirtualDevice, key: Key) -> io::Result<()> {
    emit_key_event(device, key, 1).await?;
    sleep(Duration::from_millis(10)).await;
    emit_key_event(device, key, 0).await
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

async fn emit_key_event(device: &mut VirtualDevice, key: Key, value: i32) -> io::Result<()> {
    device.emit(&[InputEvent::new(evdev::EventType(0x01), key.code(), value)])?;
    Ok(())
}

async fn select_physical_device() -> Device {
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
