use rand::{thread_rng, Rng};
use nix::sys::socket::{socket, sockopt, getsockopt, setsockopt, AddressFamily, SockType, SockFlag, SockProtocol};

#[cfg(target_os = "linux")]
#[test]
fn is_so_mark_functional() {
    use nix::sys::socket::sockopt;

    require_capability!(CAP_NET_ADMIN);

    let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();
    setsockopt(s, sockopt::Mark, &1337).unwrap();
    let mark = getsockopt(s, sockopt::Mark).unwrap();
    assert_eq!(mark, 1337);
}

#[test]
fn test_so_buf() {
    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), SockProtocol::Udp)
             .unwrap();
    let bufsize: usize = thread_rng().gen_range(4096, 131_072);
    setsockopt(fd, sockopt::SndBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::SndBuf).unwrap();
    assert!(actual >= bufsize);
    setsockopt(fd, sockopt::RcvBuf, &bufsize).unwrap();
    let actual = getsockopt(fd, sockopt::RcvBuf).unwrap();
    assert!(actual >= bufsize);
}

#[test]
fn test_so_tcp_mss() {
    use std::net::SocketAddr;
    use std::str::FromStr;
    use nix::sys::socket::{bind, connect, listen, InetAddr, SockAddr};
    use nix::unistd::{close};

    let segsize: u32 = thread_rng().gen_range(700, 1300);

    let std_sa = SocketAddr::from_str("127.0.0.1:4001").unwrap();
    let inet_addr = InetAddr::from_std(&std_sa);
    let sock_addr = SockAddr::new_inet(inet_addr);

    let rsock = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), SockProtocol::Tcp)
                .unwrap();
    bind(rsock, &sock_addr).unwrap();
    listen(rsock, 10).unwrap();
    setsockopt(rsock, sockopt::TcpMaxSeg, &segsize).unwrap();
    let actual = getsockopt(rsock, sockopt::TcpMaxSeg).unwrap();
    // Initial MSS is expected to be 536 (https://tools.ietf.org/html/rfc879#section-1) but some
    // platforms keep it even lower. This might fail if you've tuned your initial MSS to be larger
    // than 700
    assert!(actual < segsize);

    // Connect and check the MSS we advertised
    let ssock = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), SockProtocol::Tcp)
                .unwrap();
    connect(ssock, &sock_addr).unwrap();
    let actual = getsockopt(ssock, sockopt::TcpMaxSeg).unwrap();
    // Actual max segment size takes header lengths into account, max IPv4 options (60 bytes) + max
    // TCP options (40 bytes) are subtracted from the requested maximum as a lower boundary.
    assert!((segsize - 100) <= actual);
    assert!(actual <= segsize);
    close(rsock).unwrap();
    close(ssock).unwrap();
}

// The CI doesn't supported getsockopt and setsockopt on emulated processors.
// It's beleived that a QEMU issue, the tests run ok on a fully emulated system.
// Current CI just run the binary with QEMU but the Kernel remains the same as the host.
// So the syscall doesn't work properly unless the kernel is also emulated.
#[test]
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    any(target_os = "freebsd", target_os = "linux")
))]
fn test_tcp_congestion() {
    use std::ffi::OsString;

    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();

    let val = getsockopt(fd, sockopt::TcpCongestion).unwrap();
    setsockopt(fd, sockopt::TcpCongestion, &val).unwrap();

    setsockopt(fd, sockopt::TcpCongestion, &OsString::from("tcp_congestion_does_not_exist")).unwrap_err();

    assert_eq!(
        getsockopt(fd, sockopt::TcpCongestion).unwrap(),
        val
    );
}

#[test]
#[cfg(any(target_os = "android", target_os = "linux"))]
fn test_bindtodevice() {
    skip_if_not_root!("test_bindtodevice");

    let fd = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();

    let val = getsockopt(fd, sockopt::BindToDevice).unwrap();
    setsockopt(fd, sockopt::BindToDevice, &val).unwrap();

    assert_eq!(
        getsockopt(fd, sockopt::BindToDevice).unwrap(),
        val
    );
}
