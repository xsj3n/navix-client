use std::{io::{Read, Write}, net::TcpStream, sync::Arc};
use rustls::{Stream, pki_types::ServerName};
use serde::{Serialize, Deserialize};



#[derive(Deserialize)]
enum MessageType {
    RegisterationResponse,
    DeregisterationResponse,
    Poll
}

#[derive(Deserialize)]
enum MessageContents {
    RegistrationInformation,
    DeregistrationInformation,
    PollRequest
}

#[derive(Deserialize)]
struct PollResponse {
    msg_type: MessageType,
    msg_contents: String
}
// ===

#[derive(Serialize)]
struct Registration {
    hardware_id: String,
    enrollment_token: String
}

#[derive(Serialize)]
struct Deregistration {
   hardware_id: String 
}

#[derive(Serialize)]
struct Poll {
    is_rebuilding: bool
}


fn get_root_store() -> rustls::RootCertStore {
    return rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
    );
}

fn build_tls_cfg() -> Arc<rustls::ClientConfig> {
    return Arc::new(rustls::ClientConfig::builder()
        .with_root_certificates(get_root_store())
        .with_no_client_auth());
}



async fn new_connection() -> Result<(), Box<dyn std::error::Error>> {
    const HOST: &str  = "localhost";
    const PORT: u32   = 3000;
    
    let rc_config = build_tls_cfg();
    let mgmt_server = ServerName::try_from("localhost")?;
    let mut conn = rustls::ClientConnection::new(rc_config, mgmt_server)?;
        
    
    let mut sock = TcpStream::connect(format!("{HOST}:{PORT}"))?;
    let mut tls_stream = Stream::new(&mut conn, &mut sock);
    let mut response_buffer = Vec::<u8>::new();
    
    
    
     return Ok(());
}
