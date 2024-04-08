use evdev::InputEventKind;
use evdev::Key;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use uinput::event::keyboard::Key as UKey;
use uinput::event::Keyboard as UBoard;
use uinput::Event;

use evdev_rs::enums::*;
use evdev_rs::{Device, DeviceWrapper};
use uinput::event::Event::Keyboard;

// mod virtual_device;

const HOLD_TIME: u8 = 200;
const DOUBLE_TAP_TIME: u8 = 150;

#[derive(Debug)]
struct DualFunctionKey {
    tap: UKey,
    hold: UKey,
}

impl DualFunctionKey {
    fn new(tap: UKey, hold: UKey) -> Self {
        DualFunctionKey { tap, hold }
    }
}

// Define your dual-function key mappings
fn dual_function_mappings() -> HashMap<Key, DualFunctionKey> {
    let mut mappings = HashMap::new();
    // mappings.insert(
    //     Key::KEY_BACKSPACE,
    //     DualFunctionKey::new(UKey::BackSpace, UKey::LeftShift),
    // );
    // mappings.insert(
    //     Key::KEY_SPACE,
    //     DualFunctionKey::new(UKey::Space, UKey::RightShift),
    // );
    mappings.insert(
        Key::KEY_CAPSLOCK,
        DualFunctionKey::new(UKey::Esc, UKey::LeftMeta),
    );
    mappings
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open the physical device you want to replicate/merge
    let physical_device = Device::new_from_path(Path::new("/dev/input/event17")).unwrap();

    // Create a new virtual device builder
    let dev_file = Path::new("/dev/uinput");
    let mut builder = uinput::open(dev_file)
        .unwrap()
        .name("Virtual Keyboard")
        .unwrap();

    // Example: Set event types and keys based on the physical device
    if physical_device.has_event_code(&EventCode::EV_KEY(EV_KEY::KEY_1)) {
        builder = builder.event(UBoard::Key(UKey::_1)).unwrap();
    }
    // Repeat for other keys and event types as needed

    // Finally, create the virtual device
    let mut device = builder.create().unwrap();
    // let dev_file = Path::new("/dev/uinput");
    // let mut device = uinput::open(dev_file)
    //     .unwrap()
    //     .name("Keyboard")
    //     .unwrap()
    //     .event(uinput::event::Keyboard::All)
    //     .unwrap()
    //     .create()
    //     .unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    let mappings = dual_function_mappings();
    let mut press_times: HashMap<UBoard, Instant> = HashMap::new();

    let mut d = pick_device(); // Ensure this function is defined or replaced accordingly
                               //
    let _ = d.supported_keys().iter().map(|f| {
        if physical_device.has_event_code(f) {
            builder = builder.event(UBoard::Key(UKey::_1)).unwrap();
        }
    });
    // d.grab().unwrap();
    let mut events = d.into_event_stream()?;

    loop {
        let ev = events.next_event().await?;
        // println!("{ev:#?}");
        if let Some(dual_key) = mappings.get(&Key(ev.code())) {
            match ev.value() {
                0 => {
                    if let Some(press_time) = press_times.remove(&UBoard::Key(dual_key.tap)) {
                        let duration_held = press_time.elapsed();
                        if duration_held < Duration::from_millis(200) {
                            device.release(&dual_key.hold).unwrap();
                            device.press(&dual_key.tap).unwrap();
                            device.release(&dual_key.tap).unwrap();
                        } else {
                            device.release(&dual_key.hold).unwrap();
                        }
                    }
                }
                1 => {
                    device.press(&dual_key.hold).unwrap();
                    press_times.insert(UBoard::Key(dual_key.tap), Instant::now());
                }
                _ => (),
            }
        } else if let InputEventKind::Key(key) = ev.kind() {
            let ukey = &key_convert_evdev_to_uinput(key);
            let _ = match ev.value() {
                0 => {
                    press_times.remove(ukey);
                    device.release(ukey)
                }
                1 => device.press(ukey),
                _ => Ok(()),
            };
        }
        device.synchronize().unwrap();
    }
}

