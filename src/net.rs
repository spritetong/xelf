use std::{
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

/// Extension for socket2::socket.
pub trait SocketRsx: Sized {
    fn any_addr(base: IpAddr, port: u16) -> SocketAddr {
        if base.is_ipv4() {
            SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), port)
        } else {
            SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), port)
        }
    }

    /// Bind a UDP socket.
    ///
    /// On Windows, set **SIO_UDP_CONNRESET** to FALSE.
    fn udp_bind(addr: SocketAddr, v6_only: bool, reuse_address: bool) -> io::Result<Self>;

    /// Bind a TCP socket.
    fn tcp_bind(addr: SocketAddr, v6_only: bool, reuse_address: bool) -> io::Result<Self>;

    /// Set the **SIO_UDP_CONNRESET** flag to a windows socket.
    fn set_udp_connreset(&self, value: bool) -> io::Result<()>;
}

#[cfg(feature = "socket2")]
impl SocketRsx for socket2::Socket {
    fn udp_bind(addr: SocketAddr, v6_only: bool, reuse_address: bool) -> io::Result<Self> {
        let s = socket2::Socket::new(
            if addr.is_ipv4() {
                socket2::Domain::IPV4
            } else {
                socket2::Domain::IPV6
            },
            socket2::Type::DGRAM,
            Some(socket2::Protocol::UDP),
        )?;
        if addr.is_ipv6() {
            s.set_only_v6(v6_only).ok();
        }
        if reuse_address && addr.port() != 0 {
            s.set_reuse_address(true)?;
        }
        s.set_nonblocking(true)?;
        s.set_udp_connreset(false)?;
        s.bind(&addr.into())?;
        Ok(s)
    }

    fn tcp_bind(addr: SocketAddr, v6_only: bool, reuse_address: bool) -> io::Result<Self> {
        let s = socket2::Socket::new(
            if addr.is_ipv4() {
                socket2::Domain::IPV4
            } else {
                socket2::Domain::IPV6
            },
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )?;
        if addr.is_ipv6() {
            s.set_only_v6(v6_only).ok();
        }
        if reuse_address && addr.port() != 0 {
            s.set_reuse_address(true)?;
        }
        s.set_nonblocking(true)?;
        s.bind(&addr.into())?;
        Ok(s)
    }

    fn set_udp_connreset(&self, value: bool) -> io::Result<()> {
        #[cfg(windows)]
        {
            use std::mem::size_of;
            use std::ptr::null_mut;
            use winapi::shared::minwindef::{BOOL, DWORD};
            use winapi::um::mswsock::SIO_UDP_CONNRESET;
            use winapi::um::winsock2::{WSAIoctl, SOCKET};

            let mut value = BOOL::from(value);
            let mut bytes_returned: DWORD = 0;
            // SAFETY: call Windows socket API.
            match unsafe {
                debug_assert_eq!(size_of::<Self>(), size_of::<SOCKET>());
                WSAIoctl(
                    *(self as *const _ as *const SOCKET),
                    SIO_UDP_CONNRESET,
                    &mut value as *const _ as _,
                    size_of::<BOOL>() as DWORD,
                    null_mut(),
                    0,
                    &mut bytes_returned,
                    null_mut(),
                    None,
                )
            } {
                0 => Ok(()),
                _ => Err(io::Error::last_os_error()),
            }
        }

        #[cfg(not(windows))]
        {
            let _ = value;
            Ok(())
        }
    }
}
