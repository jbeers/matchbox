use matchbox_vm::types::{BxNativeFunction, BxVM, BxValue};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;

static USB_HID_READY: AtomicBool = AtomicBool::new(false);
static HID_QUEUED: AtomicU32 = AtomicU32::new(0);
static HID_SENT: AtomicU32 = AtomicU32::new(0);
static HID_ERRORS: AtomicU32 = AtomicU32::new(0);
static MOUSE_BUTTONS: AtomicU8 = AtomicU8::new(0);
static HID_COMMANDS: OnceLock<mpsc::SyncSender<HidCommand>> = OnceLock::new();
const DEFAULT_PRODUCT_NAME: &str = "MatchBox HID";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HidSnapshot {
    pub installed: bool,
    pub ready: bool,
    pub mounted: bool,
    pub queued: u32,
    pub sent: u32,
    pub errors: u32,
}

#[derive(Clone, Copy, Debug)]
enum HidCommand {
    MouseMove { dx: i8, dy: i8 },
    MouseButton { button: u8, down: bool },
    MouseScroll { dx: i8, dy: i8 },
    KeyPress { keycode: u8 },
    KeyRelease { keycode: u8 },
    KeyReleaseAll,
}

pub fn snapshot() -> HidSnapshot {
    let installed = USB_HID_READY.load(Ordering::Acquire);
    let mounted = installed && usb_hid::is_mounted();
    HidSnapshot {
        installed,
        ready: installed && usb_hid::is_mouse_ready(),
        mounted,
        queued: HID_QUEUED.load(Ordering::Acquire),
        sent: HID_SENT.load(Ordering::Acquire),
        errors: HID_ERRORS.load(Ordering::Acquire),
    }
}

pub fn register_bifs() -> HashMap<String, BxNativeFunction> {
    let mut map = HashMap::new();
    map.insert(
        "esp32registerashid".to_string(),
        esp32_register_as_hid as BxNativeFunction,
    );
    map.insert(
        "esp32RegisterAsHID".to_string(),
        esp32_register_as_hid as BxNativeFunction,
    );
    map.insert(
        "esp32usbmousemove".to_string(),
        esp32_usb_mouse_move as BxNativeFunction,
    );
    map.insert(
        "esp32USBMouseMove".to_string(),
        esp32_usb_mouse_move as BxNativeFunction,
    );
    map.insert(
        "esp32UsbMouseMove".to_string(),
        esp32_usb_mouse_move as BxNativeFunction,
    );
    map.insert(
        "esp32usbmouseclick".to_string(),
        esp32_usb_mouse_click as BxNativeFunction,
    );
    map.insert(
        "esp32USBMouseClick".to_string(),
        esp32_usb_mouse_click as BxNativeFunction,
    );
    map.insert(
        "esp32UsbMouseClick".to_string(),
        esp32_usb_mouse_click as BxNativeFunction,
    );
    map.insert(
        "esp32usbmousedown".to_string(),
        esp32_usb_mouse_down as BxNativeFunction,
    );
    map.insert(
        "esp32USBMouseDown".to_string(),
        esp32_usb_mouse_down as BxNativeFunction,
    );
    map.insert(
        "esp32UsbMouseDown".to_string(),
        esp32_usb_mouse_down as BxNativeFunction,
    );
    map.insert(
        "esp32usbmouseup".to_string(),
        esp32_usb_mouse_up as BxNativeFunction,
    );
    map.insert(
        "esp32USBMouseUp".to_string(),
        esp32_usb_mouse_up as BxNativeFunction,
    );
    map.insert(
        "esp32UsbMouseUp".to_string(),
        esp32_usb_mouse_up as BxNativeFunction,
    );
    map.insert(
        "esp32usbmousescroll".to_string(),
        esp32_usb_mouse_scroll as BxNativeFunction,
    );
    map.insert(
        "esp32USBMouseScroll".to_string(),
        esp32_usb_mouse_scroll as BxNativeFunction,
    );
    map.insert(
        "esp32UsbMouseScroll".to_string(),
        esp32_usb_mouse_scroll as BxNativeFunction,
    );
    map.insert(
        "esp32usbkeyboardpress".to_string(),
        esp32_usb_keyboard_press as BxNativeFunction,
    );
    map.insert(
        "esp32USBKeyboardPress".to_string(),
        esp32_usb_keyboard_press as BxNativeFunction,
    );
    map.insert(
        "esp32UsbKeyboardPress".to_string(),
        esp32_usb_keyboard_press as BxNativeFunction,
    );
    map.insert(
        "esp32usbkeyboardrelease".to_string(),
        esp32_usb_keyboard_release as BxNativeFunction,
    );
    map.insert(
        "esp32USBKeyboardRelease".to_string(),
        esp32_usb_keyboard_release as BxNativeFunction,
    );
    map.insert(
        "esp32UsbKeyboardRelease".to_string(),
        esp32_usb_keyboard_release as BxNativeFunction,
    );
    map.insert(
        "esp32usbkeyboardreleaseall".to_string(),
        esp32_usb_keyboard_release_all as BxNativeFunction,
    );
    map.insert(
        "esp32USBKeyboardReleaseAll".to_string(),
        esp32_usb_keyboard_release_all as BxNativeFunction,
    );
    map.insert(
        "esp32UsbKeyboardReleaseAll".to_string(),
        esp32_usb_keyboard_release_all as BxNativeFunction,
    );
    map.insert(
        "esp32usbhidready".to_string(),
        esp32_usb_hid_ready as BxNativeFunction,
    );
    map.insert(
        "esp32USBHidReady".to_string(),
        esp32_usb_hid_ready as BxNativeFunction,
    );
    map.insert(
        "esp32UsbHidReady".to_string(),
        esp32_usb_hid_ready as BxNativeFunction,
    );
    map
}

