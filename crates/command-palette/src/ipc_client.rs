use anyhow::{Context, Result};
use serde_json::Value;

#[cfg(windows)]
use std::fs::OpenOptions;
#[cfg(windows)]
use std::io::{Read, Write};
#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
const FILE_FLAG_OVERLAPPED: u32 = 0x40000000;

const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

pub struct IpcClient {
    pipe_name: String,
}

impl IpcClient {
    pub fn new() -> Self {
        Self {
            pipe_name: r"\\.\pipe\tenraku".to_string(),
        }
    }

    pub fn execute_command(&self, command: &str, args: Vec<String>) -> Result<()> {
        let request = serde_json::json!({
            "type": "execute",
            "command": command,
            "args": args
        });

        let _response = self.send_request(&request)?;
        Ok(())
    }

    #[cfg(windows)]
    fn send_request(&self, request: &Value) -> Result<Value> {
        // Open pipe connection
        let mut pipe = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(FILE_FLAG_OVERLAPPED)
            .open(&self.pipe_name)
            .context("Failed to open named pipe")?;

        // Serialize and send request
        let data = serde_json::to_vec(request).context("Failed to serialize request")?;
        let len = data.len() as u32;

        pipe.write_all(&len.to_le_bytes())
            .context("Failed to write request length")?;
        pipe.write_all(&data)
            .context("Failed to write request data")?;
        pipe.flush().context("Failed to flush pipe")?;

        // Read response
        let mut len_buf = [0u8; 4];
        pipe.read_exact(&mut len_buf)
            .context("Failed to read response length")?;
        let response_len = u32::from_le_bytes(len_buf) as usize;

        // Validate message size
        if response_len > MAX_MESSAGE_SIZE {
            anyhow::bail!(
                "Response message too large: {} bytes (max: {} bytes)",
                response_len,
                MAX_MESSAGE_SIZE
            );
        }

        let mut response_data = vec![0u8; response_len];
        pipe.read_exact(&mut response_data)
            .context("Failed to read response data")?;

        // Parse response
        let response: Value =
            serde_json::from_slice(&response_data).context("Failed to parse response")?;

        // Check for errors in response
        if let Some(status) = response.get("status") {
            if status == "error" {
                if let Some(message) = response.get("message") {
                    anyhow::bail!("Server error: {}", message);
                } else {
                    anyhow::bail!("Server returned error status");
                }
            }
        }

        Ok(response)
    }

    #[cfg(not(windows))]
    fn send_request(&self, _request: &Value) -> Result<Value> {
        anyhow::bail!("IPC client only works on Windows")
    }
}

impl Default for IpcClient {
    fn default() -> Self {
        Self::new()
    }
}
