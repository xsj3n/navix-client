use std::{fmt::Display, ops::Index};

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

// returns first instance found 
pub fn subslice_contains<T: PartialEq>(slice: &[T], target: &[T]) -> bool {
    for i in 0..slice.len() {
        if slice[i] == target[0] {
            for j in 1..target.len() {
                if slice[i + j] != target[j] { break; }
                if slice[i + j] == target[j] && j == target.len() - 1 { return true; }
            }
        }
    }
    return false;
} 

      
 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subslice_contains_test() {
        let data = b"foo bar\r\n\r\nfoo bar";
        let data2 = b"foo bar\r\n\rfoo bar";
        let delimiter = b"\r\n\r\n";
        assert!(subslice_contains(data,delimiter));
        assert!(!subslice_contains(data2,delimiter))
    }
}
