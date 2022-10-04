#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::borrow::Borrow;
use std::ffi::{c_void, CStr, CString};
use std::mem::size_of;
use std::os::raw::{c_char, c_int};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use lazy_static::lazy_static;
use log::{error, info, warn};
use std::{env, thread};
use std::time::Duration;
use crate::generated::bindings::{__stdoutp, fflush, option, size_t, spdk_app_opts, spdk_app_parse_args_rvals_SPDK_APP_PARSE_ARGS_SUCCESS, spdk_log_level_SPDK_LOG_INFO, spdk_log_level_SPDK_LOG_WARN, spdk_pci_addr};
use crate::spdk_wrapper::accel_engine_api::AccelEngineApi;
use crate::spdk_wrapper::spdk::spdk_clib::{spdk_app_opts_init, spdk_log_clear_flag, spdk_log_set_flag, spdk_log_set_level, spdk_log_set_print_level, spdk_memzone_dump};

lazy_static!{
    static ref spdkInitialized : AtomicBool = AtomicBool::new(false);
}

pub struct Spdk {
    spdkThread: Option<JoinHandle<()>>,
}

impl Spdk {
    pub fn new() -> Spdk {
        Spdk {
            spdkThread: None
        }
    }

    pub fn Init(&mut self, args: Vec<&'static str>) -> bool {
        if spdkInitialized.load(Ordering::Relaxed) {
            warn!("SPDK is already initialized");
            return true;
        }
        self.spdkThread = Some(thread::spawn(move || {
            // _InitWorker
            let mut opts : spdk_app_opts = spdk_app_opts::default();
            spdk_app_opts_init(&mut opts, size_of::<spdk_app_opts>() as size_t);
            {
                opts.name = CString::new("ibof_nvmf").unwrap().into_raw();
                opts.mem_channel = -1;
                opts.print_level = spdk_log_level_SPDK_LOG_INFO;
                opts.reactor_mask = CString::new("TODO").unwrap().into_raw();
                opts.main_core = 0; // TODO
            }
            let mut empty_option : option = option::default();
            let empty_args = 0 as *mut *mut ::std::os::raw::c_char;
            let getopt_str = 0 as *const ::std::os::raw::c_char;
            let rc = spdk_clib::spdk_app_parse_args(args.len() as i32, empty_args,
                                                    &mut opts, getopt_str, &mut empty_option,
                                                    None, None);

            if rc != spdk_app_parse_args_rvals_SPDK_APP_PARSE_ARGS_SUCCESS
            {
                error!("failed to parse spdk args: {:?}, error: {:?}", args, rc);
                std::process::exit(rc as i32);
            }
            /* Blocks until the application is exiting */
            let rc = spdk_clib::spdk_app_start(&mut opts, Some(Spdk::_AppStartedCallback), 0 as *mut c_void);
            info!("spdk_app_start result = {}", rc);
        }));
        while !spdkInitialized.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1)); // 원래는 1 us 단위였는데, 1 s 도 괜찮다고 봄.
            info!("waiting for spdk initialization...");
        }
        true
    }

    pub fn Finalize() {
        // TODO
    }

    extern "C" fn _AppStartedCallback(ctx: *mut ::std::os::raw::c_void) {
        if let Ok(_v) = env::var("MEMZONE_DUMP") {
            unsafe {
                spdk_memzone_dump(__stdoutp);
                fflush(__stdoutp);
            }
        }
        spdk_log_set_level(spdk_log_level_SPDK_LOG_WARN);
        spdk_log_set_print_level(spdk_log_level_SPDK_LOG_WARN);
        spdk_log_set_flag(CString::new("all").unwrap().into_raw());
        spdk_log_clear_flag(CString::new("reactor").unwrap().into_raw());
        spdk_log_set_flag(CString::new("bdev").unwrap().into_raw());
        spdk_log_set_flag(CString::new("bdev_nvme").unwrap().into_raw());
        spdk_log_set_flag(CString::new("nvme").unwrap().into_raw());
        spdk_log_set_flag(CString::new("bdev_malloc").unwrap().into_raw());
        spdk_log_set_flag(CString::new("bdev_ibof").unwrap().into_raw());

        AccelEngineApi.Initialize();

        spdkInitialized.store(false, Ordering::Relaxed);

        info!("poseidonos started");
    }
}

