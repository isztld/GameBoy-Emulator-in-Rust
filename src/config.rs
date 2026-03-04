//! Configuration module for emulator flags

/// Emulator configuration flags
#[derive(Debug, Clone)]
pub struct EmulatorFlags {
    pub cpu_json_test: bool,
    pub cpu_json_test_dir: Option<String>,
    pub log_cpu: bool,
    pub log_cpu_file: String,
    pub log_serial: bool,
    pub log_serial_file: String,
    pub cycle_limit: Option<u64>,
}

impl Default for EmulatorFlags {
    fn default() -> Self {
        EmulatorFlags {
            cpu_json_test: false,
            cpu_json_test_dir: None,
            log_cpu: false,
            log_cpu_file: "cpu_log.txt".to_string(),
            log_serial: false,
            log_serial_file: "serial_log.txt".to_string(),
            cycle_limit: None,
        }
    }
}
