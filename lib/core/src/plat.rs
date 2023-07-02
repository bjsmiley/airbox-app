use p2p::peer;

#[cfg(target_os = "ios")]
use ios as plat;
#[cfg(target_os = "windows")]
use win as plat;

pub const DEVICE_TYPE: peer::DeviceType = plat::DEVICE_TYPE;

pub(crate) fn host_name() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| String::from("my-flydrop"))
}

#[cfg(target_os = "windows")]
mod win {
    use p2p::peer;

    pub const DEVICE_TYPE: peer::DeviceType = peer::DeviceType::WindowsLaptop;
}

#[cfg(target_os = "ios")]
mod ios {
    use p2p::peer;

    pub const DEVICE_TYPE: peer::DeviceType = peer::DeviceType::AppleiPhone;
}
