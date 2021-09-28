use std::io::BufReader;
use std::os::unix::net::UnixStream;

use qapi::{Qmp, Stream};

use super::QemuHandler;
use crate::Result;

#[derive(Debug)]
pub struct Stdout {
    pub monitor: String,
    pub serials: Vec<String>,
}

impl QemuHandler for Stdout {
    fn qemu_args(&self) -> Vec<String> {
        let mut args = vec![
            "-chardev".to_string(),
            format!("file,id={},path=/dev/stdout", self.monitor),
        ];

        for serial in &self.serials {
            args.extend([
                "-chardev".to_string(),
                format!("file,id={},path=/dev/stdout", serial),
            ]);
        }

        args
    }

    fn setup(&self, _qmp: &mut Qmp<Stream<BufReader<&UnixStream>, &UnixStream>>) -> Result<()> {
        Ok(())
    }

    fn wait(&self) -> Result<()> {
        Ok(())
    }
}