// TODO: cfg로 linux profile vs. macos (windows) profile 만들어서 전자의 경우는 lib link, 후자의 경우는 stub
pub mod spdk_clib {
    use crate::generated::bindings::{FILE, option, size_t, spdk_app_opts, spdk_app_parse_args_rvals_SPDK_APP_PARSE_ARGS_SUCCESS, spdk_app_parse_args_rvals_t, spdk_log_level, spdk_msg_fn};

    pub(crate) fn spdk_app_opts_init(opts: &mut spdk_app_opts, opts_size: size_t) {
        // STUB
        return;
    }

    pub fn spdk_app_parse_args(
        argc: ::std::os::raw::c_int,
        argv: *mut *mut ::std::os::raw::c_char,
        opts: *mut spdk_app_opts,
        getopt_str: *const ::std::os::raw::c_char,
        app_long_opts: *mut option,
        parse: ::std::option::Option<
            unsafe extern "C" fn(
                ch: ::std::os::raw::c_int,
                arg: *mut ::std::os::raw::c_char,
            ) -> ::std::os::raw::c_int,
        >,
        usage: ::std::option::Option<unsafe extern "C" fn()>,
    ) -> spdk_app_parse_args_rvals_t {
        // STUB
        return spdk_app_parse_args_rvals_SPDK_APP_PARSE_ARGS_SUCCESS;
    }

    pub(crate) fn spdk_app_start(
        opts_user: *mut spdk_app_opts,
        start_fn: spdk_msg_fn,
        ctx: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int {
        // STUB
        return 0;
    }

    pub fn spdk_app_stop(rc: ::std::os::raw::c_int) {
        // STUB
        return;
    }

    pub fn spdk_app_fini() {
        // STUB
    }

    pub fn spdk_memzone_dump(f: *mut FILE) {
        // STUB
    }

    pub fn spdk_log_set_level(level: spdk_log_level) {
        // STUB
    }

    pub fn spdk_log_set_print_level(level: spdk_log_level) {
        // STUB
    }

    pub fn spdk_log_set_flag(flag: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int {
        // STUB
        0
    }

    pub fn spdk_log_clear_flag(flag: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int {
        // STUB
        0
    }
}

impl Default for spdk_app_opts {
    fn default() -> Self {
        spdk_app_opts {
            name: CString::new("default-name").unwrap().into_raw(),
            json_config_file: 0 as *const c_char,
            json_config_ignore_errors: false,
            rpc_addr: 0 as *const c_char,
            reactor_mask: 0 as *const c_char,
            tpoint_group_mask: 0 as *const c_char,
            shm_id: 0,
            shutdown_cb: None,
            enable_coredump: false,
            mem_channel: 0,
            main_core: 0,
            mem_size: 0,
            no_pci: false,
            hugepage_single_segments: false,
            unlink_hugepage: false,
            hugedir: 0 as *const c_char,
            print_level: 0,
            num_pci_addr: 0,
            pci_blocked: 0 as *mut spdk_pci_addr,
            pci_allowed: 0 as *mut spdk_pci_addr,
            iova_mode: 0 as *const c_char,
            delay_subsystem_init: false,
            num_entries: 0,
            env_context: 0 as *mut c_void,
            log: None,
            base_virtaddr: 0,
            opts_size: 0
        }
    }
}

impl Default for option {
    fn default() -> Self {
        option {
            name: CString::new("default-option").unwrap().into_raw(),
            has_arg: 0,
            flag: 0 as *mut c_int,
            val: 0
        }
    }
}

// extern "C" fn func1() -> ~str {
// ~"hello"
// }