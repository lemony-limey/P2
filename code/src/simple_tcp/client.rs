//! Simple IPv6 link-local client
//! (based on Saleem's Java version)

use std::{
    env,
    ffi::CString,
    io::{self, Read, Write},
    net::{Ipv6Addr, SocketAddr, SocketAddrV6, TcpStream},
    str::FromStr
};

fn parse_ipv6_with_scope(addr_str: &str, port: u16) -> io::Result<SocketAddr>
{
    // this is a little.. messy
    if let Some(percent_idx) = addr_str.find('%')
    {
        let ip_part = &addr_str[..percent_idx];
        let ifname = &addr_str[percent_idx + 1..];

        let ip = Ipv6Addr::from_str(ip_part)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        // Convert interface name (e.g. "eno1") to index using libc
        let c_ifname = CString::new(ifname)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "bad iface name"))?;
        let ifindex = unsafe { libc::if_nametoindex(c_ifname.as_ptr()) };
        if ifindex == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("interface '{}' not found", ifname),
            ));
        }

        Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, ifindex)))
    }
    else
    {
        // try parsing as a raw SocketAddr (e.g. "::1" or "127.0.0.1")
        if let Ok(ipv6) = Ipv6Addr::from_str(addr_str)
        {
            Ok(SocketAddr::V6(SocketAddrV6::new(ipv6, port, 0, 0)))
        }
        else
        {
            Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("could not get IPv6 address from input string: \"{addr_str}\"")
                )
            )
        }
    }
}

/// The client is the node which contains a file which has been requested for download by another
/// node. The client will send the data to the server.
fn run_client() -> io::Result<()>
{
    let mut args = env::args().skip(1);
    let server = match args.next()
    {
        Some(s) => s,
        None => {
            eprintln!("usage: client <server-ip-or-host> <port>");
            std::process::exit(2);
        }
    };

    let port: u16 = match args.next()
    {
        Some(p) => p.parse().expect("port must be a number"),
        None => {
            eprintln!("usage: client <server-ip-or-host> <port>");
            std::process::exit(2);
        }
    };

    // build SocketAddr taking into account IPv6 link-local scope ids
    let addr = match parse_ipv6_with_scope(&server, port)
    {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Could not resolve address: {}", e);
            std::process::exit(1);
        }
    };

    println!("connecting to {}...", addr);

    let mut stream = TcpStream::connect(addr)?;
    let mut total_bytes_sent: usize = 0;
    let mut total_bytes_received: usize = 0;

    println!("connected! sending hello...\n");

    let message = b"hello\n";
    stream.write_all(message)?;
    total_bytes_sent += message.len();

    // read response
    let mut buf = [0u8; 1024];
    match stream.read(&mut buf)
    {
        Ok(n) if n > 0 => {
            println!("received ({} bytes):\n{}", n, String::from_utf8_lossy(&buf[..n]));
            total_bytes_received += n;
        }
        Ok(_) => println!("connection closed by peer with no data"),
        Err(e) => eprintln!("read error: {}", e),
    }

    Ok(())
}
