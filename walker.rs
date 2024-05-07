use std::path::PathBuf;
use walkdir::WalkDir;

pub fn find_ts_files(
    path: PathBuf,
    exclude: Vec<String>,
) -> Result<impl Iterator<Item = walkdir::DirEntry>, walkdir::Error> {
    Ok(WalkDir::new(&path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .metadata()
                .ok()
                .map(|metadata| metadata.is_file())
                .unwrap_or(false)
        })
        .filter(move |entry| {
            entry
                .path()
                .to_str()
                .map(|path| {
                    // Remove any excluded paths
                    if exclude.iter().any(|exclude| path.contains(exclude)) {
                        false
                    } else {
                        // Filter out all files with extensions other than `ts` or `tsx` or `js` or `jsx`
                        if entry.path().extension().map_or(false, |ext| {
                            ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx"
                        }) {
                            true
                        } else {
                            false
                        }
                    }
                })
                .unwrap_or(false)
        }))
}
