---
name: build-esp32-app
description: >-
  Author MatchBox ESP32 applications using BoxLang. Covers Application.bx
  lifecycle, application.esp32 configuration, web routes and templates,
  runAsync for background work, and memory-conscious ESP32 patterns.
applyTo: "**/*.bxs **/*.bxm **/matchbox-esp32-runner/**"
---

# MatchBox ESP32 App Authoring Agent Skill

## Overview

Build ESP32 MatchBox applications that run BoxLang scripts on microcontrollers. Your app uses a `component` definition in `Application.bx` as its lifecycle entrypoint, declarative `application.esp32` configuration for platform services, file-routed web handlers (`.bxs` for logic, `.bxm` templates for HTML), and `runAsync` for safe background work.

## App Structure

```
my-esp32-app/
├── Application.bx       # App lifecycle & configuration
├── index.bxm            # GET /  – landing page template
├── status.bxm           # GET /status – device status template
├── action.post.bxs      # POST /action – form handler
└── printer/
    └── [id].bxm         # GET /printer/:id – route params
```

## Application.bx – Lifecycle & Configuration

`Application.bx` is a BoxLang component that defines your app's lifecycle. It is automatically loaded and its `onApplicationStart()` method is called before the web server starts.

```bx
component {
    function onApplicationStart() {
        // application.esp32 is pre-populated with build-time defaults.
        // Override any values you need.

        // Wi‑Fi configuration
        application.esp32.wifi.hostname = "my-device";

        // Web server port
        application.esp32.web.port = 80;

        // Use the application scope for shared state
        application.counter = 0;
        application.startTime = now();
    }
}
```

### application.esp32 Configuration Shape

| Path | Type | Description | Default |
|------|------|-------------|---------|
| `application.esp32.wifi.ssid` | string | Wi‑Fi SSID | From `MATCHBOX_ESP32_WIFI_SSID` env |
| `application.esp32.wifi.password` | string | Wi‑Fi password | From `MATCHBOX_ESP32_WIFI_PASSWORD` env |
| `application.esp32.wifi.hostname` | string | Device hostname | `"matchbox-esp32"` |
| `application.esp32.web.port` | number | HTTP server port | `80` |

All values are mutable during `onApplicationStart()`. After startup, the config is frozen for the runtime.

### The application Scope

The `application` scope struct persists for the lifetime of the device. Use it for:

- Shared counters
- Feature toggles
- Cached data (keep it small on ESP32!)
- References to platform services

```bx
function onApplicationStart() {
    application.esp32.wifi.hostname = "label-printer";
    application.jobQueue = [];
    application.lastRestart = now();
}
```

## Web Routes

MatchBox ESP32 uses **file-routed handlers**:

### Template Routes (`.bxm`) – HTML Responses

Templates render HTML and have access to the `application` scope:

```html
<!-- index.bxm -->
<!doctype html>
<html>
<body>
  <h1><%= application.esp32.wifi.hostname %></h1>
  <p>Counter: <%= application.counter %></p>
</body>
</html>
```

`<%= expression %>` evaluates a BoxLang expression and embeds the result.

### Script Routes (`.bxs`) – Logic Handlers

Scripts handle form submissions, JSON APIs, and interactive endpoints:

```bx
// action.post.bxs – POST /action
function handlePost() {
    // form scope is available for POST requests
    var name = form.name ?: "unknown";

    // Update application state
    application.counter = (application.counter ?: 0) + 1;

    // Return a struct for JSON responses
    return {
        "ok": true,
        "name": name,
        "counter": application.counter
    };
}
handlePost();
```

### Dynamic Route Parameters

Use `[name]` in filenames for route parameters:

```
printer/[id].bxm   →  GET /printer/:id
```

Route parameters are accessible via the `url` scope:

```html
<!-- printer/[id].bxm -->
<p>Printer ID: <%= url.id %></p>
```

### Available Request Scopes

| Scope | Contents |
|-------|----------|
| `url` | Query string parameters + route params |
| `form` | Form fields (POST body) |
| `request` | Request metadata |
| `cgi` | CGI-style variables (`request_method`, `path_info`, etc.) |
| `application` | Shared app state |

## Background Work with runAsync

Use `runAsync` for background tasks that should not block request handlers:

```bx
function onApplicationStart() {
    application.counter = 0;

    // Background task: increment counter every 5 seconds
    runAsync(function() {
        while (true) {
            sleep(5000);
            application.counter = (application.counter ?: 0) + 1;
        }
    });
}
```

## Memory-Conscious ESP32 Patterns

