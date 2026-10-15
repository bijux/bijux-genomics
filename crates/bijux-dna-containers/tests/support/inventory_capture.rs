//! Test-owned FIFO acquisition, replacement custody and bounded native child transport.
use anyhow::{ensure, Context, Result};
use std::{
    io::Write,
    os::unix::fs::{FileTypeExt, PermissionsExt},
    path::{Path, PathBuf},
    process::{Child, Command},
    time::Duration,
};

const REQUEST_VARIABLE: &str = "BIJUX_CONTAINER_INVENTORY_WRITER_REQUEST";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum InventoryCapture {
    Valid,
    MalformedJson,
    NonUtf8,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct WriterInputs {
    captured_sha256: String,
    replacement_sha256: String,
    executable_sha256: String,
    executable_mode: u32,
    scenario: InventoryCapture,
}

fn owned_directory(request_path: &Path, suffix: &Path) -> Result<PathBuf> {
    ensure!(
        request_path.is_absolute() && std::fs::symlink_metadata(request_path)?.is_file(),
        "writer request must be an absolute regular file"
    );
    let directory = request_path.parent().context("writer directory")?.canonicalize()?;
    ensure!(
        request_path == directory.join("inventory-writer-request.json")
            && directory.ends_with(suffix),
        "writer request must belong to its canonical owned fixture"
    );
    Ok(directory)
}

fn verify_executable(path: &Path, inputs: &WriterInputs) -> Result<()> {
    ensure!(
        std::fs::metadata(path)?.permissions().mode() == inputs.executable_mode
            && inputs.executable_mode & 0o111 != 0
            && bijux_dna_infra::hash_file_sha256(path)? == inputs.executable_sha256,
        "writer executable identity changed"
    );
    Ok(())
}

pub(crate) fn write_inventory_child_request(
    case: &str,
    suffix: &Path,
    validate_capture: impl FnOnce(&Path, &Path, &[u8], InventoryCapture) -> Result<()>,
) -> Result<bool> {
    let Some(request_path) = std::env::var_os(REQUEST_VARIABLE) else {
        return Ok(false);
    };
    ensure!(
        std::env::args().skip(1).collect::<Vec<_>>() == [case, "--exact", "--nocapture"],
        "writer must be its exact isolated case"
    );
    let request_path = PathBuf::from(request_path);
    let directory = owned_directory(&request_path, suffix)?;
    let inputs: WriterInputs = serde_json::from_slice(&std::fs::read(&request_path)?)?;
    let captured_path = directory.join("inventory-captured.json");
    let replacement_path = directory.join("inventory-replacement.json");
    let inventory_path = directory.join("image-inventory.json");
    let receipt_path = directory.join("inventory-writer-receipt.json");
    for path in [&captured_path, &replacement_path] {
        ensure!(std::fs::symlink_metadata(path)?.is_file(), "writer source must be a regular file");
    }
    ensure!(
        std::fs::symlink_metadata(&inventory_path)?.file_type().is_fifo()
            && !receipt_path.try_exists()?,
        "writer requires its FIFO and fresh receipt"
    );
    let captured = std::fs::read(&captured_path)?;
    let replacement = std::fs::read(&replacement_path)?;
    ensure!(
        bijux_dna_infra::hash_bytes_sha256(&captured) == inputs.captured_sha256
            && bijux_dna_infra::hash_bytes_sha256(&replacement) == inputs.replacement_sha256,
        "writer source bytes changed"
    );
    verify_executable(&std::env::current_exe()?, &inputs)?;
    validate_capture(&captured_path, &replacement_path, &captured, inputs.scenario)?;
    // A blocked FIFO rendezvous is bounded even while the parent is inside its real reader.
    let _watchdog = std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(15));
        std::process::exit(124);
    });
    let mut fifo = std::fs::OpenOptions::new().write(true).open(&inventory_path)?;
    fifo.write_all(&captured)?;
    std::fs::rename(&replacement_path, &inventory_path)?;
    assert_eq!(std::fs::read(&inventory_path)?, replacement);
    // Publishing this typed custody receipt precedes the descriptor close and reader EOF.
    bijux_dna_infra::atomic_create_json(&receipt_path, &inputs)?;
    drop(fifo);
    Ok(true)
}

pub(crate) struct InventoryWriter {
    child: Child,
    directory: PathBuf,
    inputs: WriterInputs,
    executable: PathBuf,
}

