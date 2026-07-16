use std::{net::TcpStream, sync::Arc};
use rustls::{Stream, pki_types::ServerName};

use crate::init::chk_if_managed;
use crate::logging::*;

pub mod init;
pub mod net;
pub mod logging;
pub mod util;
pub mod state;

#[tokio::main]
async fn main() {

    let is_managed = chk_if_managed().await;

    if is_managed.is_err() {
        log!(LogLevel::Error, "Failure to check if machine is manged due to error - {}", is_managed.unwrap_err().to_string(); 250);
        return // this return wont ever be reached but is required to make the compiler happy  
    }

    if is_managed.unwrap() != true {
        // TODO: insert enrollment guidance flow
    }  

    const HOST: &'static str  = "localhost";
    const PORT: u32     = 3000;
    
    let root_store = rustls::RootCertStore ::from_iter(
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
