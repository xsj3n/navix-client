use std::{io::{ErrorKind, Error, Read, Write}, net::TcpStream, sync::Arc};
use base64::engine::DecodePaddingMode::RequireCanonical;
use rustls::{ClientConnection, StreamOwned, pki_types::ServerName};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use crate::{state::*, util::subslice_contains};
use crate::init::{HardwareFingerprint};

type NetworkOk = Result<(), Error>;
type NetworkIO = Result<usize, Error>;

// TODO: Move to protobuf eventually 

// Registration struct stuff ===

#[derive(Serialize)]
pub struct Registration {
     fingerprint: HardwareFingerprint,
     enrollment_token: String
}

impl Registration {
    pub fn new(token: String, fingerprint: String) -> Result<Self, Box<dyn std::error::Error>> {
        return Ok(Self {
            fingerprint: fingerprint,
            enrollment_token: token
        });
    }
}


// Poll struct stuff ===


#[derive(Deserialize, Debug)]
pub struct PollResponse {
    pub wipe: bool,
    pub rebuild: bool,
    pub script: bool,
    pub reboot: bool,
    pub rotate_root_pwd: bool,
    pub rotate_luks: bool,
    pub malware_scan: bool 
}

impl PollResponse {
    pub fn new() -> Self {
        return Self {
            wipe: false,
            rebuild: false,
            script: false,
            reboot: false,
            rotate_root_pwd: false,
            rotate_luks: false,
            malware_scan: false
        }
    }   
}

#[derive(Serialize)]
pub struct Poll {
    is_rebuilding: bool,
    configuration: String,
    last_sync_timestamp: String 
}

impl Poll {
    pub fn new() -> Self {  
        return Self {
            is_rebuilding: false, // TODO,
            configuration: "default".to_string(), // TODO
            last_sync_timestamp: get_last_sync_timestamp().unwrap_or("failed".to_string()) 
        }
    }
}


// ===



pub struct Server {
    hostname: String,
    port: u32,
    tls_state: Option<ClientConnection>,
    tls_stream: Option<StreamOwned<ClientConnection, TcpStream>>
}

impl Server {
    pub fn new(hostname: impl AsRef<str>, port: u32, state: Option<ClientConnection>) -> Self {
        return Self {
            hostname: hostname.as_ref().to_string(),
            port: port,
            tls_state: state,
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
    
    fn default_state(&self) -> Result<ClientConnection, Error> {
        let server = match ServerName::try_from(self.hostname.as_str()) {
            Ok(s) => s,
            Err(_) => return Err(Error::new(ErrorKind::Other, "Invalid DNS name or IP address"))
        };
        
        return Ok(
            match ClientConnection::new(self.default_tls_cfg(), server.to_owned()) {
                Ok(c) => c,
                Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string()))
            }
        );
    }

    fn connect(&mut self) -> NetworkOk {
        let tcp_stream = TcpStream::connect(self.addr())?;
        let tls_state = self.tls_state.take().unwrap_or(self.default_state()?);
        self.tls_stream = Some(StreamOwned::new(tls_state, tcp_stream));
        return Ok(());
    }

    fn _write(&mut self, buf: &[u8]) -> NetworkIO {
        match self.tls_stream.as_mut() {
            None => return Err(Error::new(ErrorKind::Other, "No TLS stream has been connected")),
            Some(s) => return Ok(s.write(buf)?)
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> NetworkIO {
        match self.tls_stream.as_mut() {
            None => return Err(Error::new(ErrorKind::Other, "No TLS stream has been connected")),
            Some(s) => return Ok(s.read(buf)?)
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> NetworkOk {
        match self.tls_stream.as_mut() {
            None => return Err(Error::new(ErrorKind::Other, "No TLS stream has been connected")),
            Some(s) => {
                s.write_all(buf)?;
                s.flush()?;
                return Ok(());
            }
        }
    }


    fn close(&mut self) -> NetworkOk {
        match self.tls_stream.as_mut() {
            None => return Err(Error::new(ErrorKind::Other, "No TLS stream has been connected")),
            Some(s) => {
                s.sock.shutdown(std::net::Shutdown::Both)?;
                self.tls_stream = None;
                return Ok(());
            }, 
        }
    }

    fn read_until_delimiter(&mut self, buf: &mut [u8]) -> NetworkIO {
        let mut loops = 0;
        let mut len = 0;
        loop {
            loops = loops + 1;
            if loops > 100 {
                return Err(Error::new(ErrorKind::TimedOut, "Abnormal amount of reads from the server for one message"));
            }

            match self.read(buf) {
                Ok(l) => len = len + l,
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Interrupted => continue,
                        _ => return Err(e)
                    }
                }
            };

            if subslice_contains(&buf, b"\r\n\r\n") { return Ok(len); }
        }
    }



    pub fn request<T: Serialize, RT: DeserializeOwned>(&mut self, request: T) -> Result<RT, Error> {
        let json = serde_json::to_vec(&request)?;
        let mut response_buffer = Vec::<u8>::new();
        let delimiter = b"\r\n\r\n";

        self.connect()?;    
        self.write_all(&json)?;
        self.write_all(delimiter)?;
        self.read_until_delimiter(&mut response_buffer)?;
        loop {

                      
            match self.read(&mut response_buffer) {
                Ok(len)  =>  {
                    if len < target_len + 4 { // account for delimiter 
                        continue;
                    }

                    _ = self.close();
                    return Ok(serde_json::from_slice::<RT>(&response_buffer)?);
                }
        
                Err(e) => {
                    match e.kind() {
                        ErrorKind::Interrupted => continue,
                        _                      => return Err(e)
                    }
                }
            }
        } 
    }
    

     
    
}