fn key_convert_evdev_to_uinput(key: evdev::Key) -> uinput::event::Keyboard {
    use uinput::event::Keyboard;
    match key {
        Key::KEY_1 => Keyboard::Key(UKey::_1),
        Key::KEY_2 => Keyboard::Key(UKey::_2),
        Key::KEY_3 => Keyboard::Key(UKey::_3),
        Key::KEY_4 => Keyboard::Key(UKey::_4),
        Key::KEY_5 => Keyboard::Key(UKey::_5),
        Key::KEY_6 => Keyboard::Key(UKey::_6),
        Key::KEY_7 => Keyboard::Key(UKey::_7),
        Key::KEY_8 => Keyboard::Key(UKey::_8),
        Key::KEY_9 => Keyboard::Key(UKey::_9),
        Key::KEY_0 => Keyboard::Key(UKey::_0),
        Key::KEY_Q => Keyboard::Key(UKey::Q),
        Key::KEY_W => Keyboard::Key(UKey::W),
        Key::KEY_E => Keyboard::Key(UKey::E),
        Key::KEY_R => Keyboard::Key(UKey::R),
        Key::KEY_T => Keyboard::Key(UKey::T),
        Key::KEY_Y => Keyboard::Key(UKey::Y),
        Key::KEY_U => Keyboard::Key(UKey::U),
        Key::KEY_I => Keyboard::Key(UKey::I),
        Key::KEY_O => Keyboard::Key(UKey::O),
        Key::KEY_P => Keyboard::Key(UKey::P),
        Key::KEY_A => Keyboard::Key(UKey::A),
        Key::KEY_S => Keyboard::Key(UKey::S),
        Key::KEY_D => Keyboard::Key(UKey::D),
        Key::KEY_F => Keyboard::Key(UKey::F),
        Key::KEY_G => Keyboard::Key(UKey::G),
        Key::KEY_H => Keyboard::Key(UKey::H),
        Key::KEY_J => Keyboard::Key(UKey::J),
        Key::KEY_K => Keyboard::Key(UKey::K),
        Key::KEY_L => Keyboard::Key(UKey::L),
        Key::KEY_Z => Keyboard::Key(UKey::Z),
        Key::KEY_X => Keyboard::Key(UKey::X),
        Key::KEY_C => Keyboard::Key(UKey::C),
        Key::KEY_V => Keyboard::Key(UKey::V),
        Key::KEY_B => Keyboard::Key(UKey::B),
        Key::KEY_N => Keyboard::Key(UKey::N),
        Key::KEY_M => Keyboard::Key(UKey::M),
        Key::KEY_LEFTCTRL => Keyboard::Key(UKey::LeftControl),
        Key::KEY_CAPSLOCK => Keyboard::Key(UKey::CapsLock),
        Key::KEY_ESC => Keyboard::Key(UKey::Esc),
        Key::KEY_MINUS => Keyboard::Key(UKey::Minus),
        Key::KEY_EQUAL => Keyboard::Key(UKey::Equal),
        Key::KEY_BACKSPACE => Keyboard::Key(UKey::BackSpace),
        Key::KEY_TAB => Keyboard::Key(UKey::Tab),
        Key::KEY_LEFTBRACE => Keyboard::Key(UKey::LeftBrace),
        Key::KEY_RIGHTBRACE => Keyboard::Key(UKey::RightBrace),
        Key::KEY_ENTER => Keyboard::Key(UKey::Enter),
        Key::KEY_SEMICOLON => Keyboard::Key(UKey::SemiColon),
        Key::KEY_APOSTROPHE => Keyboard::Key(UKey::Apostrophe),
        Key::KEY_GRAVE => Keyboard::Key(UKey::Grave),
        Key::KEY_LEFTSHIFT => Keyboard::Key(UKey::LeftShift),
        Key::KEY_BACKSLASH => Keyboard::Key(UKey::BackSlash),
        Key::KEY_COMMA => Keyboard::Key(UKey::Comma),
        Key::KEY_DOT => Keyboard::Key(UKey::Dot),
        Key::KEY_SLASH => Keyboard::Key(UKey::Slash),
        Key::KEY_RIGHTSHIFT => Keyboard::Key(UKey::RightShift),
        Key::KEY_LEFTALT => Keyboard::Key(UKey::LeftAlt),
        Key::KEY_SPACE => Keyboard::Key(UKey::Space),
        Key::KEY_F1 => Keyboard::Key(UKey::F1),
        Key::KEY_F2 => Keyboard::Key(UKey::F2),
        Key::KEY_F3 => Keyboard::Key(UKey::F3),
        Key::KEY_F4 => Keyboard::Key(UKey::F4),
        Key::KEY_F5 => Keyboard::Key(UKey::F5),
        Key::KEY_F6 => Keyboard::Key(UKey::F6),
        Key::KEY_F7 => Keyboard::Key(UKey::F7),
        Key::KEY_F8 => Keyboard::Key(UKey::F8),
        Key::KEY_F9 => Keyboard::Key(UKey::F9),
        Key::KEY_F10 => Keyboard::Key(UKey::F10),
        Key::KEY_F11 => Keyboard::Function(uinput::event::keyboard::Function::F11),
        Key::KEY_F12 => Keyboard::Function(uinput::event::keyboard::Function::F12),
        Key::KEY_NUMLOCK => Keyboard::Key(UKey::NumLock),
        Key::KEY_SCROLLLOCK => Keyboard::Key(UKey::ScrollLock),
        Key::KEY_LEFTMETA => Keyboard::Key(UKey::LeftMeta),
        Key::KEY_RIGHTMETA => Keyboard::Key(UKey::RightMeta),
        Key::KEY_RIGHTCTRL => Keyboard::Key(UKey::RightControl),
        Key::KEY_MUTE => Keyboard::Misc(uinput::event::keyboard::Misc::Mute),
        Key::KEY_VOLUMEDOWN => Keyboard::Misc(uinput::event::keyboard::Misc::VolumeDown),
        Key::KEY_VOLUMEUP => Keyboard::Misc(uinput::event::keyboard::Misc::VolumeUp),
        Key::KEY_NEXTSONG => Keyboard::Misc(uinput::event::keyboard::Misc::NextSong),
        Key::KEY_PLAYPAUSE => Keyboard::Misc(uinput::event::keyboard::Misc::PlayPause),
        Key::KEY_PREVIOUSSONG => Keyboard::Misc(uinput::event::keyboard::Misc::PreviousSong),
        Key::KEY_BRIGHTNESSUP => Keyboard::Misc(uinput::event::keyboard::Misc::BrightnessUp),
        Key::KEY_BRIGHTNESSDOWN => Keyboard::Misc(uinput::event::keyboard::Misc::BrightnessDown),
        Key::KEY_KBDILLUMUP => Keyboard::Misc(uinput::event::keyboard::Misc::IllumUp),
        Key::KEY_KBDILLUMDOWN => Keyboard::Misc(uinput::event::keyboard::Misc::IllumDown),
        Key::KEY_DASHBOARD => Keyboard::Misc(uinput::event::keyboard::Misc::DashBoard),
        Key::KEY_SCALE => Keyboard::Misc(uinput::event::keyboard::Misc::Scale),
        Key::KEY_SYSRQ => Keyboard::Misc(uinput::event::keyboard::Misc::Print),
        Key::KEY_DELETE => Keyboard::Key(UKey::Delete),
        Key::KEY_PAGEUP => Keyboard::Key(UKey::PageUp),
        Key::KEY_PAGEDOWN => Keyboard::Key(UKey::PageDown),
        Key::KEY_HOME => Keyboard::Key(UKey::Home),
        Key::KEY_END => Keyboard::Key(UKey::End),
        Key::KEY_LEFT => Keyboard::Key(UKey::Left),
        Key::KEY_RIGHT => Keyboard::Key(UKey::Right),
        Key::KEY_UP => Keyboard::Key(UKey::Up),
        Key::KEY_DOWN => Keyboard::Key(UKey::Down),
        _ => Keyboard::Key(UKey::Reserved),
    }
}

pub fn pick_device() -> evdev::Device {
    use std::io::prelude::*;

    let mut args = std::env::args_os();
    args.next();
    if let Some(dev_file) = args.next() {
        let dev = evdev::Device::open(dev_file).unwrap();
        let d = evdev::Device::from(dev);
        d
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
// cargo build --release && sudo target/release/myev /dev/input/event17