fn esp32_register_as_hid(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() > 1 {
        return Err(format!(
            "esp32RegisterAsHID accepts 0 or 1 arguments, got {}",
            args.len()
        ));
    }

    let product_name = args
        .first()
        .filter(|value| !value.is_null())
        .map(|value| vm.to_string(*value));

    crate::diagnostics::record_event("hid register requested");
    ensure_usb_hid_ready(product_name.as_deref())?;
    crate::diagnostics::record_event("hid register complete");
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_hid_ready(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if !args.is_empty() {
        return Err(format!(
            "esp32USBHidReady accepts 0 arguments, got {}",
            args.len()
        ));
    }
    Ok(BxValue::new_bool(
        USB_HID_READY.load(Ordering::Acquire) && usb_hid::is_mounted(),
    ))
}

fn esp32_usb_mouse_move(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 2 {
        return Err(format!(
            "esp32USBMouseMove requires 2 arguments, got {}",
            args.len()
        ));
    }

    let dx = clamp_i8(args[0].as_number(), "x")?;
    let dy = clamp_i8(args[1].as_number(), "y")?;

    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::MouseMove { dx, dy })?;

    Ok(BxValue::new_bool(true))
}

fn esp32_usb_mouse_click(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err(format!(
            "esp32USBMouseClick requires 1 argument, got {}",
            args.len()
        ));
    }
    let button = clamp_button(args[0].as_number())?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    // Click = down + up with a short delay in the worker
    enqueue_command(HidCommand::MouseButton { button, down: true })?;
    enqueue_command(HidCommand::MouseButton {
        button,
        down: false,
    })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_mouse_down(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err(format!(
            "esp32USBMouseDown requires 1 argument, got {}",
            args.len()
        ));
    }
    let button = clamp_button(args[0].as_number())?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::MouseButton { button, down: true })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_mouse_up(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err(format!(
            "esp32USBMouseUp requires 1 argument, got {}",
            args.len()
        ));
    }
    let button = clamp_button(args[0].as_number())?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::MouseButton {
        button,
        down: false,
    })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_mouse_scroll(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 2 {
        return Err(format!(
            "esp32USBMouseScroll requires 2 arguments, got {}",
            args.len()
        ));
    }
    let dx = clamp_i8(args[0].as_number(), "scrollX")?;
    let dy = clamp_i8(args[1].as_number(), "scrollY")?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::MouseScroll { dx, dy })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_keyboard_press(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err(format!(
            "esp32USBKeyboardPress requires 1 argument, got {}",
            args.len()
        ));
    }
    let keycode = clamp_keycode(args[0].as_number())?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::KeyPress { keycode })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_keyboard_release(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() != 1 {
        return Err(format!(
            "esp32USBKeyboardRelease requires 1 argument, got {}",
            args.len()
        ));
    }
    let keycode = clamp_keycode(args[0].as_number())?;
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::KeyRelease { keycode })?;
    Ok(BxValue::new_bool(true))
}

fn esp32_usb_keyboard_release_all(_vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if !args.is_empty() {
        return Err(format!(
            "esp32USBKeyboardReleaseAll accepts 0 arguments, got {}",
            args.len()
        ));
    }
    ensure_usb_hid_ready(None)?;
    usb_hid::ensure_mounted()?;
    enqueue_command(HidCommand::KeyReleaseAll)?;
    Ok(BxValue::new_bool(true))
}

fn clamp_i8(value: f64, name: &str) -> Result<i8, String> {
    if !value.is_finite() {
        return Err(format!("{} must be a finite number", name));
    }

    let rounded = value.round();
    if rounded < i8::MIN as f64 || rounded > i8::MAX as f64 {
        return Err(format!("{} must be between -128 and 127", name));
    }

    Ok(rounded as i8)
}

