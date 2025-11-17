//! Simple non-blocking TCP server using IPv6 link-local addresses
//! (based on Saleem's Java version)

use std::fs::File;
use std::io::BufReader;
use crate::{
    constants::{BUFFER_SIZE, SLEEP_TIME_MS, SO_TIMEOUT_MS},
    simple_tcp::{anyhow, self},
};
use std::{
    io::{Read, Write},
    net::{TcpListener, SocketAddr, IpAddr, Ipv6Addr},
    thread,
    time::Duration,
};
use std::path::PathBuf;
use log::{info, error};

/// The server is the node requesting the download of the file from another node.
/// It will receive the file from the client.
pub fn run_server(
    file_requested_from_client: PathBuf,
) -> anyhow::Result<()>
{
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
    //     .format_timestamp_millis()
    //     .init();
    
    info!("Starting TCP server");

    let Some(port) = simple_tcp::get_uid_port() else { return Err(anyhow!("oh no")) };
    info!("Using port: {}", port);
    
    // For link-local binding, we need to bind to the unspecified address
    // The system will handle the interface selection
    let bind_addr = SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port.get());
    
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

    let new_file = File::create_new(file_requested_from_client)?;
    
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

    let mut reader = BufReader::new(stream);

    // Echo server loop
    loop
    {
        let mut buffer = vec![0u8; BUFFER_SIZE];
        match reader.read(&mut buffer)
        {
            Ok(0) => {
                info!("<Connection closed by client>");
                break;
            }
            Ok(bytes_read) => {
                let received = String::from_utf8_lossy(&buffer[..bytes_read]);
                info!("Received {} bytes --> {}", bytes_read, received.trim());

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
