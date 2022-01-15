// #![feature(native_link_modifiers_whole_archive)]

use bindgen::builder;

fn main() -> anyhow::Result<()> {
    let bindings = builder()
        .clang_arg("-Ifluent-bit/lib/cmetrics/include")
        .clang_arg("-Ifluent-bit/lib/c-ares-809d5e84/include")
        .clang_arg("-Ifluent-bit/build/lib/c-ares-809d5e84")
        .clang_arg("-Ifluent-bit/lib/mbedtls-2.27.0/include")
        .clang_arg("-Ifluent-bit/lib/monkey/deps/flb_libco")
        .clang_arg("-Ifluent-bit/lib/msgpack-c/include")
        .clang_arg("-Ifluent-bit/lib/monkey/include")
        .clang_arg("-Ifluent-bit/include")
        .blocklist_item("IPPORT_RESERVED")
        .allowlist_type("flb_input_plugin")
        .allowlist_type("flb_loglevel_helper")
        .allowlist_function("flb_input_set_context")
        .allowlist_function("flb_input_set_collector_time")
        .allowlist_function("flb_input_get_property")
        .allowlist_function("flb_input_chunk_append_raw")
        .no_debug("flb_sds")
        .no_debug("cmt_sds")
        .header("wrapper.h")
        .generate()
        .expect("Failed to generate bindings");

    bindings.write_to_file("src/bindings.rs")?;

    Ok(())
}