fn clamp_button(value: f64) -> Result<u8, String> {
    if !value.is_finite() {
        return Err("button must be a valid number (1=left, 2=right, 3=middle)".to_string());
    }
    let btn = value.round() as i64;
    if btn < 1 || btn > 7 {
        return Err("button must be between 1 and 7 (1=left, 2=right, 3=middle)".to_string());
    }
    Ok(btn as u8)
}

fn clamp_keycode(value: f64) -> Result<u8, String> {
    if !value.is_finite() {
        return Err("keycode must be a finite number between 0 and 255".to_string());
    }
    let code = value.round() as i64;
    if code < 0 || code > 255 {
        return Err("keycode must be between 0 and 255".to_string());
    }
    Ok(code as u8)
}

fn ensure_usb_hid_ready(product_name: Option<&str>) -> Result<(), String> {
    if USB_HID_READY.load(Ordering::Acquire) {
        if let Some(product_name) = product_name {
            usb_hid::ensure_product_name_matches(product_name)?;
        }
        return Ok(());
    }

    usb_hid::install(product_name.unwrap_or(DEFAULT_PRODUCT_NAME))?;
    MOUSE_BUTTONS.store(0, Ordering::Release);
    start_hid_worker();
    USB_HID_READY.store(true, Ordering::Release);
    println!("[esp32-bif] USB HID ready");
    Ok(())
}

fn enqueue_command(command: HidCommand) -> Result<(), String> {
    let tx = HID_COMMANDS
        .get()
        .ok_or_else(|| "USB HID worker is not running".to_string())?;
    crate::diagnostics::record_event(format!("hid queue {:?}", command));
    tx.send(command)
        .map_err(|error| format!("USB HID command queue is stopped: {}", error))?;
    HID_QUEUED.fetch_add(1, Ordering::AcqRel);
    crate::diagnostics::record_event(format!("hid queued {:?}", command));
    Ok(())
}

fn start_hid_worker() {
    HID_COMMANDS.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<HidCommand>(8);
        let builder = thread::Builder::new()
            .name("matchbox-hid-worker".to_string())
            .stack_size(4096);
        match builder.spawn(move || {
            println!("[esp32-bif] USB HID worker started");
            crate::diagnostics::record_event("hid worker started");
            while let Ok(command) = rx.recv() {
                let desc = format!("{:?}", command);
                crate::diagnostics::record_event(format!("hid worker send start {}", desc));
                let result = match command {
                    HidCommand::MouseMove { dx, dy } => usb_hid::send_mouse_move_report(dx, dy),
                    HidCommand::MouseButton { button, down } => {
                        usb_hid::send_mouse_button_report(button, down)
                    }
                    HidCommand::MouseScroll { dx, dy } => usb_hid::send_mouse_scroll_report(dx, dy),
                    HidCommand::KeyPress { keycode } => usb_hid::send_key_report(keycode, true),
                    HidCommand::KeyRelease { keycode } => usb_hid::send_key_report(keycode, false),
                    HidCommand::KeyReleaseAll => usb_hid::send_key_release_all(),
                };
                match result {
                    Ok(()) => {
                        HID_SENT.fetch_add(1, Ordering::AcqRel);
                        crate::diagnostics::record_event(format!(
                            "hid worker send complete {}",
                            desc
                        ));
                        println!("[esp32-bif] USB HID worker send complete")
                    }
                    Err(error) => {
                        HID_ERRORS.fetch_add(1, Ordering::AcqRel);
                        crate::diagnostics::record_event(format!(
                            "hid worker send error {}",
                            error
                        ));
                        println!("[esp32-bif] USB HID worker send error: {}", error)
                    }
                }
            }
            crate::diagnostics::record_event("hid worker stopped");
            println!("[esp32-bif] USB HID worker stopped");
        }) {
            Ok(_) => {}
            Err(error) => {
                crate::diagnostics::record_event(format!("hid worker failed to start {}", error));
                println!("[esp32-bif] USB HID worker failed to start: {}", error)
            }
        }
        tx
    });
}

mod usb_hid {
    use esp_idf_sys::{self as sys, ESP_OK};
    use std::ffi::CString;
    use std::sync::atomic::Ordering;
    use std::sync::{Mutex, OnceLock};

    const TUSB_DESC_DEVICE: u8 = 0x01;
    const TUSB_DESC_CONFIGURATION: u8 = 0x02;
    const TUSB_DESC_INTERFACE: u8 = 0x04;
    const TUSB_DESC_ENDPOINT: u8 = 0x05;
    const TUSB_DESC_HID: u8 = 0x21;

    const TUSB_CLASS_HID: u8 = 0x03;
    const HID_SUBCLASS_BOOT: u8 = 0x01;
    const HID_PROTOCOL_MOUSE: u8 = 0x02;
    const HID_PROTOCOL_KEYBOARD: u8 = 0x01;

