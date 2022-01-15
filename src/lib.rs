use std::os::raw::c_void;

mod bindings {
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]
    #![allow(non_upper_case_globals)]
    include!("bindings.rs");
}

#[derive(Debug)]
struct FLBContext {
    collect_cnt: usize,
}

#[derive(Clone, Debug)]
struct Config {
    interval_sec: i64,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

macro_rules! check_result {
    ($exp: expr) => {
        match $exp {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e.to_string());
                return -1;
            }
        }
    };
}

#[no_mangle]
unsafe extern "C" fn cb_init(
    flb_input_instance: *mut bindings::flb_input_instance,
    flb_config: *mut bindings::flb_config,
    _user: *mut std::os::raw::c_void,
) -> std::os::raw::c_int {
    let ctx = Box::new(FLBContext { collect_cnt: 0 });

    let cfg = check_result!(configure(&ctx, flb_input_instance));

    let ctx_ptr = Box::into_raw(ctx) as *mut c_void;
    bindings::flb_input_set_context(flb_input_instance, ctx_ptr);

    bindings::flb_input_set_collector_time(
        flb_input_instance,
        Some(cb_collect),
        cfg.interval_sec,
        0,
        flb_config,
    );

    0
}

fn get_config_string(
    flb_input_instance: *mut bindings::flb_input_instance,
    key: &str,
    default: &str,
) -> Result<String> {
    let key = std::ffi::CString::new(key)?;
    let s = unsafe { bindings::flb_input_get_property(key.as_ptr(), flb_input_instance) };
    if !s.is_null() {
        let value = unsafe { std::ffi::CStr::from_ptr(s) };
        let value = value.to_str()?.parse()?;
        Ok(value)
    } else {
        Ok(default.to_string())
    }
}

// Se the C code, config_map, for declaring config params
fn configure(
    _ctx: &FLBContext,
    flb_input_instance: *mut bindings::flb_input_instance,
) -> Result<Config> {
    let interval_sec = get_config_string(flb_input_instance, "interval_sec", "10")?.parse()?;

    Ok(Config { interval_sec })
}

/// # Safety
///
/// This function assumes cb_init has been called to initialze ctx
#[no_mangle]
pub unsafe extern "C" fn cb_collect(
    flb_input_instance: *mut bindings::flb_input_instance,
    _flb_config: *mut bindings::flb_config,
    ctx: *mut std::os::raw::c_void,
) -> std::os::raw::c_int {
    let ctx: &mut FLBContext = &mut *(ctx as *mut FLBContext);

    // Increate collect count
    ctx.collect_cnt += 1;

    // Constrult message pack
    // It has the following structure [time-stamp, {a:foo, b:bar, ..}]
    let mut mp = Vec::new();

    // Array with two items
    check_result!(rmp::encode::write_array_len(&mut mp, 2));

    // Encode time

    let now = check_result!(
        std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH)
    );
    const NS: u128 = 1000000000;
    let now_sec = now.as_nanos() / NS;
    let now_nsec = now.as_nanos() - now_sec * NS;

    // The timestamp is written as 8 bytes, 4 bytes seconds and 4 bytes nanos
    check_result!(rmp::encode::write_ext_meta(&mut mp, 8, 0));
    mp.extend_from_slice(&(now_sec as u32).to_be_bytes());
    mp.extend_from_slice(&(now_nsec as u32).to_be_bytes());

    // Let's add a single record to the record map
    check_result!(rmp::encode::write_map_len(&mut mp, 1));

    // The record will be the number of cb_collect calls
    check_result!(rmp::encode::write_str(&mut mp, "collect-calls"));
    check_result!(rmp::encode::write_u64(&mut mp, ctx.collect_cnt as _));

    let res = bindings::flb_input_chunk_append_raw(
        flb_input_instance,
        std::ptr::null(),
        0,
        mp.as_mut_ptr() as *const c_void,
        mp.len() as u64,
    );

    if res != 0 {
        eprintln!("Failed to store chunk");
    }

    res
}

#[no_mangle]
unsafe extern "C" fn cb_exit(
    ctx: *mut std::os::raw::c_void,
    _flb_config: *mut bindings::flb_config,
) -> std::os::raw::c_int {
    // Make sure we drop the CTX allocated in cb_init
    let unboxed_ctx: &mut FLBContext = &mut *(ctx as *mut FLBContext);
    drop(Box::from_raw(unboxed_ctx));
    0
}
