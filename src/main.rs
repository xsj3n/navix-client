use std::{net::TcpStream, sync::Arc};
use rustls::{Stream, pki_types::ServerName};

pub mod init;
pub mod net;
pub mod logging;

#[tokio::main]
async fn main() {
    const HOST: &'static str  = "localhost";
    const PORT: u32     = 3000;
    
    let root_store = rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
    );

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();


    let rc_config = Arc::new(config);
    let mgmt_server = ServerName::try_from("localhost").expect("[!] Invalid DNS name"); 
    let mut conn = rustls::ClientConnection::new(rc_config, mgmt_server).expect("[!] Failed to initalize TLS state");
        
    
    let mut tcp = TcpStream::connect(format!("{HOST}:{PORT}")).expect("[!] Failed to establish connection");
    let mut tls = Stream::new(&mut conn, &mut tcp);


    let mut response_buffer = Vec::<u8>::new();
    
}