    const TUSB_XFER_INTERRUPT: u8 = 0x03;
    const TUSB_DIR_IN: u8 = 0x80;
    const HID_ITF_MOUSE: u8 = 0;
    const HID_ITF_KEYBOARD: u8 = 1;
    const HID_EP_IN_MOUSE: u8 = TUSB_DIR_IN | 1;
    const HID_EP_IN_KEYBOARD: u8 = TUSB_DIR_IN | 2;
    const HID_EP_SIZE: u16 = 8;
    const HID_POLL_MS: u8 = 10;
    const CONFIG_TOTAL_LEN: u16 = 9 + (9 + 9 + 7) + (9 + 9 + 7);

    #[repr(C, packed)]
    struct DeviceDescriptor {
        b_length: u8,
        b_descriptor_type: u8,
        bcd_usb: u16,
        b_device_class: u8,
        b_device_sub_class: u8,
        b_device_protocol: u8,
        b_max_packet_size0: u8,
        id_vendor: u16,
        id_product: u16,
        bcd_device: u16,
        i_manufacturer: u8,
        i_product: u8,
        i_serial_number: u8,
        b_num_configurations: u8,
    }

    #[repr(C)]
    struct TinyUsbConfig {
        device_descriptor: *const DeviceDescriptor,
        string_descriptor: *const *const u8,
        string_descriptor_count: i32,
        external_phy: bool,
        configuration_descriptor: *const u8,
        self_powered: bool,
        vbus_monitor_io: i32,
    }

    unsafe extern "C" {
        fn tinyusb_driver_install(config: *const TinyUsbConfig) -> i32;
        fn tud_mounted() -> bool;
        fn tud_hid_n_ready(instance: u8) -> bool;
        fn tud_hid_n_report(instance: u8, report_id: u8, report: *const u8, len: u16) -> bool;
    }

    static DEVICE_DESCRIPTOR: DeviceDescriptor = DeviceDescriptor {
        b_length: 18,
        b_descriptor_type: TUSB_DESC_DEVICE,
        bcd_usb: 0x0200,
        b_device_class: 0,
        b_device_sub_class: 0,
        b_device_protocol: 0,
        b_max_packet_size0: 64,
        id_vendor: 0x303a,
        id_product: 0x4001,
        bcd_device: 0x0100,
        i_manufacturer: 1,
        i_product: 2,
        i_serial_number: 3,
        b_num_configurations: 1,
    };

    unsafe impl Sync for DeviceDescriptor {}

    static CONFIG_DESCRIPTOR: [u8; CONFIG_TOTAL_LEN as usize] = [
        // Configuration descriptor
        9,
        TUSB_DESC_CONFIGURATION,
        (CONFIG_TOTAL_LEN & 0xff) as u8,
        (CONFIG_TOTAL_LEN >> 8) as u8,
        2,    // bNumInterfaces
        1,    // bConfigurationValue
        0,    // iConfiguration
        0x80, // bmAttributes (bus powered)
        100,  // bMaxPower (200mA)
        // ── Interface 0: Mouse ──
        9,
        TUSB_DESC_INTERFACE,
        HID_ITF_MOUSE,
        0, // bAlternateSetting
        1, // bNumEndpoints
        TUSB_CLASS_HID,
        HID_SUBCLASS_BOOT,
        HID_PROTOCOL_MOUSE,
        0, // iInterface
        9,
        TUSB_DESC_HID,
        0x11,
        0x01, // HID spec release
        0,    // country code
        1,    // num descriptors
        0x22, // report descriptor type
        (MOUSE_REPORT_DESCRIPTOR.len() & 0xff) as u8,
        (MOUSE_REPORT_DESCRIPTOR.len() >> 8) as u8,
        7,
        TUSB_DESC_ENDPOINT,
        HID_EP_IN_MOUSE,
        TUSB_XFER_INTERRUPT,
        (HID_EP_SIZE & 0xff) as u8,
        (HID_EP_SIZE >> 8) as u8,
        HID_POLL_MS,
        // ── Interface 1: Keyboard ──
        9,
        TUSB_DESC_INTERFACE,
        HID_ITF_KEYBOARD,
        0, // bAlternateSetting
        1, // bNumEndpoints
        TUSB_CLASS_HID,
        HID_SUBCLASS_BOOT,
        HID_PROTOCOL_KEYBOARD,
        0, // iInterface
        9,
        TUSB_DESC_HID,
        0x11,
        0x01,
        0,
        1,
        0x22,
        (KEYBOARD_REPORT_DESCRIPTOR.len() & 0xff) as u8,
        (KEYBOARD_REPORT_DESCRIPTOR.len() >> 8) as u8,
        7,
        TUSB_DESC_ENDPOINT,
        HID_EP_IN_KEYBOARD,
        TUSB_XFER_INTERRUPT,
        (HID_EP_SIZE & 0xff) as u8,
        (HID_EP_SIZE >> 8) as u8,
        HID_POLL_MS,
    ];

