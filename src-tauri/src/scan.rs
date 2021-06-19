use std::{path::{Path, PathBuf}, sync::{Arc, Mutex}};

use crate::{db::ConnectionManager, error::AppError, settings::Settings};

/// The commands module contains thin wrappers around the functions in the scan module
/// to make them available to Tauri.
pub mod commands {
    use crate::db::InMemoryConnectionManager;

    use super::*;
    use tauri::{ command, State };

    #[command]
    pub fn scan_start(
        scanner: State<Arc<Mutex<Scanner<InMemoryConnectionManager>>>>,
        settings: State<Settings<InMemoryConnectionManager>>,
    ) -> Result<(), AppError> {
        scanner.inner().lock()?.scan(&settings)
    }

    #[command]
    pub fn scan_progress(_scanner: State<Mutex<Scanner<InMemoryConnectionManager>>>) {
        unimplemented!()
    }
}

/// Scanner executes the tasks in the task list, scanning directories and gathering information about files.
pub struct Scanner<T: ConnectionManager> {
    conn_manager: Arc<T>,
    task_list: Mutex<Vec<ScanTask>>,
}

impl<T: ConnectionManager> Scanner<T> {
    /// Creates a new Scanner that will scan for 3D printing files in the given directories.
    pub fn new(conn_manager: Arc<T>) -> Scanner<T> {
        Scanner { 
            conn_manager, 
            task_list: Mutex::new(vec![]),
        }
    }

    /// Runs a full scan of the directories configured in the settings.
    pub fn scan(&mut self, settings: &Settings<T>) -> Result<(), AppError> {
        let dirs = settings.list_dirs()?;
        let tasks = self.task_list.get_mut()?;

        for dir in dirs {
            let path = Path::new(&dir);
            if path.is_dir() {
                tasks.push(ScanTask::ScanDir(path.to_path_buf()));
            }
        }

        Ok(())
    }
}

// TODO: Scan tasks - expand directory, parse file, thingiverse lookup, browser downloads search, etc.
enum ScanTask {
    Init,
    ScanDir(PathBuf),
    ScanStl(PathBuf),
    ScanZip(PathBuf),
}

// TODO: expand zip files into a tree

/// FileType
#[derive(Debug, Eq, PartialEq)]
enum FileType {
    ThingiverseZip,
    OtherZip,
    Stl,
    Image,
    Readme,
}

// TODO: download location (browser history plugin?), thingiverse link, tags, readme
/// FileInfo contains details about a 3D printing file.
struct FileInfo<'a> {
    file_type: FileType,
    path: &'a Path,
}

fn scan(path: &Path) -> Option<FileInfo> {
    // Check the file's metadata
    let meta = path.metadata();
    if meta.is_err() {
        return None;
    }

    // 3D printing files are only files - don't index directories.
    if !meta.unwrap().is_file() {
        return None;
    }

    // Classify the file based on its extension.
    if path.extension().is_none() {
        return None;
    }

    match path.extension().unwrap().to_str() {
        Some("stl") => scan_stl(path),
        Some("zip") => scan_zip(path),
        _ => None,
    }
}

/// Scans the given stl file, returning information about the file.
fn scan_stl<'a> (path: &'a Path) -> Option<FileInfo<'a>> {
    Some(FileInfo {
        file_type: FileType::Stl,
        path
    })
}

/// Scans the given zip file, returning information about the file.
/// A thingiverse zip has files/ and images/ directories, a LICENSE.txt file, and a README.txt
/// file containing a title like 'NAME by AUTHOR on Thingiverse: https://www.thingiverse.com/thing:1234'.
/// Not all zip files are relevant, and not all thingiverse zip files fit this format.
fn scan_zip(_path: &Path) -> Option<FileInfo> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use crate::db::InMemoryConnectionManager;

    use super::*;

    #[test]
    fn test_scan_irrelevant() {
        let path= Path::new("icons/icon.ico");
        assert!(scan(path).is_none());
    }

    #[test]
    fn test_scan_stl() {
        let file_name = "test/resources/freighterbenchy-v2.stl";
        let path= Path::new(file_name);
        let maybe_scanned = scan(path);

        assert!(maybe_scanned.is_some());

        let scanned = &maybe_scanned.unwrap();
        assert_eq!(FileType::Stl, scanned.file_type);

        assert!(scanned.path.to_str().unwrap().ends_with(file_name));
    }

    #[test]
    fn test_scan_thingiverse_archive() {
        let path= Path::new("test/resources/Benchy.zip");
        let scanned = scan(path);

        assert!(scanned.is_some());
    }

    #[test]
    fn test_full_scan() {
        let conn_manager = Arc::new(InMemoryConnectionManager::new("test_full_scan").unwrap());
        conn_manager.migrate();

        let settings = Settings::new(conn_manager.clone());
        settings.add_dir("test/resources");

        let mut scanner = Scanner::new(conn_manager.clone());

        scanner.scan(&settings);
    }
}
