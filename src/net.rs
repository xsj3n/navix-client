use std::{io::{Read, Write}, net::TcpStream, sync::Arc};
use rustls::{ClientConnection, StreamOwned, pki_types::ServerName};
use serde::{Serialize, Deserialize};
use crate::state::*;

type NetworkResponse = Result<String, Box<dyn std::error::Error>>;
type NetworkOk = Result<(), Box<dyn std::error::Error>>;
type NetworkIO = Result<usize, Box<dyn std::error::Error>>;

#[derive(Deserialize, Serialize)]
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
    msg_type: MessageType,
    is_rebuilding: bool,
    configuration: String,
    last_sync_timestamp: String 
}

impl Poll {
    fn new() -> Self {  
        return Self {
            msg_type: MessageType::Poll,
            is_rebuilding: false, // TODO,
            configuration: "default".to_string(), // TODO
            last_sync_timestamp: get_last_sync_timestamp().unwrap_or("failed".to_string()) 
        }
    }
}





// ===



struct Server {
    hostname: String,
    port: u32,
    tls_state: Option<ClientConnection>,
    tls_stream: Option<StreamOwned<ClientConnection, TcpStream>>
}

impl Server {
    fn new(hostname: String, port: u32, state: ClientConnection) -> Self {
        return Self {
            hostname: hostname,
            port: port,
            tls_state: Some(state),
            tls_stream: None
        }
    }

    fn get_root_store(&self) -> rustls::RootCertStore {
        return rustls::RootCertStore::from_iter(
            webpki_roots::TLS_SERVER_ROOTS.iter().cloned()
        );
    }
    

    fn addr(&self) -> String {
        return format!("{}:{}", self.hostname, self.port);
    }

    fn default_tls_cfg(&self) -> Arc<rustls::ClientConfig> {
        return Arc::new(rustls::ClientConfig::builder()
            .with_root_certificates(self.get_root_store())
            .with_no_client_auth());
    }
    
    fn default_state(&self) -> Result<ClientConnection, Box<dyn std::error::Error>> {
        return Ok(ClientConnection::new(
            self.default_tls_cfg(),
            ServerName::try_from(self.hostname.clone()).unwrap()
        )?);
    }

    fn connect(&mut self) -> NetworkOk {
        let tcp_stream = TcpStream::connect(self.addr())?;
        let tls_state = self.tls_state.take().unwrap_or(self.default_state()?);
        self.tls_stream = Some(StreamOwned::new(tls_state, tcp_stream));
        return Ok(());
    }

    fn write(&mut self, buf: &[u8]) -> NetworkIO {
        match self.tls_stream.as_mut() {
            None => return Err("No TLS stream has been connected".into()),
            Some(s) => return Ok(s.write(buf)?)
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> NetworkIO {
        match self.tls_stream.as_mut() {
            None => return Err("No TLS stream has been connected".into()),
            Some(s) => return Ok(s.read(buf)?)
        }
    }

    fn poll(&self) -> NetworkResponse {
        let poll = Poll::new();
        let response_buffer = Vec::<u8>::new();
        
    }

    
}




