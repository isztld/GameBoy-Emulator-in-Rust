//! Configuration module for emulator flags

/// Emulator configuration flags
#[derive(Debug, Clone)]
pub struct EmulatorFlags {
    pub log_cpu: bool,
    pub log_cpu_file: String,
    pub log_serial: bool,
    pub log_serial_file: String,
}

impl Default for EmulatorFlags {
    fn default() -> Self {
        EmulatorFlags {
            log_cpu: false,
            log_cpu_file: "cpu_log.txt".to_string(),
            log_serial: false,
            log_serial_file: "serial_log.txt".to_string(),
        }
    }
}
