use std::fs::{read};
use std::process::Command;
use base64::Engine;
use base64::engine::general_purpose;



type HashingError = Box<dyn std::error::Error>;
type FilePath = String;

fn command_to_string(cmd: impl AsRef<str>) -> Result<String, Box<dyn std::error::Error>> {
    let split: Vec<&str> = cmd.as_ref().split(" ").collect();
    let mut cmd_obj =  Command::new(split[0]);
    split.iter().for_each(|s|{
        if split[0] != *s {
            cmd_obj.arg(s);
        }
    });
    
    let output = cmd_obj.output()?.stdout;
    if output.len() == 0 {
        return Err(format!("{} returned 0 bytes", split[0]).into());
    }
    return Ok(
        String::from_utf8(output)?
    );
    
}


fn get_disk_serial() -> Result<Vec<u8>, HashingError> {
    let root_disk = command_to_string("findmnt -n -o SOURCE /")
        .map_err(|e| format!("Failed to gather findmnt output: {}", e))?;
    

    let udevadm_output = command_to_string(format!("udevadm info --query=property --name={}", root_disk.trim()))
        .map_err(|e| format!("Failed to gather udevadm output: {}", e))?;


    match udevadm_output.lines().find(|s| s.contains("ID_SERIAL_SHORT=")) {
        Some(s) => return Ok(s.split("=").last().unwrap().as_bytes().to_owned()),
        None =>  return Err(format!("get_disk_serial(): udevadm returned: {}", udevadm_output).into())
    }    
}


fn get_tpm_pub_ek(out_dir: FilePath) -> Result<Vec<u8>, HashingError> {
    let ek_pub_path = out_dir.clone() + "ek.pub";
    let tpm2_createek_status = Command::new("tpm2_createek")
        .arg("-G")
        .arg("rsa")
        .arg("-u")
        .arg(&ek_pub_path)
        .arg("-c")
        .arg(out_dir + "ek.ctx")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !tpm2_createek_status.success() {
        let err_msg = format!("tpm2_createek exited with error code: {}", tpm2_createek_status.code().unwrap_or(-1));
        return Err(err_msg.into());
    }
    
    return Ok(
        read(&ek_pub_path).map_err(|e| format!("Failed to read produced EK public key at {}: {}", &ek_pub_path, e))?
    );
 }   


// there may be edge cases where a machine had more than one pcie nic slot but
// we will keep it simple for now 
fn get_mac_address() -> Result<Vec<u8>, HashingError> {

    // we want to get all NIC MACS 
    let linked_paths: std::fs::ReadDir = std::fs::read_dir("/sys/class/net")?;
    let followed_paths: Vec<String> = linked_paths.map(|e| {
        let e = e.unwrap();
        return std::fs::canonicalize(e.path()).unwrap().to_str().unwrap().to_owned(); 
    })
    .filter(|p| !p.contains("virtual") && !p.contains("usb"))
    .collect();

    for path in followed_paths.iter() {
            let eth_nic_entry: std::fs::DirEntry = std::fs::read_dir(path).map_err(|e| format!("Failed to read linked path {}: {}", path.to_string(), e))?
                .filter_map(|entry| entry.ok())
                .find(|entry| !entry.path().to_str().unwrap().contains("wireless"))
                .expect("Failed to read MAC symlink from /sys/class/net");
            return Ok(
                read(eth_nic_entry.path()).map_err(|e| format!("Failed to read derived ethernet NIC path {}: {}", eth_nic_entry.path().to_str().unwrap(), e))?
            )
    }
    
    return Err("MAC address conditionals unmet".into());
}

fn get_board_serial() -> Result<Vec<u8>, HashingError> {
    return Ok(
        read("/sys/class/dmi/id/board_serial")
            .map_err(|e| format!("Error reading board_serial: {}", e))?
    );
}

fn get_tpm_version() -> Result<Vec<u8>, HashingError> {
    return Ok(
        read("/sys/class/tpm/tpm0/tpm_version_major")
            .map_err(|e| format!("Error reading tpm_version_major: {}", e))?
    ); 
}

fn get_product_family() -> Result<Vec<u8>, HashingError> {
    return Ok(
        read("/sys/class/dmi/id/product_family")
            .map_err(|e| format!("Error reading product_family: {}", e))?
    );
}

fn get_manufacturer() -> Result<Vec<u8>, HashingError> {
    return Ok(
        read("/sys/class/dmi/id/bios_vendor")
            .map_err(|e| format!("Error reading bios_vendor: {}", e))?
    );
}

fn get_product_name() -> Result<Vec<u8>,  HashingError> {
    return Ok(
        read("/sys/class/dmi/id/product_name")
            .map_err(|e| format!("Error reading product_name: {}", e))?
    );
}

fn get_bios_uuid() -> Result<Vec<u8>,  HashingError> {
    return Ok(
        read("/sys/class/dmi/id/product_uuid")
            .map_err(|e| format!("Error reading product_uuid: {}", e))?
    );
}

#[derive(Debug)]
pub struct MachineFingerprint {
    board_serial: Vec<u8>,
    disk_serial: Vec<u8>,
    tpm_ek_pub: Vec<u8>,
    tpm_version: Vec<u8>,
    mac_addr: Vec<u8>,
    product_family: Vec<u8>,
    manufacturer: Vec<u8>,
    product_name: Vec<u8>,
    bios_uuid: Vec<u8>,
    hardware_fingerprint: String
}

impl MachineFingerprint {
    pub fn new() -> Result<Self, Vec<HashingError>> {
        let out_dir = "./".to_string();
        let results = vec![
            get_board_serial(),
            get_disk_serial(),
            get_tpm_pub_ek(out_dir),
            get_tpm_version(),
            get_mac_address(),
            get_product_family(),
            get_manufacturer(),
            get_product_name(),
            get_bios_uuid()
        ];

        // collecting errors makes things easier down the line
        if results.iter().any(|r| r.is_err()) {
            return Err(results.into_iter().filter_map(|r| r.err()).collect());
        }

        let bytes: Vec<Vec<u8>> = results.into_iter().map(|r| r.unwrap()).collect();
        let hardware_id = bytes.iter().map(|b| general_purpose::STANDARD.encode(b))
            .collect::<Vec<_>>()
            .join("::");
   
        
        return Ok(Self {
            board_serial:   bytes[0].clone(),
            disk_serial:    bytes[1].clone(),
            tpm_ek_pub:     bytes[2].clone(),
            tpm_version:    bytes[3].clone(),
            mac_addr:       bytes[4].clone(),
            product_family: bytes[5].clone(),
            manufacturer:   bytes[6].clone(),
            product_name:   bytes[7].clone(),
            bios_uuid:      bytes[8].clone(),
            hardware_fingerprint: hardware_id,
        });
    }
}



pub fn chk_if_managed() -> Result<bool, Box<dyn std::error::Error>> {
 
    let hostname = command_to_string("hostname")?;
    let cert_path = "./".to_string() + &hostname + ".cer";
    if std::path::Path::new(&cert_path).exists() {
        return Ok(true);
    }

    

    return Ok(false);
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_information() -> () {
        
        match MachineFingerprint::new() {
            Ok(f) => println!("Hardware hash:\n{:#?}\n\n=",f.hardware_fingerprint),
            Err(e) => {
                println!("Collected hashing errors:");
                e.into_iter().for_each(|error| println!("[!]: {}", error));
            }
        }
    }
}
