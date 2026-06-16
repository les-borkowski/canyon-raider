// quad-storage.js — compatible with mq_js_bundle.js
//
// The original quad-storage.js called js_object() and get_js_object() as globals,
// but those are private to the sapp_jsutils IIFE inside mq_js_bundle.js.
// This version intercepts js_create_string to mirror string values by their pool
// ID, then uses allocate_vec_u8 + js_create_string to return new values into the
// pool without needing the private helpers.

miniquad_add_plugin({
    register_plugin: function(importObject) {
        // Mirror of sapp_jsutils string pool: pool_id -> decoded string value.
        // Populated by wrapping js_create_string so we know which key/value each
        // ID refers to when quad_storage_* functions are called.
        var str_mirror = {};

        var orig_create_string = importObject.env.js_create_string;
        var orig_free_object   = importObject.env.js_free_object;

        importObject.env.js_create_string = function(ptr, len) {
            var id = orig_create_string(ptr, len);
            str_mirror[id] = UTF8ToString(ptr, len);
            return id;
        };

        importObject.env.js_free_object = function(id) {
            delete str_mirror[id];
            orig_free_object(id);
        };

        // Write a JS string into WASM memory and register it in sapp_jsutils's
        // pool by calling js_create_string. Returns the pool ID.
        // allocate_vec_u8 intentionally leaks its Vec<u8> (see miniquad source),
        // so the JS side is expected to use-and-forget the pointer.
        function pool_string(value) {
            if (value == null) return -2; // sapp_jsutils null sentinel
            var encoded = new TextEncoder().encode(value);
            var len = encoded.length;
            var ptr = wasm_exports.allocate_vec_u8(len);
            new Uint8Array(wasm_memory.buffer, ptr, len).set(encoded);
            return orig_create_string(ptr, len);
        }

        importObject.env.quad_storage_length = function() {
            return localStorage.length;
        };

        importObject.env.quad_storage_has_key = function(i) {
            return +(localStorage.key(i) != null);
        };

        importObject.env.quad_storage_key = function(i) {
            return pool_string(localStorage.key(i));
        };

        importObject.env.quad_storage_has_value = function(key_id) {
            var key = str_mirror[key_id];
            return +(key != null && localStorage.getItem(key) != null);
        };

        importObject.env.quad_storage_get = function(key_id) {
            var key = str_mirror[key_id];
            if (key == null) return -2;
            return pool_string(localStorage.getItem(key));
        };

        importObject.env.quad_storage_set = function(key_id, value_id) {
            var key   = str_mirror[key_id];
            var value = str_mirror[value_id];
            if (key != null && value != null) {
                localStorage.setItem(key, value);
            }
        };

        importObject.env.quad_storage_remove = function(key_id) {
            var key = str_mirror[key_id];
            if (key != null) localStorage.removeItem(key);
        };

        importObject.env.quad_storage_clear = function() {
            localStorage.clear();
        };
    },

    on_init: function() {},

    // (0 << 24) + (1 << 16) + 0 = 65536, matching quad-storage-sys crate 0.1.0
    version: 65536,
    name: "quad_storage"
});
