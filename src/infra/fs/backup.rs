use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct BackupMetadata {
    pub path: PathBuf,
}

#[allow(dead_code)]
pub fn create_backup(
    workbook_path: &Path,
    timestamp: impl AsRef<str>,
) -> Result<BackupMetadata, BackupError> {
    let backup_dir = workbook_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(".xmind-backups");
    create_backup_in_dir(workbook_path, &backup_dir, timestamp)
}

pub fn create_backup_in_dir(
    workbook_path: &Path,
    backup_dir: &Path,
    timestamp: impl AsRef<str>,
) -> Result<BackupMetadata, BackupError> {
    fs::create_dir_all(backup_dir)?;

    let stem = workbook_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("workbook");
    let extension = workbook_path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("xmind");
    let backup_path = backup_dir.join(format!("{stem}.{}.{extension}", timestamp.as_ref()));

    fs::copy(workbook_path, &backup_path)?;

    Ok(BackupMetadata { path: backup_path })
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("backup could not be created: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::create_backup;

    #[test]
    fn create_backup_copies_workbook_to_default_timestamped_path() {
        let temp_dir = tempfile::tempdir().expect("temp dir is created");
        let workbook = temp_dir.path().join("roadmap.xmind");
        fs::write(&workbook, b"xmind-bytes").expect("workbook is written");

        let backup = create_backup(&workbook, "20260522-153000").expect("backup is created");

        assert_eq!(
            backup.path,
            temp_dir
                .path()
                .join(".xmind-backups")
                .join("roadmap.20260522-153000.xmind")
        );
        assert_eq!(
            fs::read(&backup.path).expect("backup is readable"),
            b"xmind-bytes"
        );
        assert_eq!(
            fs::read(&workbook).expect("source remains readable"),
            b"xmind-bytes"
        );
    }
}
