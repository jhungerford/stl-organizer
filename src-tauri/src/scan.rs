use std::{fs::{self, File}, path::Path};


/// The commands module contains thin wrappers around the functions in the scan module
/// to make them available to Tauri.
pub mod commands {
    use super::*;
    use tauri::{ command, State };

    #[command]
    pub fn scan_start() {
        
    }

    #[command]
    pub fn scan_get() {

    }

    #[command]
    pub fn scan_progress() {

    }
}



/// A scan looks for 3D printing files in the directories configured in settings,
/// and ScanState tracks the state the 
enum ScanState<'a> {
    /// StartDirs is the initial state before a scan starts - the scanner will recursively
    /// expand the directories into a list of potential files, and move the scanner to the 
    /// ScanningFiles state.
    StartDirs(Vec<File>),
    /// ScanningFiles 
    ScanningFiles(Vec<&'a Path>),
    /// ScanIdle indicates that a scan isn't currently running.
    ScanIdle,
}

/// FileType 
enum FileType {
    ThingiverseZip,
    Stl,
}

// TODO: download location (browser history plugin?), thingiverse link, tags, readme
/// FileInfo contains details about a 3D printing file.
struct FileInfo<'a> {
    file_type: FileType,
    name: &'a str,
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
        Some(".stl") => scan_stl(path),
        Some(".zip") => scan_zip(path),
        _ => None,
    }
}

/// Scans the given stl file, returning information about the file.
fn scan_stl(path: &Path) -> Option<FileInfo> {
    unimplemented!()
}

fn scan_zip(path: &Path) -> Option<FileInfo> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_irrelevant() {
        let path= Path::new("icons/icon.ico");
        assert!(scan(path).is_none());
    }

    #[test]
    fn test_scan_stl() {
        let path= Path::new("test/resources/Benchy.zip");
        assert!(scan(path).is_some());
    }

    #[test]
    fn test_scan_thingiverse_archive() {
        let path= Path::new("test/resources/freighterbenchy-v2.stl");
        assert!(scan(path).is_some());
    }
}
