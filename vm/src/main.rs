use std::env;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
// use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use qapi::{qmp, Qmp};
use structopt::StructOpt;
use tmux_interface::TmuxCommand;

mod meta;
mod tty_handler;

use meta::{Drive, DriveType, Metadata, NetworkInterface};
use tty_handler::{
    manual::Manual,
    stdout::Stdout,
    tmux::{Tmux, TARGET_SESSION_NAME},
    QemuHandler,
};

pub(crate) type Result<T, E = Box<dyn Error + Send + Sync + 'static>> = core::result::Result<T, E>;

const NETWORK_UP: &str = include_str!("network-up.sh");
const NETWORK_DOWN: &str = include_str!("network-down.sh");

fn validate_is_file(path: String) -> Result<(), String> {
    if Path::new(&path).is_file() {
        return Ok(());
    }

    Err(format!("'{}' was not a file", path))
}

#[derive(Debug, StructOpt)]
struct Args {
    /// The path to the metadata of the machine being emulated
    #[structopt(required = true, validator = validate_is_file)]
    meta_file: PathBuf,
    /// Whether or not to run in non-interactive mode (will print everything to stdout and will not
    /// accept any input)
    #[structopt(long, conflicts_with = "manual")]
    non_interactive: bool,
    /// Whether or not to manually set up screen sessions
    #[structopt(long, conflicts_with = "non-interactive")]
    manual: bool,
}

fn main() -> Result<()> {
    let args = Args::from_args();
    let non_interactive = args.non_interactive || !atty::is(atty::Stream::Stdout);
    let meta_file = fs::canonicalize(args.meta_file)?;

    let reader = BufReader::new(File::open(&meta_file)?);
    let meta: Metadata = serde_json::from_reader(reader)?;

    let temp_dir = tempfile::tempdir()?;
    let sock_path = temp_dir.path().join("qmp-sock");
    let handler: Box<dyn QemuHandler> = if non_interactive {
        Box::new(Stdout {
            monitor: String::from("mon0"),
            serials: vec![String::from("ttyS0"), String::from("ttyS1")],
        })
    } else if args.manual {
        Box::new(Manual {
            monitor: String::from("mon0"),
            serials: vec![String::from("ttyS0"), String::from("ttyS1")],
        })
    } else {
        Box::new(Tmux {
            monitor: String::from("mon0"),
            serials: vec![String::from("ttyS0"), String::from("ttyS1")],
        })
    };

    let mut cmd = Command::new("qemu-kvm");
    cmd.stdin(Stdio::null());

    if !non_interactive {
        cmd.stderr(Stdio::null());
        cmd.stdout(Stdio::null());
    }

    // don't start emulation immediately, to allow us time to set up
    cmd.arg("-S");

    cmd.arg("-nographic");
    cmd.args(["-machine", "q35,smm=on"]);
    cmd.args(["-m", "16G"]);
    cmd.args(["-cpu", "max"]);

    cmd.args(handler.qemu_args());

    cmd.args(["-serial", "chardev:ttyS0"]);
    cmd.args(["-serial", "chardev:ttyS1"]);
    cmd.args(["-mon", "chardev=mon0"]);
    cmd.args(["-msg", "timestamp=on"]);
    cmd.args([
        "-qmp",
        &format!("unix:{},server=on,wait=off", sock_path.display()),
    ]);

    cmd.args(self::metadata(&meta_file, &temp_dir)?);
    cmd.args(self::interfaces(meta.network.interfaces, &temp_dir)?);
    cmd.args(self::drives(meta.specs.drives, &temp_dir)?);
    cmd.args(self::uefi(meta.specs.features.uefi, &temp_dir)?);

    let mut child = cmd.current_dir(&temp_dir.path()).spawn()?;

    // TODO: ^C handler that quits qemu so we can exit gracefully and clean up tempfiles
    // FIXME: this causes a panic on clean exits because we clean it up before it can clean itself up
    // maybe we can just kill the child process by its pid? `Child` has a `id()` func that will hand it to us
    // ctrlc::set_handler(move || {
    //     fs::remove_dir_all(&temp_path).expect("failed to clean up temp_dir");
    // })?;

    while !sock_path.exists() {}

    let stream = UnixStream::connect(sock_path).expect("Failed to connect to QMP socket");
    let mut qmp = Qmp::from_stream(&stream);

    qmp.handshake().expect("Failed to handshake with QMP");

    handler.setup(&mut qmp).map_err(|e| {
        format!(
            "Failed to execute handler setup associated with '{:?}': {:?}",
            handler, e
        )
    })?;

    qmp.execute(&qmp::cont {})
        .expect("Failed to begin VM execution");

    // Attaching can't be done in the handler's setup() because that function won't return until
    // tmux exits, thus preventing the VM from being continued above.
    if !(non_interactive || args.manual) {
        TmuxCommand::new()
            .attach_session()
            .target_session(TARGET_SESSION_NAME)
            .output()?;
    }

    child.wait()?;

    Ok(())
}

