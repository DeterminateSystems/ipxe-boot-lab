use std::fmt::Debug;
use std::io::BufReader;
use std::os::unix::net::UnixStream;

use qapi::{Qmp, Stream};

pub mod manual;
pub mod stdout;

use crate::Result;

pub trait QemuHandler: Debug {
    fn qemu_args(&self) -> Vec<String>;

    fn setup(&self, qmp: &mut Qmp<Stream<BufReader<&UnixStream>, &UnixStream>>) -> Result<()>;
}
