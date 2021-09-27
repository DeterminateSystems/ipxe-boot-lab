use std::io::BufReader;
use std::os::unix::net::UnixStream;

use qapi::{qmp, Qmp, Stream};

use super::{QemuHandler, SCREEN_INVOCATION};
use crate::Result;

#[derive(Debug)]
pub struct Manual {
    pub monitor: String,
    pub serials: Vec<String>,
}

impl QemuHandler for Manual {
    fn qemu_args(&self) -> Vec<String> {
        let mut args = vec!["-chardev".to_string(), format!("pty,id={}", self.monitor)];

        for serial in &self.serials {
            args.extend(["-chardev".to_string(), format!("pty,id={}", serial)]);
        }

        args
    }

    fn setup(&self, qmp: &mut Qmp<Stream<BufReader<&UnixStream>, &UnixStream>>) -> Result<()> {
        let chardevs = qmp.execute(&qmp::query_chardev {})?;

        for dev in chardevs {
            if !self.serials.iter().any(|n| dev.label == *n) && dev.label != self.monitor {
                continue;
            }

            println!(
                ": {} ; {} {} 115200",
                dev.label,
                SCREEN_INVOCATION,
                dev.filename.trim_start_matches("pty:")
            );
        }

        // give you time to run screen
        std::thread::sleep_ms(5000);

        Ok(())
    }
}
