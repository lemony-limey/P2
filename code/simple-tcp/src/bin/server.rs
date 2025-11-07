//! Simple non-blocking TCP server using IPv6 link-local addresses
//! (based on Saleem's Java version)

use std::{
    io::{Read, Write},
    net::{TcpListener, SocketAddr, IpAddr, Ipv6Addr},
    thread,
    time::Duration,
};
use log::{info, error};

const SLEEP_TIME_MS: u64 = 1000;
const BUFFER_SIZE: usize = 80;
const SO_TIMEOUT_MS: u64 = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();
    
    info!("Starting TCP server");
    
    let port = simple_tcp::get_uid_port();
    info!("Using port: {}", port);
    
    // For link-local binding, we need to bind to the unspecified address
    // The system will handle the interface selection
    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port);
    
    info!("Binding to: {}", bind_addr);
    let listener = TcpListener::bind(bind_addr)?;
    listener.set_nonblocking(true)?;
    
    // Log the actual link-local address for reference
    if let Ok((ipv6_addr, interface)) = simple_tcp::get_link_local_ipv6() 
    {
        info!("Link-local IPv6 available: {} on interface {}", ipv6_addr, interface);
        info!("Clients should connect to: [{}%{}]:{}", ipv6_addr, interface, port);
    }
    
    info!("Waiting for connection...");
    
    // Wait for a connection
    let mut stream = loop 
    {
        match listener.accept() 
        {
            Ok((stream, addr)) => {
                info!("New connection from: {}", addr);
                break stream;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No incoming connection, keep trying
                thread::sleep(Duration::from_millis(SO_TIMEOUT_MS));
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
                return Err(e.into());
            }
        }
    };
    
    // No longer need to accept connections
    drop(listener);
    
    stream.set_read_timeout(Some(Duration::from_millis(SO_TIMEOUT_MS)))?;
    stream.set_write_timeout(Some(Duration::from_millis(SO_TIMEOUT_MS)))?;
    
    let mut buffer = vec![0u8; BUFFER_SIZE];
    
    // Echo server loop
    loop 
    {
        match stream.read(&mut buffer)
        {
            Ok(0) => {
                info!("<Connection closed by client>");
                break;
            }
            Ok(bytes_read) => {
                let received = String::from_utf8_lossy(&buffer[..bytes_read]);
                info!("Received {} bytes --> {}", bytes_read, received.trim());
                
                info!("Sending back {} bytes", bytes_read);
                if let Err(e) = stream.write_all(&buffer[..bytes_read])
                {
                    error!("Error writing to client: {}", e);
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock 
                   || e.kind() == std::io::ErrorKind::TimedOut => {
                // No data available, continue waiting
            }
            Err(e) => {
                error!("Error reading from client: {}", e);
                break;
            }
        }
        
        thread::sleep(Duration::from_millis(SLEEP_TIME_MS));
    }
    
    info!("<Closing connection>");
    Ok(())
}
