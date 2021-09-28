use std::io::BufReader;
use std::os::unix::net::UnixStream;

use qapi::{qmp, Qmp, Stream};
use tmux_interface::TmuxCommand;

use super::{QemuHandler, SCREEN_INVOCATION};
use crate::Result;

pub(crate) const TARGET_SESSION_NAME: &str = "ipxe_boot_lab";

#[derive(Debug)]
pub struct Tmux {
    pub monitor: String,
    pub serials: Vec<String>,
}

impl QemuHandler for Tmux {
    fn qemu_args(&self) -> Vec<String> {
        let mut args = vec!["-chardev".to_string(), format!("pty,id={}", self.monitor)];

        for serial in &self.serials {
            args.extend(["-chardev".to_string(), format!("pty,id={}", serial)]);
        }

        args
    }

    fn setup(&self, qmp: &mut Qmp<Stream<BufReader<&UnixStream>, &UnixStream>>) -> Result<()> {
        // Fixes error where screen expects to own /tmp/screens/S-root, but cannot because it isn't
        // root outside the namespace:
        // "You are not the owner of /tmp/screens/S-root."
        let temp_dir = tempfile::TempDir::new()?;
        std::env::set_var("SCREENDIR", temp_dir.path());

        let chardevs = qmp.execute(&qmp::query_chardev {})?;
        let monitor = chardevs
            .iter()
            .find(|dev| dev.label == self.monitor)
            .ok_or_else(|| "Couldn't find mon0 chardev")?;

        let tmux = TmuxCommand::new();

        tmux.new_session()
            .detached()
            .window_name(&monitor.label)
            .session_name(TARGET_SESSION_NAME)
            .shell_command(format!(
                "{} {} 115200",
                SCREEN_INVOCATION,
                monitor.filename.trim_start_matches("pty:")
            ))
            .output()?;

        for chardev in chardevs {
            if chardev.label == self.monitor {
                continue;
            }

            if self.serials.contains(&chardev.label) {
                tmux.new_window()
                    .window_name(&chardev.label)
                    .shell_command(format!(
                        "{} {} 115200",
                        SCREEN_INVOCATION,
                        chardev.filename.trim_start_matches("pty:")
                    ))
                    .output()?;
            }
        }

        Ok(())
    }

    fn wait(&self) -> Result<()> {
        TmuxCommand::new()
            .attach_session()
            .target_session(TARGET_SESSION_NAME)
            .output()?;

        Ok(())
    }
}
