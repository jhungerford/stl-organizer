use std::{path::{Path, PathBuf}, sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}}, time::SystemTime};

use tokio::{runtime::Runtime, task::{self, JoinHandle}};

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
    tasks: Arc<Mutex<Vec<JoinHandle<()>>>>,
    progress: Arc<ScanProgress>,
}

impl<T: ConnectionManager> Scanner<T> {
    /// Creates a new Scanner that will scan for 3D printing files in the given directories.
    pub fn new(conn_manager: Arc<T>) -> Result<Scanner<T>, AppError> {
        let tasks = Arc::new(Mutex::new(vec![]));
        let progress = Arc::new(ScanProgress::new());

        Ok(Scanner { conn_manager, tasks, progress })
    }

    /// Starts a full scan of the directories configured in the settings.  The scan will run in the background.
    #[tokio::main]
    pub async fn scan(&self, settings: &Settings<T>) -> Result<(), AppError> {
        self.scan_settings(settings).await?; // TODO: want scan to return, but tasks to continue running in the background.

        Ok(())
    }

    async fn scan_settings(&self, settings: &Settings<T>) -> Result<(), AppError> {
        let dirs = settings.list_dirs()?;

        for dir in dirs {
            let task = Path::new(&dir).to_path_buf();
            let join_handle = tokio::spawn(scan_dir(task, self.progress.clone()));
            self.tasks.lock().unwrap().push(join_handle);
        }

        Ok(())
    }

    /// Blocks until the scan is complete.  Useful for testing, but the application shouldn't need to block on the scan.
    pub async fn join(&mut self) -> Result<(), AppError> {
        while let Some(task) = self.tasks.lock()?.pop() {
            task.await?;
        }

        Ok(())
    } 
}

pub struct ScanProgress {
    num_scan_dir_tasks: AtomicUsize,
    total_tasks: AtomicUsize,
    complete_tasks: AtomicUsize,
    start: SystemTime,
}

impl ScanProgress {
    /// Creates a new ScanProgress with 0 running tasks and a start time of now.
    fn new() -> Self {
        ScanProgress {
            num_scan_dir_tasks: AtomicUsize::new(0),
            total_tasks: AtomicUsize::new(0),
            complete_tasks: AtomicUsize::new(0),
            start: SystemTime::now()
        }
    }

    fn add_scan_dir(&self) {
        self.total_tasks.fetch_add(1, Ordering::Relaxed); // TODO: ordering, especially for methods that touch multiple counters.
        unimplemented!()
    }

    fn done_scan_dir(&self) {
        unimplemented!()
    }

    fn add_task(&self) {
        unimplemented!()
    }

    fn done_task(&self) {
        unimplemented!()
    }
}

// TODO: Scan tasks - expand directory, parse file, thingiverse lookup, browser downloads search, etc.

async fn scan_dir(dir: PathBuf, progress: Arc<ScanProgress>) {
    unimplemented!()
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 5)]
    async fn test_full_scan() -> Result<(), AppError> {
        let conn_manager = Arc::new(InMemoryConnectionManager::new("test_full_scan")?);
        conn_manager.migrate()?;

        let settings = Settings::new(conn_manager.clone());
        settings.add_dir("test/resources")?;

        let mut scanner = Scanner::new(conn_manager.clone())?;

        tokio::try_join!(scanner.scan_settings(&settings))?;
        tokio::try_join!(scanner.join())?;

        // TODO: assert side effects.

        Ok(())
    }
}