impl InventoryWriter {
    pub(crate) fn start(
        inventory_path: &Path,
        captured: &[u8],
        replacement: &[u8],
        scenario: InventoryCapture,
        case: &str,
    ) -> Result<Self> {
        let directory = inventory_path.parent().context("inventory directory")?.canonicalize()?;
        ensure!(
            inventory_path.file_name() == Some(std::ffi::OsStr::new("image-inventory.json")),
            "owned inventory source path"
        );
        let captured_path = directory.join("inventory-captured.json");
        let replacement_path = directory.join("inventory-replacement.json");
        std::fs::write(&captured_path, captured)?;
        std::fs::write(&replacement_path, replacement)?;
        let executable = std::env::current_exe()?.canonicalize()?;
        let inputs = WriterInputs {
            captured_sha256: bijux_dna_infra::hash_bytes_sha256(captured),
            replacement_sha256: bijux_dna_infra::hash_file_sha256(&replacement_path)?,
            executable_sha256: bijux_dna_infra::hash_file_sha256(&executable)?,
            executable_mode: std::fs::metadata(&executable)?.permissions().mode(),
            scenario,
        };
        let request_path = directory.join("inventory-writer-request.json");
        bijux_dna_infra::atomic_create_json(&request_path, &inputs)?;
        let mkfifo =
            std::env::split_paths(&std::env::var_os("PATH").context("mkfifo search path")?)
                .map(|root| root.join("mkfifo"))
                .find(|path| path.is_file())
                .context("native mkfifo provider unavailable")?
                .canonicalize()?;
        let provider_sha256 = bijux_dna_infra::hash_file_sha256(&mkfifo)?;
        let provider_mode = std::fs::metadata(&mkfifo)?.permissions().mode();
        ensure!(provider_mode & 0o111 != 0, "mkfifo provider must be executable");
        let inventory_path = directory.join("image-inventory.json");
        std::fs::remove_file(&inventory_path)?;
        let result = bijux_dna_process::capture(
            Command::new(&mkfifo).arg(&inventory_path),
            bijux_dna_process::CaptureLimits {
                timeout: Duration::from_secs(20),
                max_output_bytes: 4096,
            },
            "inventory FIFO setup",
        )?;
        ensure!(
            !result.timed_out && !result.output_limit_exceeded && result.status.success(),
            "native FIFO setup failed: {result:?}"
        );
        ensure!(
            bijux_dna_infra::hash_file_sha256(&mkfifo)? == provider_sha256
                && std::fs::metadata(&mkfifo)?.permissions().mode() == provider_mode,
            "mkfifo provider identity changed"
        );
        bijux_dna_infra::atomic_create_json(
            &directory.join("inventory-mkfifo-provider.json"),
            &serde_json::json!({"path":mkfifo,"sha256":provider_sha256,"mode":provider_mode}),
        )?;
        let log = std::fs::File::create(directory.join("inventory-writer.log"))?;
        let mut command = Command::new(std::env::current_exe()?);
        command
            .args([case, "--exact", "--nocapture"])
            .env(REQUEST_VARIABLE, &request_path)
            .stdin(std::process::Stdio::null())
            .stdout(log.try_clone()?)
            .stderr(log);
        bijux_dna_process::isolate_process_group(&mut command);
        let child = command.spawn()?;
        Ok(Self { child, directory, inputs, executable })
    }

    pub(crate) fn finish(&mut self) -> Result<()> {
        let outcome = bijux_dna_process::wait_for_status(
            &mut self.child,
            Duration::from_secs(20),
            "inventory FIFO writer",
        )?;
        ensure!(
            !outcome.timed_out && outcome.status.success(),
            "inventory writer failed: {outcome:?}"
        );
        let receipt: WriterInputs = serde_json::from_slice(&std::fs::read(
            self.directory.join("inventory-writer-receipt.json"),
        )?)?;
        assert_eq!(receipt, self.inputs);
        verify_executable(&self.executable, &self.inputs)
    }
}

impl Drop for InventoryWriter {
    fn drop(&mut self) {
        // Shared process cancellation reaps the isolated group; direct cleanup covers errors.
        let _ = bijux_dna_process::wait_for_status(
            &mut self.child,
            Duration::ZERO,
            "abandoned inventory writer",
        );
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
