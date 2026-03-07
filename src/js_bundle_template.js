/* STANDALONE MATCHBOX JS BUNDLE TEMPLATE (WASI BASED) */

let wasm;
const cachedTextDecoder = new TextDecoder('utf-8');
const cachedTextEncoder = new TextEncoder();

// Minimal WASI shim
const wasiShim = {
    fd_write: (fd, iovs, iovs_len, nwritten) => {
        const view = new DataView(wasm.memory.buffer);
        let written = 0;
        for (let i = 0; i < iovs_len; i++) {
            const ptr = view.getUint32(iovs + i * 8, true);
            const len = view.getUint32(iovs + i * 8 + 4, true);
            const str = cachedTextDecoder.decode(new Uint8Array(wasm.memory.buffer).subarray(ptr, ptr + len));
            if (fd === 1 || fd === 2) console.log(str);
            written += len;
        }
        view.setUint32(nwritten, written, true);
        return 0;
    },
    random_get: (buf, len) => {
        crypto.getRandomValues(new Uint8Array(wasm.memory.buffer).subarray(buf, buf + len));
        return 0;
    },
    proc_exit: (code) => { throw new Error("Process exited with code " + code); },
    environ_sizes_get: (count, buf_size) => {
        const view = new DataView(wasm.memory.buffer);
        view.setUint32(count, 0, true);
        view.setUint32(buf_size, 0, true);
        return 0;
    },
    environ_get: (environ, environ_buf) => 0,
    args_sizes_get: (count, buf_size) => {
        const view = new DataView(wasm.memory.buffer);
        view.setUint32(count, 0, true);
        view.setUint32(buf_size, 0, true);
        return 0;
    },
    args_get: (args, args_buf) => 0,
    clock_time_get: (id, precision, time) => {
        const view = new DataView(wasm.memory.buffer);
        const now = BigInt(Date.now()) * 1000000n;
        view.setBigUint64(time, now, true);
        return 0;
    },
    poll_oneoff: (in_ptr, out_ptr, nsubscriptions, nevents) => {
        return 0;
    },
    fd_close: (fd) => 0,
    fd_read: (fd, iovs, iovs_len, nread) => 0,
    fd_seek: (fd, offset, whence, newoffset) => 0,
    fd_fdstat_get: (fd, stat) => 0,
    path_open: (fd, dirflags, path, path_len, oflags, fs_rights_base, fs_rights_inheriting, fdflags, opened_fd) => 0,
    path_filestat_get: (fd, flags, path, path_len, stat) => 0,
    path_remove_directory: (fd, path, path_len) => 0,
    path_unlink_file: (fd, path, path_len) => 0
};

async function init(wasmBytes) {
    const imports = {
        wasi_snapshot_preview1: wasiShim
    };

    const { instance } = await WebAssembly.instantiate(wasmBytes, imports);
    wasm = instance.exports;
}

export class BoxLangVM {
    constructor() {}

    load_bytecode(bytes) {
        const len = bytes.length;
        const ptr = wasm.boxlang_alloc(len);
        const mem = new Uint8Array(wasm.memory.buffer);
        mem.set(bytes, ptr);
        const res = wasm.boxlang_load_bytecode(ptr, len);
        if (res !== 0) {
            throw new Error("Failed to load bytecode: " + res);
        }
    }

    async call(name, args) {
        const nameBuf = cachedTextEncoder.encode(name);
        const namePtr = wasm.boxlang_alloc(nameBuf.length);
        new Uint8Array(wasm.memory.buffer).set(nameBuf, namePtr);

        const argsBuf = cachedTextEncoder.encode(JSON.stringify(args));
        const argsPtr = wasm.boxlang_alloc(argsBuf.length);
        new Uint8Array(wasm.memory.buffer).set(argsBuf, argsPtr);

        const resPtr = wasm.boxlang_call(namePtr, nameBuf.length, argsPtr, argsBuf.length);
        const resLen = wasm.boxlang_get_last_result_len();
        
        const resBuf = new Uint8Array(wasm.memory.buffer).subarray(resPtr, resPtr + resLen);
        const resStr = new TextDecoder().decode(resBuf);
        const res = JSON.parse(resStr);
        
        if (res && res.error) {
            throw new Error(res.error);
        }
        return res;
    }
}

/* __REPLACE_ME__ */
