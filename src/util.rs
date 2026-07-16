pub async fn command_to_string(cmd: impl AsRef<str>) -> Result<String, Box<dyn std::error::Error>> {
    let split: Vec<&str> = cmd.as_ref().split(" ").collect();
    let mut cmd_obj =  std::process::Command::new(split[0]);
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
