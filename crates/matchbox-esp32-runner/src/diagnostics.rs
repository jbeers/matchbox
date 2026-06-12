use esp_idf_svc::nvs::{EspCustomNvsPartition, EspNvs, NvsCustom};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

const NAMESPACE: &str = "mbxdiag";
const KEY_BOOT_COUNT: &str = "boot";
const KEY_EVENT_COUNT: &str = "events";
const KEY_LAST_EVENT: &str = "last";
const KEY_LAST_RESET: &str = "reset";

static EVENT_COUNT: AtomicU32 = AtomicU32::new(0);
static BOOT_COUNT: AtomicU32 = AtomicU32::new(0);
static MIN_FREE_HEAP: AtomicU32 = AtomicU32::new(u32::MAX);
static LAST_EVENT: OnceLock<Mutex<String>> = OnceLock::new();
static NVS_STATUS: OnceLock<Mutex<String>> = OnceLock::new();

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSnapshot {
    pub boot_count: u32,
    pub event_count: u32,
    pub reset_reason: String,
    pub last_reset_reason: String,
    pub last_event: String,
    pub nvs_available: bool,
    pub nvs_status: String,
    pub heap: HealthHeapSnapshot,
    pub hid: crate::hid::HidSnapshot,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthHeapSnapshot {
    pub free: u32,
    pub min_free: u32,
    pub largest_internal_8bit_block: u32,
    pub free_internal_8bit: u32,
}

fn last_event() -> &'static Mutex<String> {
    LAST_EVENT.get_or_init(|| Mutex::new(String::new()))
}

fn nvs_status() -> &'static Mutex<String> {
    NVS_STATUS.get_or_init(|| Mutex::new(String::new()))
}

pub fn init() {
    let reset_reason = reset_reason();
    let mut boot_count = 1;
    let mut event_count = 0;
    let mut previous_event = String::new();
    let mut nvs_available = false;
    let mut status = String::from("unavailable");

    if let Ok(mut nvs) = open_nvs() {
        nvs_available = true;
        status = String::from("ok");
        boot_count = nvs
            .get_u32(KEY_BOOT_COUNT)
            .ok()
            .flatten()
            .unwrap_or(0)
            .saturating_add(1);
        event_count = nvs.get_u32(KEY_EVENT_COUNT).ok().flatten().unwrap_or(0);
        previous_event = read_str(&nvs, KEY_LAST_EVENT);
        let _ = nvs.set_u32(KEY_BOOT_COUNT, boot_count);
        let _ = nvs.set_str(KEY_LAST_RESET, &reset_reason);
    } else if let Err(error) = open_nvs() {
        status = format!("{}", error);
    }

    BOOT_COUNT.store(boot_count, Ordering::Release);
    EVENT_COUNT.store(event_count, Ordering::Release);
    if let Ok(mut current) = last_event().lock() {
        *current = previous_event.clone();
    }
    if let Ok(mut current) = nvs_status().lock() {
        *current = status.clone();
    }

    println!(
        "[matchbox] diagnostics boot={} reset={} nvs={} nvs_status='{}' last_event='{}'",
        boot_count, reset_reason, nvs_available, status, previous_event
    );
}

pub fn record_event(event: impl AsRef<str>) {
    let event = event.as_ref();
    let event = if event.len() > 120 {
        &event[..120]
    } else {
        event
    };
    let count = EVENT_COUNT.fetch_add(1, Ordering::AcqRel).saturating_add(1);

    if let Ok(mut current) = last_event().lock() {
        current.clear();
        current.push_str(event);
    }
}

pub fn snapshot() -> DiagnosticsSnapshot {
    let mut boot_count = BOOT_COUNT.load(Ordering::Acquire);
    let mut event_count = EVENT_COUNT.load(Ordering::Acquire);
    let mut last_reset_reason = String::new();
    let mut nvs_available = false;
    let mut nvs_status_value = nvs_status()
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();

    let mut last_event_value = last_event()
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();

    if let Ok(nvs) = open_nvs() {
        nvs_available = true;
        nvs_status_value = String::from("ok");
        boot_count = nvs
            .get_u32(KEY_BOOT_COUNT)
            .ok()
            .flatten()
            .unwrap_or(boot_count);
        event_count = nvs
            .get_u32(KEY_EVENT_COUNT)
            .ok()
            .flatten()
            .unwrap_or(event_count);
        last_reset_reason = read_str(&nvs, KEY_LAST_RESET);
        let stored_last_event = read_str(&nvs, KEY_LAST_EVENT);
        if !stored_last_event.is_empty() {
            last_event_value = stored_last_event;
        }
    } else if let Err(error) = open_nvs() {
        nvs_status_value = format!("{}", error);
    }

    DiagnosticsSnapshot {
        boot_count,
        event_count,
        reset_reason: reset_reason(),
        last_reset_reason,
        last_event: last_event_value,
        nvs_available,
        nvs_status: nvs_status_value,
        heap: heap_snapshot(),
        hid: crate::hid::snapshot(),
    }
}

