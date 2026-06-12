use matchbox_vm::types::{BxNativeFunction, BxVM, BxValue};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;

static USB_HID_READY: AtomicBool = AtomicBool::new(false);
static HID_QUEUED: AtomicU32 = AtomicU32::new(0);
static HID_SENT: AtomicU32 = AtomicU32::new(0);
static HID_ERRORS: AtomicU32 = AtomicU32::new(0);
static HID_COMMANDS: OnceLock<mpsc::SyncSender<HidCommand>> = OnceLock::new();
const DEFAULT_PRODUCT_NAME: &str = "MatchBox HID";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HidSnapshot {
    pub ready: bool,
    pub queued: u32,
    pub sent: u32,
    pub errors: u32,
}

#[derive(Clone, Copy, Debug)]
struct HidCommand {
    dx: i8,
    dy: i8,
}

pub fn snapshot() -> HidSnapshot {
    HidSnapshot {
        ready: USB_HID_READY.load(Ordering::Acquire),
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
    enqueue_mouse_move(dx, dy)?;

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

fn ensure_usb_hid_ready(product_name: Option<&str>) -> Result<(), String> {
    if USB_HID_READY.load(Ordering::Acquire) {
        if let Some(product_name) = product_name {
            usb_hid::ensure_product_name_matches(product_name)?;
        }
        return Ok(());
    }

    usb_hid::install(product_name.unwrap_or(DEFAULT_PRODUCT_NAME))?;
    start_hid_worker();
    USB_HID_READY.store(true, Ordering::Release);
    println!("[esp32-bif] USB HID ready");
    Ok(())
}

fn enqueue_mouse_move(dx: i8, dy: i8) -> Result<(), String> {
    let tx = HID_COMMANDS
        .get()
        .ok_or_else(|| "USB HID worker is not running".to_string())?;
    crate::diagnostics::record_event(format!("hid queue dx={} dy={}", dx, dy));
    tx.try_send(HidCommand { dx, dy })
        .map_err(|error| format!("USB HID command queue is full or stopped: {}", error))?;
    HID_QUEUED.fetch_add(1, Ordering::AcqRel);
    crate::diagnostics::record_event(format!("hid queued dx={} dy={}", dx, dy));
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
                println!(
                    "[esp32-bif] USB HID worker sending dx={} dy={}",
                    command.dx, command.dy
                );
                crate::diagnostics::record_event(format!(
                    "hid worker send start dx={} dy={}",
                    command.dx, command.dy
                ));
                match usb_hid::send_mouse_report(command.dx, command.dy) {
                    Ok(()) => {
                        HID_SENT.fetch_add(1, Ordering::AcqRel);
                        crate::diagnostics::record_event(format!(
                            "hid worker send complete dx={} dy={}",
                            command.dx, command.dy
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
    use std::sync::{Mutex, OnceLock};

    const TUSB_DESC_DEVICE: u8 = 0x01;
    const TUSB_DESC_CONFIGURATION: u8 = 0x02;
    const TUSB_DESC_INTERFACE: u8 = 0x04;
    const TUSB_DESC_ENDPOINT: u8 = 0x05;
    const TUSB_DESC_HID: u8 = 0x21;

    const TUSB_CLASS_HID: u8 = 0x03;
    const HID_SUBCLASS_BOOT: u8 = 0x01;
    const HID_PROTOCOL_MOUSE: u8 = 0x02;

    const TUSB_XFER_INTERRUPT: u8 = 0x03;
    const TUSB_DIR_IN: u8 = 0x80;
    const HID_ITF: u8 = 0;
    const HID_EP_IN: u8 = TUSB_DIR_IN | 1;
    const HID_EP_SIZE: u16 = 8;
    const HID_POLL_MS: u8 = 10;
    const CONFIG_TOTAL_LEN: u16 = 9 + 9 + 9 + 7;

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
        9,
        TUSB_DESC_CONFIGURATION,
        (CONFIG_TOTAL_LEN & 0xff) as u8,
        (CONFIG_TOTAL_LEN >> 8) as u8,
        1,
        1,
        0,
        0x80,
        50,
        9,
        TUSB_DESC_INTERFACE,
        HID_ITF,
        0,
        1,
        TUSB_CLASS_HID,
        HID_SUBCLASS_BOOT,
        HID_PROTOCOL_MOUSE,
        0,
        9,
        TUSB_DESC_HID,
        0x11,
        0x01,
        0,
        1,
        0x22,
        (MOUSE_REPORT_DESCRIPTOR.len() & 0xff) as u8,
        (MOUSE_REPORT_DESCRIPTOR.len() >> 8) as u8,
        7,
        TUSB_DESC_ENDPOINT,
        HID_EP_IN,
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

    pub fn send_mouse_report(dx: i8, dy: i8) -> Result<(), String> {
        let _guard = REPORT_LOCK
            .lock()
            .map_err(|_| "USB HID report lock poisoned".to_string())?;

        send_report(dx, dy)?;
        unsafe {
            sys::vTaskDelay(20);
        }
        send_report(0, 0)
    }

    fn send_report(dx: i8, dy: i8) -> Result<(), String> {
        for _ in 0..200 {
            if unsafe { tud_mounted() && tud_hid_n_ready(0) } {
                let report = [0u8, dx as u8, dy as u8];
                let sent = unsafe { tud_hid_n_report(0, 0, report.as_ptr(), report.len() as u16) };
                if sent {
                    return Ok(());
                }
            }

            unsafe {
                sys::vTaskDelay(10);
            }
        }

        println!("[esp32-bif] USB HID mouse is not mounted or ready yet");
        Ok(())
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
    extern "C" fn tud_hid_descriptor_report_cb(_instance: u8) -> *const u8 {
        MOUSE_REPORT_DESCRIPTOR.as_ptr()
    }
}