    static MOUSE_REPORT_DESCRIPTOR: [u8; 50] = [
        0x05, 0x01, 0x09, 0x02, 0xa1, 0x01, 0x09, 0x01, 0xa1, 0x00, 0x05, 0x09, 0x19, 0x01, 0x29,
        0x03, 0x15, 0x00, 0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x81, 0x02, 0x95, 0x01, 0x75, 0x05,
        0x81, 0x03, 0x05, 0x01, 0x09, 0x30, 0x09, 0x31, 0x15, 0x81, 0x25, 0x7f, 0x75, 0x08, 0x95,
        0x02, 0x81, 0x06, 0xc0, 0xc0,
    ];

    // Boot keyboard report descriptor: 8-byte report (modifier, reserved, 6 key slots)
    static KEYBOARD_REPORT_DESCRIPTOR: [u8; 45] = [
        0x05, 0x01, // Usage Page (Generic Desktop)
        0x09, 0x06, // Usage (Keyboard)
        0xa1, 0x01, // Collection (Application)
        0x05, 0x07, //   Usage Page (Keyboard/Keypad)
        0x19, 0xe0, //   Usage Minimum (224 = Left Control)
        0x29, 0xe7, //   Usage Maximum (231 = Right GUI)
        0x15, 0x00, //   Logical Minimum (0)
        0x25, 0x01, //   Logical Maximum (1)
        0x75, 0x01, //   Report Size (1)
        0x95, 0x08, //   Report Count (8)
        0x81, 0x02, //   Input (Data, Variable, Absolute) — modifier byte
        0x95, 0x01, //   Report Count (1)
        0x75, 0x08, //   Report Size (8)
        0x81, 0x03, //   Input (Constant, Variable, Absolute) — reserved byte
        0x95, 0x06, //   Report Count (6)
        0x75, 0x08, //   Report Size (8)
        0x15, 0x00, //   Logical Minimum (0)
        0x25, 0x65, //   Logical Maximum (101)
        0x05, 0x07, //   Usage Page (Keyboard/Keypad)
        0x19, 0x00, //   Usage Minimum (0)
        0x29, 0x65, //   Usage Maximum (101)
        0x81, 0x00, //   Input (Data, Array) — 6 key slots
        0xc0, // End Collection
    ];

    static KEYBOARD_REPORT_LEN: usize = 8;

    static MANUFACTURER: &[u8] = b"MatchBox\0";
    static SERIAL: &[u8] = b"MBX-HID-01\0";
    static PRODUCT_NAME: OnceLock<CString> = OnceLock::new();
    static STRING_DESCRIPTORS: OnceLock<StringDescriptors> = OnceLock::new();
    static REPORT_LOCK: Mutex<()> = Mutex::new(());
    #[repr(transparent)]
    struct StringDescriptors([*const u8; 3]);

    unsafe impl Sync for TinyUsbConfig {}
    unsafe impl Send for StringDescriptors {}
    unsafe impl Sync for StringDescriptors {}

    pub fn ensure_product_name_matches(product_name: &str) -> Result<(), String> {
        let Some(current_name) = PRODUCT_NAME.get() else {
            return Ok(());
        };

        if current_name.to_str().ok() == Some(product_name) {
            return Ok(());
        }

        Err(format!(
            "USB HID is already registered as '{}'; reboot before registering as '{}'",
            current_name.to_string_lossy(),
            product_name
        ))
    }

    pub fn install(product_name: &str) -> Result<(), String> {
        ensure_product_name_matches(product_name)?;
        let product_name = CString::new(product_name)
            .map_err(|_| "esp32RegisterAsHID device name cannot contain NUL bytes".to_string())?;
        let product_name = PRODUCT_NAME.get_or_init(|| product_name);
        let string_descriptors = STRING_DESCRIPTORS.get_or_init(|| {
            StringDescriptors([
                MANUFACTURER.as_ptr(),
                product_name.as_ptr(),
                SERIAL.as_ptr(),
            ])
        });

        let config = TinyUsbConfig {
            device_descriptor: &DEVICE_DESCRIPTOR,
            string_descriptor: string_descriptors.0.as_ptr(),
            string_descriptor_count: string_descriptors.0.len() as i32,
            external_phy: false,
            configuration_descriptor: CONFIG_DESCRIPTOR.as_ptr(),
            self_powered: false,
            vbus_monitor_io: -1,
        };

        let result = unsafe { tinyusb_driver_install(&config) };
        if result != ESP_OK {
            return Err(format!(
                "tinyusb_driver_install failed with ESP error {}",
                result
            ));
        }

        Ok(())
    }

    pub fn is_mouse_ready() -> bool {
        unsafe { tud_mounted() && tud_hid_n_ready(HID_ITF_MOUSE) }
    }

    pub fn is_mounted() -> bool {
        unsafe { tud_mounted() }
    }

