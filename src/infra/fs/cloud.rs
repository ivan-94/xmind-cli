use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_MATERIALIZATION_TIMEOUT: Duration = Duration::from_secs(120);
const MATERIALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct CloudFileResolver {
    timeout: Duration,
}

impl Default for CloudFileResolver {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_MATERIALIZATION_TIMEOUT,
        }
    }
}

impl CloudFileResolver {
    pub fn resolve_workbook_path(&self, path: &Path) -> Result<PathBuf, CloudFileError> {
        let original_path = path.to_path_buf();
        let logical_path =
            logical_path_from_icloud_placeholder(path).unwrap_or_else(|| path.to_path_buf());

        if logical_path.exists() {
            return Ok(logical_path);
        }

        let Some(placeholder_path) = hidden_icloud_placeholder_path(&logical_path) else {
            return Ok(logical_path);
        };
        if !placeholder_path.exists() {
            return Ok(logical_path);
        }

        if cfg!(target_os = "macos") {
            let attempts = self.materialize(&logical_path);
            if logical_path.exists() {
                Ok(logical_path)
            } else {
                Err(CloudFileError {
                    original_path,
                    logical_path,
                    placeholder_path,
                    attempts,
                })
            }
        } else {
            Ok(logical_path)
        }
    }

    fn materialize(&self, logical_path: &Path) -> Vec<CloudMaterializationAttempt> {
        let mut attempts = Vec::new();

        let fileprovider_attempt =
            run_cloud_command("fileproviderctl", "materialize", logical_path);
        let fileprovider_succeeded = fileprovider_attempt.succeeded();
        attempts.push(fileprovider_attempt);
        if fileprovider_succeeded && wait_for_path(logical_path, self.timeout) {
            return attempts;
        }

        let brctl_attempt = run_cloud_command("brctl", "download", logical_path);
        let brctl_succeeded = brctl_attempt.succeeded();
        attempts.push(brctl_attempt);
        if brctl_succeeded {
            let _ = wait_for_path(logical_path, self.timeout);
        }

        attempts
    }
}

#[derive(Debug, thiserror::Error)]
#[error("cloud workbook could not be downloaded")]
pub struct CloudFileError {
    pub original_path: PathBuf,
    pub logical_path: PathBuf,
    pub placeholder_path: PathBuf,
    pub attempts: Vec<CloudMaterializationAttempt>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudMaterializationAttempt {
    pub tool: String,
    pub subcommand: String,
    pub status: Option<i32>,
    pub stderr: String,
    pub error: Option<String>,
}

impl CloudMaterializationAttempt {
    fn succeeded(&self) -> bool {
        self.status == Some(0) && self.error.is_none()
    }
}

fn run_cloud_command(
    tool: &'static str,
    subcommand: &'static str,
    path: &Path,
) -> CloudMaterializationAttempt {
    match Command::new(tool).arg(subcommand).arg(path).output() {
        Ok(output) => CloudMaterializationAttempt {
            tool: tool.to_owned(),
            subcommand: subcommand.to_owned(),
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            error: None,
        },
        Err(error) => CloudMaterializationAttempt {
            tool: tool.to_owned(),
            subcommand: subcommand.to_owned(),
            status: None,
            stderr: String::new(),
            error: Some(error.to_string()),
        },
    }
}

fn wait_for_path(path: &Path, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if path.exists() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(MATERIALIZATION_POLL_INTERVAL);
    }
}

fn hidden_icloud_placeholder_path(logical_path: &Path) -> Option<PathBuf> {
    let file_name = logical_path.file_name()?.to_str()?;
    Some(
        logical_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(format!(".{file_name}.icloud")),
    )
}

fn logical_path_from_icloud_placeholder(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?.to_str()?;
    let logical_file_name = file_name.strip_prefix('.')?.strip_suffix(".icloud")?;
    if !logical_file_name.ends_with(".xmind") {
        return None;
    }
    Some(
        path.parent()
            .unwrap_or_else(|| Path::new(""))
            .join(logical_file_name),
    )
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{hidden_icloud_placeholder_path, logical_path_from_icloud_placeholder};

    #[test]
    fn hidden_icloud_placeholder_path_matches_cloud_docs_naming() {
        assert_eq!(
            hidden_icloud_placeholder_path(Path::new("/tmp/roadmap.xmind")),
            Some(PathBuf::from("/tmp/.roadmap.xmind.icloud"))
        );
    }

    #[test]
    fn logical_path_from_icloud_placeholder_strips_hidden_prefix_and_suffix() {
        assert_eq!(
            logical_path_from_icloud_placeholder(Path::new("/tmp/.roadmap.xmind.icloud")),
            Some(PathBuf::from("/tmp/roadmap.xmind"))
        );
    }

    #[test]
    fn non_xmind_icloud_placeholder_is_not_treated_as_workbook() {
        assert_eq!(
            logical_path_from_icloud_placeholder(Path::new("/tmp/.notes.txt.icloud")),
            None
        );
    }
}
