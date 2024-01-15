/// # bsod
/// The safest library on the block. Calling the bsod function will cause a blue screen of death.
/// ## links
/// - [`crates.io`](https://crates.io/crates/bsod)
/// - [`docs.rs`](https://docs.rs/bsod/latest/bsod/)
use std::{
    ffi::{c_ulong, c_ulonglong, CString},
    mem::transmute,
};

#[cfg(target_os = "linux")]
use std::process::Command;
#[cfg(target_os = "linux")]
use zbus::export::serde::Serialize;
#[cfg(target_os = "linux")]
use zbus::zvariant::DynamicType;

#[cfg(windows)]
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{NTSTATUS, STATUS_FLOAT_MULTIPLE_FAULTS},
        System::LibraryLoader::{GetProcAddress, LoadLibraryA},
    },
};

#[cfg(windows)]
type RtlAdjustPrivilige = unsafe extern "C" fn(
    privilge: c_ulong,
    enable: bool,
    currentThread: bool,
    enabled: *mut bool,
) -> NTSTATUS;
#[cfg(windows)]
type NtRaiseHardError = unsafe extern "C" fn(
    errorStatus: NTSTATUS,
    numberOfParams: c_ulong,
    unicodeStrParamMask: c_ulong,
    params: *const c_ulonglong,
    responseOption: c_ulong,
    response: *mut c_ulong,
) -> i64;

#[cfg(windows)]
macro_rules! make_pcstr {
    ($str:expr) => {{
        // considering the program will exit almost immediately i don't care about mem leaks.
        let cstr = CString::new($str).unwrap();

        let pc = PCSTR::from_raw(cstr.as_ptr() as *const u8);

        std::mem::forget(cstr);

        pc
    }};
}

/// this function will cause a blue screen of death
#[cfg(windows)]
pub fn bsod() {
    unsafe {
        let hndl = LoadLibraryA(make_pcstr!("ntdll.dll")).expect("ntdll to exist");
        let adjust_priv: RtlAdjustPrivilige = transmute(
            GetProcAddress(hndl, make_pcstr!("RtlAdjustPrivilege"))
                .expect("RtlAdjustPrivilige to exist"),
        );
        let raise_hard_err: NtRaiseHardError = transmute(
            GetProcAddress(hndl, make_pcstr!("NtRaiseHardError"))
                .expect("NtRaiseHardError to exist"),
        );

        let mut lol: c_ulong = 0;
        let mut enabled = false;
        adjust_priv(19, true, false, &mut enabled);
        raise_hard_err(
            STATUS_FLOAT_MULTIPLE_FAULTS,
            0,
            0,
            std::mem::zeroed(),
            6,
            &mut lol,
        );
    }
}

#[cfg(all(not(windows), not(target_os = "linux")))]
pub fn bsod() {}

/// this will cause a shutdown on linux
#[cfg(target_os = "linux")]
pub fn bsod() {
    dbus(
        "org.kde.ksmserver",
        "/KSMServer",
        "org.kde.KSMServerInterface",
        "logout",
        &(-1, 2, 2),
    );
    dbus(
        "org.gnome.SessionManager",
        "org/gnome/SessionManager",
        "org.gnome.SessionManager",
        "Shutdown",
        &(),
    );
    dbus(
        "org.xfce.SessionManager",
        "/org/xfce/SessionManager",
        "org.xfce.SessionManager",
        "Shutdown",
        &(true),
    );
    dbus(
        "org.freedesktop.login1",
        "/org/freedesktop/login1",
        "org.freedesktop.login1.Manager",
        "PowerOff",
        &(true),
    );
    dbus(
        "org.freedesktop.PowerManagement",
        "/org/freedesktop/PowerManagement",
        "org.freedesktop.PowerManagement",
        "Shutdown",
        &(),
    );
    dbus(
        "org.freedesktop.SessionManagement",
        "/org/freedesktop/Sessionmanagement",
        "org.freedesktop.SessionManagement",
        "Shutdown",
        &(),
    );
    dbus(
        "org.freedesktop.ConsoleKit",
        "/org/freedesktop/ConsoleKit/Manager",
        "org.freedesktop.ConsoleKit.Manager",
        "Stop",
        &(),
    );
    dbus(
        "org.freedesktop.Hal",
        "/org/freedesktop/Hal/devices/computer",
        "org.freedesktop.Hal.Device.SystemPowermanagement",
        "Shutdown",
        &(),
    );
    dbus(
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "PowerOff",
        &(),
    );
    // If dbus doesn't work, this is the last resort
    Command::new("shutdown").args(&["-h", "now"]).output();
}

#[cfg(target_os = "linux")]
fn dbus<T: Serialize + DynamicType>(
    destination: &str,
    path: &str,
    interface: &str,
    method: &str,
    body: &T,
) {
    let mut owner: bool = false;
    if let Ok(connection) = zbus::blocking::Connection::session() {
        let reply = connection.call_method(
            Some("org.freedesktop.DBus"),
            "/",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &(destination),
        );
        owner = reply.and_then(|r| r.body()).unwrap_or(false);
    }
    if owner {
        if let Ok(connection) = zbus::blocking::Connection::session() {
            let reply =
                connection.call_method(Some(destination), path, Some(interface), method, body);
        }
    }
}