fn metadata(meta_file: &Path, temp_dir: &tempfile::TempDir) -> Result<Vec<String>> {
    let metadata_api_dir = temp_dir.path().join("metadata-api");

    fs::create_dir_all(&metadata_api_dir)?;
    fs::copy(&meta_file, metadata_api_dir.join("metadata.json"))?;

    let args = vec![
        "-virtfs".to_string(),
        format!("local,id=metadata_api_stub,security_model=none,path={},readonly=on,mount_tag=metadata_api_stub", metadata_api_dir.display()),
    ];

    Ok(args)
}

fn interfaces(
    interfaces: Vec<NetworkInterface>,
    temp_dir: &tempfile::TempDir,
) -> Result<Vec<String>> {
    let mut args = vec![];

    let network_up = temp_dir.path().join("network-up.sh");
    let network_down = temp_dir.path().join("network-down.sh");
    let mut network_up_file = OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(&network_up)?;
    let mut network_down_file = OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(&network_down)?;

    write!(network_up_file, "{}", NETWORK_UP)?;
    write!(network_down_file, "{}", NETWORK_DOWN)?;

    for (i, interface) in interfaces.iter().enumerate() {
        let netdev = format!("netdev{}", i);
        let mac = &interface.mac;

        args.extend([
            "-netdev".to_string(),
            format!(
                "tap,id={},script={},downscript={}",
                netdev,
                network_up.display(),
                network_down.display()
            ),
        ]);
        args.extend([
            "-device".to_string(),
            format!("virtio-net-pci,netdev={},mac={}", netdev, mac),
        ]);
    }

    Ok(args)
}

fn drives(drives: Vec<Drive>, temp_dir: &tempfile::TempDir) -> Result<Vec<String>> {
    let mut args = vec![
        "-device".to_string(),
        "virtio-scsi-pci,id=scsi0".to_string(),
    ];

    for drive in drives {
        // qemu-img doesn't like the "B" suffix (e.g. in GB)
        let size = drive.size.trim_end_matches('B');
        let count = drive.count;
        let drive_type = &drive.ty;
        let category = &drive.category;

        for ctr in 0..count {
            let outfile = format!("{}/{}-{}.img", temp_dir.path().display(), category, ctr);
            let ident = format!("{}{}", category, ctr);
            let dev_ident = format!("dev{}", ident);

            Command::new("qemu-img")
                .args(["create", "-f", "qcow2", &outfile, size])
                .status()?;

            args.extend([
                "-drive".to_string(),
                format!("file={},if=none,id={}", outfile, ident),
            ]);

            match drive_type {
                DriveType::Ssd => {
                    args.extend([
                        "-device".to_string(),
                        format!("scsi-hd,drive={},id={}", ident, dev_ident),
                    ]);
                    args.extend([
                        "-set".to_string(),
                        format!("device.{}.rotation_rate=1", dev_ident),
                    ]);
                }
                DriveType::Hdd => {
                    args.extend([
                        "-device".to_string(),
                        format!("scsi-hd,drive={},id={}", ident, dev_ident),
                    ]);
                }
                DriveType::Nvme => {
                    args.extend([
                        "-device".to_string(),
                        format!("nvme,drive={},serial={}", ident, ident),
                    ]);
                }
            }
        }
    }

    Ok(args)
}

fn uefi(uefi: bool, temp_dir: &tempfile::TempDir) -> Result<Vec<String>> {
    let mut args = vec![];

    if uefi {
        let ovmf_orig = env::var("OVMF_PATH").map_err(|_| {
            "OVMF_PATH was not present, but is required due to UEFI booting".to_string()
        })?;
        let ovmf_code = temp_dir.path().join("OVMF_CODE.fd");
        let ovmf_vars = temp_dir.path().join("OVMF_VARS.fd");

        fs::copy(format!("{}/FV/OVMF_CODE.fd", ovmf_orig), &ovmf_code)?;
        fs::copy(format!("{}/FV/OVMF_VARS.fd", ovmf_orig), &ovmf_vars)?;

        // TODO: this means efi vars likely won't be saved... do we care?
        // fs::set_permissions(&ovmf_vars, fs::Permissions::from_mode(0o644))?;

        args.extend([
            "-drive".to_string(),
            format!(
                "if=pflash,format=raw,file={},readonly=on",
                ovmf_code.display()
            ),
        ]);
        args.extend([
            "-drive".to_string(),
            format!("if=pflash,format=raw,file={}", ovmf_vars.display()),
        ]);
    }

    Ok(args)
}