pub fn start_health_sampler() {
    static STARTED: OnceLock<()> = OnceLock::new();
    STARTED.get_or_init(|| {
        let builder = thread::Builder::new()
            .name("matchbox-health".to_string())
            .stack_size(4096);
        match builder.spawn(|| loop {
            let heap = heap_snapshot();
            let hid = crate::hid::snapshot();
            let event_count = EVENT_COUNT.load(Ordering::Acquire);
            let last_event = last_event()
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default();
            println!(
                "[matchbox] health free={} min_free={} largest={} hid_ready={} hid_queued={} hid_sent={} hid_errors={} events={} last='{}'",
                heap.free,
                heap.min_free,
                heap.largest_internal_8bit_block,
                hid.ready,
                hid.queued,
                hid.sent,
                hid.errors,
                event_count,
                last_event
            );
            thread::sleep(Duration::from_secs(5));
        }) {
            Ok(_) => {}
            Err(error) => println!("[matchbox] health sampler failed to start: {}", error),
        }
    });
}

fn heap_snapshot() -> HealthHeapSnapshot {
    unsafe {
        let free = esp_idf_sys::esp_get_free_heap_size() as u32;
        let mut current_min = MIN_FREE_HEAP.load(Ordering::Acquire);
        while free < current_min {
            match MIN_FREE_HEAP.compare_exchange(
                current_min,
                free,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(actual) => current_min = actual,
            }
        }
        let min_free = MIN_FREE_HEAP.load(Ordering::Acquire);
        HealthHeapSnapshot {
            free,
            min_free,
            largest_internal_8bit_block: esp_idf_sys::heap_caps_get_largest_free_block(
                esp_idf_sys::MALLOC_CAP_INTERNAL | esp_idf_sys::MALLOC_CAP_8BIT,
            ) as u32,
            free_internal_8bit: esp_idf_sys::heap_caps_get_free_size(
                esp_idf_sys::MALLOC_CAP_INTERNAL | esp_idf_sys::MALLOC_CAP_8BIT,
            ) as u32,
        }
    }
}

fn open_nvs() -> Result<EspNvs<NvsCustom>, esp_idf_sys::EspError> {
    let partition = EspCustomNvsPartition::take("nvs")?;
    EspNvs::new(partition, NAMESPACE, true)
}

fn read_str(nvs: &EspNvs<NvsCustom>, key: &str) -> String {
    let mut buffer = [0u8; 160];
    nvs.get_str(key, &mut buffer)
        .ok()
        .flatten()
        .unwrap_or("")
        .to_string()
}

fn reset_reason() -> String {
    let reason = unsafe { esp_idf_sys::esp_reset_reason() };
    let label = match reason {
        esp_idf_sys::esp_reset_reason_t_ESP_RST_UNKNOWN => "ESP_RST_UNKNOWN",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_POWERON => "ESP_RST_POWERON",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_EXT => "ESP_RST_EXT",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_SW => "ESP_RST_SW",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_PANIC => "ESP_RST_PANIC",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_INT_WDT => "ESP_RST_INT_WDT",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_TASK_WDT => "ESP_RST_TASK_WDT",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_WDT => "ESP_RST_WDT",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_DEEPSLEEP => "ESP_RST_DEEPSLEEP",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_BROWNOUT => "ESP_RST_BROWNOUT",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_SDIO => "ESP_RST_SDIO",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_USB => "ESP_RST_USB",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_JTAG => "ESP_RST_JTAG",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_EFUSE => "ESP_RST_EFUSE",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_PWR_GLITCH => "ESP_RST_PWR_GLITCH",
        esp_idf_sys::esp_reset_reason_t_ESP_RST_CPU_LOCKUP => "ESP_RST_CPU_LOCKUP",
        _ => "ESP_RST_UNRECOGNIZED",
    };
    format!("{} ({})", label, reason)
}
