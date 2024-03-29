use std::fmt::Debug;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::process::Stdio;

use qapi::{Qmp, Stream};

pub mod manual;
pub mod stdout;
pub mod tmux;

use crate::Result;

const SCREEN_INVOCATION: &str = concat!(env!("SCREEN_BIN"), " -h 1000000");

pub trait QemuHandler: Debug {
    fn qemu_args(&self) -> Vec<String>;

    fn setup(&self, qmp: &mut Qmp<Stream<BufReader<&UnixStream>, &UnixStream>>) -> Result<()>;

    fn wait(&self) -> Result<()>;

    fn stderr_destination(&self) -> Stdio;
    fn stdout_destination(&self) -> Stdio;
}
