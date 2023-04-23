use p2p::peer;

pub(crate) fn device_type() -> peer::DeviceType {
    #[cfg(target_os = "windows")]
    return win::device_type();
    #[cfg(target_os = "ios")]
    return ios::device_type();
}

pub(crate) fn host_name() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| String::from("UNKNOWN"))
}

#[cfg(target_os = "windows")]
mod win {
    use p2p::peer;

    pub fn device_type() -> peer::DeviceType {
        peer::DeviceType::WindowsLaptop
    }
}

#[cfg(target_os = "ios")]
mod ios {
    use p2p::peer;

    pub fn device_type() -> peer::DeviceType {
        peer::DeviceType::AppleiPhone
    }
}