    pub fn ensure_mounted() -> Result<(), String> {
        if is_mounted() {
            Ok(())
        } else {
            Err("USB HID device is not mounted; reconnect the native USB/OTG port after enabling HID".to_string())
        }
    }

    pub fn ensure_mouse_ready() -> Result<(), String> {
        if is_mouse_ready() {
            Ok(())
        } else {
            Err("USB HID mouse is not mounted or ready yet; reconnect the native USB/OTG port after enabling HID".to_string())
        }
    }

    pub fn send_mouse_move_report(dx: i8, dy: i8) -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        let buttons = super::MOUSE_BUTTONS.load(Ordering::Acquire);
        send_report(HID_ITF_MOUSE, &[buttons, dx as u8, dy as u8])?;
        send_report(HID_ITF_MOUSE, &[buttons, 0, 0])
    }

    pub fn send_mouse_button_report(button: u8, down: bool) -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        let mask = 1u8 << (button - 1);
        let current = super::MOUSE_BUTTONS.load(Ordering::Acquire);
        let buttons = if down {
            current | mask
        } else {
            current & !mask
        };
        send_report(HID_ITF_MOUSE, &[buttons, 0, 0])?;
        super::MOUSE_BUTTONS.store(buttons, Ordering::Release);
        Ok(())
    }

    pub fn send_mouse_scroll_report(dx: i8, dy: i8) -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        let buttons = super::MOUSE_BUTTONS.load(Ordering::Acquire);
        // Scroll is sent on the Z axis (byte 4 in the boot protocol)
        send_report(HID_ITF_MOUSE, &[buttons, 0, 0, dy as u8])?;
        unsafe {
            sys::vTaskDelay(10);
        }
        send_report(HID_ITF_MOUSE, &[buttons, 0, 0, 0])
    }

    pub fn send_key_report(keycode: u8, pressed: bool) -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        // Boot keyboard report: 1 byte modifiers, 1 reserved, 6 key slots
        let report = if pressed {
            [0u8, 0, keycode, 0, 0, 0, 0, 0]
        } else {
            [0u8, 0, 0, 0, 0, 0, 0, 0]
        };
        send_report(HID_ITF_KEYBOARD, &report)
    }

    pub fn send_key_release_all() -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        send_report(HID_ITF_KEYBOARD, &[0u8, 0, 0, 0, 0, 0, 0, 0])
    }

    fn send_report(instance: u8, data: &[u8]) -> Result<(), String> {
        for _ in 0..200 {
            if unsafe { tud_mounted() && tud_hid_n_ready(instance) } {
                let sent =
                    unsafe { tud_hid_n_report(instance, 0, data.as_ptr(), data.len() as u16) };
                if sent {
                    return Ok(());
                }
            }

            unsafe {
                sys::vTaskDelay(10);
            }
        }

        println!(
            "[esp32-bif] USB HID instance {} is not mounted or ready yet",
            instance
        );
        Err(format!(
            "USB HID instance {} is not mounted or ready yet",
            instance
        ))
    }

    #[unsafe(no_mangle)]
    extern "C" fn tud_hid_get_report_cb(
        _instance: u8,
        _report_id: u8,
        _report_type: u8,
        _buffer: *mut u8,
        _reqlen: u16,
    ) -> u16 {
        0
    }

    #[unsafe(no_mangle)]
    extern "C" fn tud_hid_set_report_cb(
        _instance: u8,
        _report_id: u8,
        _report_type: u8,
        _buffer: *const u8,
        _bufsize: u16,
    ) {
    }

    #[unsafe(no_mangle)]
    extern "C" fn tud_hid_descriptor_report_cb(instance: u8) -> *const u8 {
        if instance == 1 {
            KEYBOARD_REPORT_DESCRIPTOR.as_ptr()
        } else {
            MOUSE_REPORT_DESCRIPTOR.as_ptr()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use matchbox_vm::vm::VM;

    fn vm_with_hid_bifs() -> VM {
        VM::new_with_bifs(register_bifs(), std::collections::HashMap::new())
    }

    fn invoke_bif(vm: &mut VM, name: &str, args: &[BxValue]) -> Result<BxValue, String> {
        let bifs = register_bifs();
        let func = bifs.get(name).expect("BIF not registered");
        func(vm, args)
    }

    // ── esp32USBHidReady ──────────────────────────────────────────

    #[test]
    fn hid_ready_returns_false_before_registration() {
        let mut vm = vm_with_hid_bifs();
        // Reset state to simulate pre-registration
        USB_HID_READY.store(false, Ordering::Release);

        let result = invoke_bif(&mut vm, "esp32USBHidReady", &[]).unwrap();
        assert!(
            !result.as_bool(),
            "HID should not be ready before registerAsHID is called"
        );
    }

    #[test]
    fn hid_ready_rejects_extra_args() {
        let mut vm = vm_with_hid_bifs();
        let result = invoke_bif(&mut vm, "esp32USBHidReady", &[BxValue::new_number(1.0)]);
        assert!(
            result.is_err(),
            "esp32USBHidReady should reject extra arguments"
        );
    }

    #[test]
    fn hid_ready_is_case_insensitive() {
        let mut vm = vm_with_hid_bifs();
        USB_HID_READY.store(false, Ordering::Release);

        // All registered aliases should exist and return bool
        for alias in &["esp32USBHidReady", "esp32UsbHidReady", "esp32usbhidready"] {
            let result = invoke_bif(&mut vm, alias, &[]).unwrap();
            assert!(!result.as_bool(), "alias {alias} should return false");
        }
    }

    // ── Mouse BIF validation ─────────────────────────────────────

    #[test]
    fn mouse_move_rejects_wrong_arg_count() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(&mut vm, "esp32USBMouseMove", &[]).unwrap_err();
        assert!(err.contains("2 arguments"), "should reject 0 args: {err}");

        let err =
            invoke_bif(&mut vm, "esp32USBMouseMove", &[BxValue::new_number(1.0)]).unwrap_err();
        assert!(err.contains("2 arguments"), "should reject 1 arg: {err}");

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseMove",
            &[
                BxValue::new_number(1.0),
                BxValue::new_number(2.0),
                BxValue::new_number(3.0),
            ],
        )
        .unwrap_err();
        assert!(err.contains("2 arguments"), "should reject 3 args: {err}");
    }

    #[test]
    fn mouse_move_rejects_non_finite_values() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseMove",
            &[BxValue::new_number(f64::NAN), BxValue::new_number(0.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("must be a finite number"),
            "should reject NaN: {err}"
        );
    }

    #[test]
    fn mouse_move_rejects_out_of_range_values() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseMove",
            &[BxValue::new_number(128.0), BxValue::new_number(0.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("between -128 and 127"),
            "should reject x=128: {err}"
        );

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseMove",
            &[BxValue::new_number(0.0), BxValue::new_number(-129.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("between -128 and 127"),
            "should reject y=-129: {err}"
        );
    }

    // ── RegisterAsHID validation ─────────────────────────────────

    #[test]
    fn register_as_hid_rejects_too_many_args() {
        let mut vm = vm_with_hid_bifs();
        // Reset so we don't hit "already registered" path
        USB_HID_READY.store(false, Ordering::Release);

        let err = invoke_bif(
            &mut vm,
            "esp32RegisterAsHID",
            &[BxValue::new_number(1.0), BxValue::new_number(2.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("0 or 1 arguments"),
            "should reject 2 args: {err}"
        );
    }

    // ── Mouse button BIFs ───────────────────────────────────────

    #[test]
    fn mouse_button_rejects_wrong_arg_count() {
        let mut vm = vm_with_hid_bifs();

        for bif in &["esp32USBMouseClick", "esp32USBMouseDown", "esp32USBMouseUp"] {
            let err = invoke_bif(&mut vm, bif, &[]).unwrap_err();
            assert!(
                err.contains("1 argument"),
                "{bif} should reject 0 args: {err}"
            );

            let err = invoke_bif(
                &mut vm,
                bif,
                &[BxValue::new_number(1.0), BxValue::new_number(2.0)],
            )
            .unwrap_err();
            assert!(
                err.contains("1 argument"),
                "{bif} should reject 2 args: {err}"
            );
        }
    }

    #[test]
    fn mouse_button_rejects_invalid_button_numbers() {
        let mut vm = vm_with_hid_bifs();

        for bif in &["esp32USBMouseClick", "esp32USBMouseDown", "esp32USBMouseUp"] {
            let err = invoke_bif(&mut vm, bif, &[BxValue::new_number(-1.0)]).unwrap_err();
            assert!(
                err.contains("button must be"),
                "{bif} should reject -1: {err}"
            );

            let err = invoke_bif(&mut vm, bif, &[BxValue::new_number(8.0)]).unwrap_err();
            assert!(
                err.contains("button must be"),
                "{bif} should reject 8: {err}"
            );

            let err = invoke_bif(&mut vm, bif, &[BxValue::new_number(f64::NAN)]).unwrap_err();
            assert!(
                err.contains("button must be"),
                "{bif} should reject NaN: {err}"
            );
        }
    }

    #[test]
    fn mouse_click_has_case_insensitive_aliases() {
        let mut vm = vm_with_hid_bifs();

        for alias in &[
            "esp32USBMouseClick",
            "esp32UsbMouseClick",
            "esp32usbmouseclick",
        ] {
            // Should fail with "not ready" not "BIF not registered"
            let result = invoke_bif(&mut vm, alias, &[BxValue::new_number(1.0)]);
            assert!(result.is_err(), "alias {alias} should be registered");
        }
    }

    // ── Mouse scroll BIFs ───────────────────────────────────────

    #[test]
    fn mouse_scroll_rejects_wrong_arg_count() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(&mut vm, "esp32USBMouseScroll", &[]).unwrap_err();
        assert!(err.contains("2 arguments"), "should reject 0 args: {err}");

        let err =
            invoke_bif(&mut vm, "esp32USBMouseScroll", &[BxValue::new_number(1.0)]).unwrap_err();
        assert!(err.contains("2 arguments"), "should reject 1 arg: {err}");
    }

    #[test]
    fn mouse_scroll_rejects_non_finite_values() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseScroll",
            &[BxValue::new_number(f64::INFINITY), BxValue::new_number(0.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("must be a finite number"),
            "should reject Infinity: {err}"
        );
    }

    #[test]
    fn mouse_scroll_rejects_out_of_range_values() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBMouseScroll",
            &[BxValue::new_number(128.0), BxValue::new_number(0.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("between -128 and 127"),
            "should reject dx=128: {err}"
        );
    }

    #[test]
    fn mouse_scroll_has_case_insensitive_aliases() {
        let mut vm = vm_with_hid_bifs();

        for alias in &[
            "esp32USBMouseScroll",
            "esp32UsbMouseScroll",
            "esp32usbmousescroll",
        ] {
            // Should fail with "not ready" not "BIF not registered"
            let result = invoke_bif(
                &mut vm,
                alias,
                &[BxValue::new_number(1.0), BxValue::new_number(2.0)],
            );
            assert!(result.is_err(), "alias {alias} should be registered");
        }
    }

    // ── Keyboard BIFs ───────────────────────────────────────────

    #[test]
    fn keyboard_press_rejects_wrong_arg_count() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(&mut vm, "esp32USBKeyboardPress", &[]).unwrap_err();
        assert!(err.contains("1 argument"), "should reject 0 args: {err}");

        let err = invoke_bif(
            &mut vm,
            "esp32USBKeyboardPress",
            &[BxValue::new_number(1.0), BxValue::new_number(2.0)],
        )
        .unwrap_err();
        assert!(err.contains("1 argument"), "should reject 2 args: {err}");
    }

    #[test]
    fn keyboard_press_rejects_invalid_keycode() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBKeyboardPress",
            &[BxValue::new_number(-1.0)],
        )
        .unwrap_err();
        assert!(err.contains("between 0 and 255"), "should reject -1: {err}");

        let err = invoke_bif(
            &mut vm,
            "esp32USBKeyboardPress",
            &[BxValue::new_number(256.0)],
        )
        .unwrap_err();
        assert!(
            err.contains("between 0 and 255"),
            "should reject 256: {err}"
        );
    }

    #[test]
    fn keyboard_release_rejects_wrong_arg_count() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(&mut vm, "esp32USBKeyboardRelease", &[]).unwrap_err();
        assert!(err.contains("1 argument"), "should reject 0 args: {err}");
    }

    #[test]
    fn keyboard_release_all_accepts_zero_args() {
        let mut vm = vm_with_hid_bifs();

        // Should fail with "not ready" not "wrong args"
        let result = invoke_bif(&mut vm, "esp32USBKeyboardReleaseAll", &[]);
        assert!(result.is_err(), "should be registered");
        assert!(
            !result.unwrap_err().contains("argument"),
            "error should not be about arg count"
        );
    }

    #[test]
    fn keyboard_release_all_rejects_extra_args() {
        let mut vm = vm_with_hid_bifs();

        let err = invoke_bif(
            &mut vm,
            "esp32USBKeyboardReleaseAll",
            &[BxValue::new_number(1.0)],
        )
        .unwrap_err();
        assert!(err.contains("0 arguments"), "should reject 1 arg: {err}");
    }

    #[test]
    fn keyboard_bifs_have_case_insensitive_aliases() {
        let mut vm = vm_with_hid_bifs();

        for (bif, arg) in &[
            ("esp32USBKeyboardPress", Some(BxValue::new_number(4.0))),
            ("esp32UsbKeyboardPress", Some(BxValue::new_number(4.0))),
            ("esp32usbkeyboardpress", Some(BxValue::new_number(4.0))),
            ("esp32USBKeyboardRelease", Some(BxValue::new_number(4.0))),
            ("esp32UsbKeyboardRelease", Some(BxValue::new_number(4.0))),
            ("esp32usbkeyboardrelease", Some(BxValue::new_number(4.0))),
            ("esp32USBKeyboardReleaseAll", None),
            ("esp32UsbKeyboardReleaseAll", None),
            ("esp32usbkeyboardreleaseall", None),
        ] {
            let args: &[BxValue] = if let Some(a) = arg {
                std::slice::from_ref(a)
            } else {
                &[]
            };
            let result = invoke_bif(&mut vm, bif, args);
            assert!(result.is_err(), "alias {bif} should be registered");
        }
    }
}
