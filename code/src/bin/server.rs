use std::env;
use anyhow;
use code::simple_tcp::server::run_server;
use std::path::PathBuf;

/// This is only used if the server is run from the command line, not when initiated by a UDP
/// multicast request.
#[tokio::main]
async fn main() -> anyhow::Result<()>
{
    let mut args = env::args().skip(1);
    let file_name = match args.next()
    {
        Some(s) => s,
        None => {
            eprintln!("usage: server <file_name>");
            std::process::exit(2);
        }
    };

    let path_buf = PathBuf::from(file_name);
    println!("{:#?}", path_buf);
    let result = run_server(PathBuf::from(path_buf), 488596).await;

    println!("result: {:#?}", result);

    Ok(())
}
