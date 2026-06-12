use crate::profile::StrictProfile;
use anyhow::Result;
use core::convert::TryInto;
use embedded_svc::wifi::{
    AccessPointConfiguration, AuthMethod, ClientConfiguration, Configuration as WifiConfiguration,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::handle::RawHandle;
use esp_idf_svc::ipv4::{Configuration as IpConfiguration, Mask, RouterConfiguration, Subnet};
use esp_idf_svc::netif::{EspNetif, NetifConfiguration, NetifStack};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi, WifiDriver};
use matchbox_vm::types::{BxNativeFunction, BxVM, BxValue};
use std::collections::HashMap;
use std::ffi::CString;
use std::net::Ipv4Addr;
use std::sync::{Mutex, OnceLock};

pub struct WifiState {
    pub ip: String,
    pub mode: &'static str,
    _wifi: BlockingWifi<EspWifi<'static>>,
}

fn active_wifi() -> &'static Mutex<Option<WifiState>> {
    static ACTIVE_WIFI: OnceLock<Mutex<Option<WifiState>>> = OnceLock::new();
    ACTIVE_WIFI.get_or_init(|| Mutex::new(None))
}

pub fn active_ip() -> Option<String> {
    active_wifi()
        .lock()
        .ok()
        .and_then(|state| state.as_ref().map(|wifi| wifi.ip.clone()))
}

pub fn register_bifs() -> HashMap<String, BxNativeFunction> {
    let mut map = HashMap::new();
    map.insert(
        "esp32wifistation".to_string(),
        esp32_wifi_station as BxNativeFunction,
    );
    map.insert(
        "esp32WifiStation".to_string(),
        esp32_wifi_station as BxNativeFunction,
    );
    map.insert(
        "esp32wifiaccesspoint".to_string(),
        esp32_wifi_access_point as BxNativeFunction,
    );
    map.insert(
        "esp32WifiAccessPoint".to_string(),
        esp32_wifi_access_point as BxNativeFunction,
    );
    map.insert(
        "esp32wifistatus".to_string(),
        esp32_wifi_status as BxNativeFunction,
    );
    map.insert(
        "esp32WifiStatus".to_string(),
        esp32_wifi_status as BxNativeFunction,
    );
    map
}

fn esp32_wifi_station(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.len() < 2 || args.len() > 3 {
        return Err("esp32WifiStation requires ssid, password, and optional hostname".to_string());
    }
    let ssid = vm.to_string(args[0]);
    let password = vm.to_string(args[1]);
    let hostname = if args.len() > 2 && !args[2].is_null() {
        vm.to_string(args[2])
    } else {
        "matchbox-esp32".to_string()
    };

    let state = connect_station(&ssid, &password, &hostname).map_err(|error| error.to_string())?;
    let ip = state.ip.clone();
    *active_wifi()
        .lock()
        .map_err(|_| "wifi state lock poisoned".to_string())? = Some(state);
    Ok(status_value(vm, "station", &ip))
}

fn esp32_wifi_access_point(vm: &mut dyn BxVM, args: &[BxValue]) -> Result<BxValue, String> {
    if args.is_empty() || args.len() > 3 {
        return Err(
            "esp32WifiAccessPoint requires ssid, optional password, and optional channel"
                .to_string(),
        );
    }
    let ssid = vm.to_string(args[0]);
    let password = if args.len() > 1 && !args[1].is_null() {
        Some(vm.to_string(args[1]))
    } else {
        None
    };
    let channel = if args.len() > 2 && !args[2].is_null() {
        args[2].as_number() as u8
    } else {
        1
    };

    let state = connect_access_point(&ssid, password.as_deref(), channel)
        .map_err(|error| error.to_string())?;
    let ip = state.ip.clone();
    *active_wifi()
        .lock()
        .map_err(|_| "wifi state lock poisoned".to_string())? = Some(state);
    Ok(status_value(vm, "accessPoint", &ip))
}

fn esp32_wifi_status(vm: &mut dyn BxVM, _args: &[BxValue]) -> Result<BxValue, String> {
    let state = active_wifi()
        .lock()
        .map_err(|_| "wifi state lock poisoned".to_string())?;
    match state.as_ref() {
        Some(state) => Ok(status_value(vm, state.mode, &state.ip)),
        None => Ok(status_value(vm, "off", "")),
    }
}

fn status_value(vm: &mut dyn BxVM, mode: &str, ip: &str) -> BxValue {
    let result = vm.struct_new();
    let mode = BxValue::new_ptr(vm.string_new(mode.to_string()));
    let ip = BxValue::new_ptr(vm.string_new(ip.to_string()));
    vm.struct_set(result, "mode", mode);
    vm.struct_set(result, "ip", ip);
    BxValue::new_ptr(result)
}

pub fn connect(profile: &StrictProfile) -> Result<WifiState> {
    connect_station(
        profile.wifi_ssid,
        profile.wifi_password,
        profile.wifi_hostname,
    )
}

pub fn connect_station(ssid: &str, password: &str, hostname: &str) -> Result<WifiState> {
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    let hostname_c = CString::new(hostname)?;
    esp_idf_sys::esp!(unsafe {
        esp_idf_sys::esp_netif_set_hostname(
            wifi.wifi().sta_netif().handle(),
            hostname_c.as_ptr() as *const _,
        )
    })?;

    let configuration = WifiConfiguration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&configuration)?;
    wifi.start()?;
    println!("[matchbox] Wi-Fi station started for SSID '{}'", ssid);

    wifi.connect()?;
    println!("[matchbox] Wi-Fi connected");

    wifi.wait_netif_up()?;
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    let ip = ip_info.ip.to_string();
    println!("[matchbox] Wi-Fi ready. hostname='{}' ip={}", hostname, ip);

    Ok(WifiState {
        ip,
        mode: "station",
        _wifi: wifi,
    })
}

pub fn connect_access_point(ssid: &str, password: Option<&str>, channel: u8) -> Result<WifiState> {
    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let ap_ip = Ipv4Addr::new(192, 168, 4, 1);
    let wifi = WifiDriver::new(peripherals.modem, sys_loop.clone(), Some(nvs))?;
    let wifi = EspWifi::wrap_all(
        wifi,
        EspNetif::new(NetifStack::Sta)?,
        EspNetif::new_with_conf(&NetifConfiguration {
            ip_configuration: Some(IpConfiguration::Router(RouterConfiguration {
                subnet: Subnet {
                    gateway: ap_ip,
                    mask: Mask(24),
                },
                dhcp_enabled: true,
                dns: Some(ap_ip),
                secondary_dns: None,
            })),
            ..NetifConfiguration::wifi_default_router()
        })?,
    )?;
    let mut wifi = BlockingWifi::wrap(wifi, sys_loop)?;

    let auth_method = if password.filter(|value| !value.is_empty()).is_some() {
        AuthMethod::WPA2Personal
    } else {
        AuthMethod::None
    };

    let configuration = WifiConfiguration::AccessPoint(AccessPointConfiguration {
        ssid: ssid.try_into().unwrap(),
        auth_method,
        password: password.unwrap_or("").try_into().unwrap(),
        channel,
        max_connections: 4,
        ..Default::default()
    });

    wifi.set_configuration(&configuration)?;
    wifi.start()?;
    let ip = wifi.wifi().ap_netif().get_ip_info()?.ip.to_string();
    println!(
        "[matchbox] Wi-Fi access point ready. ssid='{}' ip={}",
        ssid, ip
    );

    Ok(WifiState {
        ip,
        mode: "accessPoint",
        _wifi: wifi,
    })
}
