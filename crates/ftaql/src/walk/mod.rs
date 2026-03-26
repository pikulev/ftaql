use crossbeam_channel::unbounded;
use ignore::DirEntry;
use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::structs::{FileData, FtaQlConfigResolved};

pub fn walk_and_analyze_files<I, P, V>(
    entries: I,
    repo_path: &String,
    config: &FtaQlConfigResolved,
    process_entry: P,
    is_valid: V,
) -> Vec<FileData>
where
    I: Iterator<Item = Result<DirEntry, ignore::Error>> + Send,
    P: Fn(DirEntry, &String, &FtaQlConfigResolved) -> Option<Vec<FileData>> + Sync + Send,
    V: Fn(&String, &DirEntry, &FtaQlConfigResolved) -> bool + Sync + Send,
{
    let (tx, rx) = unbounded();

    entries
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map_or(false, |ft| ft.is_file()))
        .filter(|entry| is_valid(repo_path, entry, config))
        .for_each_with(tx, |tx, entry| {
            if let Some(data_vec) = process_entry(entry, repo_path, config) {
                tx.send(data_vec).unwrap();
            }
        });

    rx.iter().flatten().collect()
}