The ESP32 has limited RAM (~520KB). Follow these patterns:

### ✅ Do

- Keep responses small (under 10KB)
- Use bounded loops with `sleep()` in `runAsync`
- Clear temporary variables after use
- Use `application` scope for shared persisted state
- Return structs/arrays directly for JSON responses

### ❌ Don't

- Accumulate unbounded arrays in `application` scope
- Load large files or binary blobs into BoxLang values
- Create deeply nested structs
- Block web routes waiting on background fibers
- Store request data in `application` scope

### Loop Pattern for runAsync

```bx
function onApplicationStart() {
    runAsync(function() {
        // Always have an exit condition or sleep
        var iterations = 0;
        while (iterations < 1000) {
            doWork();
            sleep(1000);  // yield to the scheduler
            iterations++;
        }
    });
}
```

## Minimal Web-Controlled Device Example

This example creates a simple toggle that can be controlled from a web page:

```bx
// Application.bx
component {
    function onApplicationStart() {
        application.esp32.wifi.hostname = "toggle-demo";
        application.isOn = false;
    }
}
```

```html
<!-- index.bxm -->
<!doctype html>
<html>
<head>
  <title>Toggle Demo</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    body { font-family: system-ui; padding: 1rem; text-align: center; }
    button { font-size: 2rem; padding: 1rem 2rem; border-radius: 8px; }
    .on { background: #4caf50; color: white; }
    .off { background: #e0e0e0; }
  </style>
</head>
<body>
  <h1><%= application.esp32.wifi.hostname %></h1>
  <form method="post" action="/toggle">
    <!--# if application.isOn #-->
    <button class="on">Turn OFF</button>
    <!--# else #-->
    <button class="off">Turn ON</button>
    <!--# endif #-->
  </form>
</body>
</html>
```

```bx
// toggle.post.bxs
function toggle() {
    application.isOn = !application.isOn;
    return { "isOn": application.isOn };
}
toggle();
```

## ESP32-Specific BIFs (Reference)

### WiFi
- `esp32WifiStation(ssid, password, [hostname])` – Connect to WiFi
- `esp32WifiAccessPoint(ssid, [password], [channel])` – Create access point
- `esp32WifiStatus()` – Get current WiFi status

### HID (USB)
- `esp32USBHidReady()` – Check if HID is ready
- `esp32USBMouseMove(x, y)` – Move mouse
- `esp32USBMouseClick(button)` – Click mouse button
- `esp32USBKeyboardPress(key)` – Press a key
- `esp32USBKeyboardType(text)` – Type text

### Camera
- `esp32CameraCapture()` – Capture a photo
- `esp32CameraCaptureBitmap()` – Capture as bitmap
- `esp32PhotoInfo()` – Get photo info
- `esp32PhotoUrl()` – Get photo URL

### Bluetooth
- `esp32BluetoothConnect(address)` – Connect to device
- `esp32BluetoothDisconnect()` – Disconnect
- `esp32BluetoothWrite(data)` – Send data
- `esp32PrintTspl(zpl)` – Print TSPL command

### Device Diagnostics
- `esp32ResetReason()` – Get last reset reason
- `esp32HeapInfo()` – Get heap information
- `esp32UptimeMillis()` – Get uptime in ms
- `esp32ChipInfo()` – Get chip information
- `esp32Restart()` – Restart the device

### Storage (NVS)
- `esp32NvsGet(key, [defaultValue])` – Get persisted value
- `esp32NvsSet(key, value)` – Persist value
- `esp32NvsDelete(key)` – Delete persisted value
- `esp32NvsKeys([prefix])` – List persisted keys

### GPIO
- `esp32PinMode(pin, mode)` – Set pin mode (INPUT/OUTPUT)
- `esp32DigitalRead(pin)` – Read digital input
- `esp32DigitalWrite(pin, value)` – Write digital output

## Verification Checklist

After building an ESP32 app, verify:

- [ ] `Application.bx` exists with `onApplicationStart()`
- [ ] `application.esp32` config values are set
- [ ] Index page loads on `GET /`
- [ ] All routes respond with correct HTTP methods
- [ ] `GET /__matchbox/status` shows expected routes and diagnostics
- [ ] Background `runAsync` tasks start and don't crash
- [ ] Monitor output shows `[matchbox]` boot messages without errors
- [ ] Fast deploy (`--flash`) works after initial full flash

## Related Skills

- **build-flash-esp32** — Building and flashing ESP32 apps
- **coredump-esp32** — Reading ESP32 coredumps
- **debug-esp32-app** — Debugging ESP32 applications
