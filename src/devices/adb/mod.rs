pub mod client;
pub mod protocol;

pub use client::{exec, shell_escape};
pub use protocol::{
    connect_to_adb, exec_shell, list_devices, read_device_list_update, send_adb_cmd,
    subscribe_track_devices,
};
