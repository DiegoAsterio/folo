use criterion::{criterion_group, criterion_main, Criterion};
use folo::criterion::FoloAdapter;
use std::path::{Path, PathBuf};

criterion_group!(benches, file_io, scan_many_files);
criterion_main!(benches);

const FILE_SIZE: usize = 10 * 1024 * 1024;
const FILE_PATH: &str = "testdata.bin";
const PARALLEL_READS: usize = 10;

fn file_io(c: &mut Criterion) {
    let tokio = tokio::runtime::Builder::new_multi_thread().build().unwrap();

    // Create a large testdata.bin file, overwriting if it exists.
    let testdata = std::iter::repeat(0u8).take(FILE_SIZE).collect::<Vec<_>>();
    std::fs::write(FILE_PATH, &testdata).unwrap();

    let mut group = c.benchmark_group("file_io");

    group.bench_function("folo_read_file_to_vec_one", |b| {
        b.to_async(FoloAdapter::default()).iter(|| {
            folo::rt::spawn_on_any(|| async {
                let file = folo::fs::read(FILE_PATH).await.unwrap();
                assert_eq!(file.len(), FILE_SIZE);
            })
        });
    });

    group.bench_function("tokio_read_file_to_vec_one", |b| {
        b.to_async(&tokio).iter(|| {
            tokio::task::spawn(async {
                let file = tokio::fs::read(FILE_PATH).await.unwrap();
                assert_eq!(file.len(), FILE_SIZE);
            })
        });
    });

    group.bench_function("folo_read_file_to_vec_many", |b| {
        b.to_async(FoloAdapter::default()).iter(|| {
            folo::rt::spawn_on_any(|| async {
                let tasks = (0..PARALLEL_READS)
                    .map(|_| {
                        folo::rt::spawn(async {
                            let file = folo::fs::read(FILE_PATH).await.unwrap();
                            assert_eq!(file.len(), FILE_SIZE);
                        })
                    })
                    .collect::<Vec<_>>();

                for task in tasks {
                    task.await;
                }
            })
        });
    });

    group.bench_function("tokio_read_file_to_vec_many", |b| {
        b.to_async(&tokio).iter(|| {
            tokio::task::spawn(async {
                let tasks = (0..PARALLEL_READS)
                    .map(|_| {
                        tokio::task::spawn(async {
                            let file = tokio::fs::read(FILE_PATH).await.unwrap();
                            assert_eq!(file.len(), FILE_SIZE);
                        })
                    })
                    .collect::<Vec<_>>();

                for task in tasks {
                    _ = task.await;
                }
            })
        });
    });

    group.finish();

    // Delete our test data file.
    std::fs::remove_file(FILE_PATH).unwrap();
}

const SCAN_PATH: &str = "c:\\Source\\Oss\\folo";

// We read in every file in the target directory, recursively, concurrently.
fn scan_many_files(c: &mut Criterion) {
    // First make the list of files in advance - we want a vec of all the files in SCAN_PATH, recursive.
    let files = generate_file_list(SCAN_PATH);
    println!("Found {} files to scan.", files.len());

    // We use multithreaded mode for each, distributing the files across the processors for maximum
    // system-wide stress. Presumably this will lead to all threads being heavily used and stress
    // the I/O capabilities of the entire system (if a sufficiently rich input directory is used).
    let tokio = tokio::runtime::Builder::new_multi_thread().build().unwrap();

    let mut group = c.benchmark_group("scan_many_files");

    group.bench_function("folo_scan_many_files", |b| {
        b.to_async(FoloAdapter::default()).iter_batched(
            || files.clone(),
            |files| {
                folo::rt::spawn_on_any(move || async move {
                    let tasks = files
                        .iter()
                        .cloned()
                        .map(|file| {
                            folo::rt::spawn_on_any(|| async {
                                let _ = folo::fs::read(file).await;
                            })
                        })
                        .collect::<Vec<_>>();

                    for task in tasks {
                        task.await;
                    }
                })
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("tokio_scan_many_files", |b| {
        b.to_async(&tokio).iter_batched(
            || files.clone(),
            |files| {
                tokio::task::spawn(async move {
                    let tasks = files
                        .iter()
                        .cloned()
                        .map(|file| {
                            tokio::task::spawn(async {
                                let _ = tokio::fs::read(file).await;
                            })
                        })
                        .collect::<Vec<_>>();

                    for task in tasks {
                        _ = task.await;
                    }
                })
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// Generate a list of all files in SCAN_PATH and all subdirectories recursively,
/// returning a boxed slice with their absolute paths.
fn generate_file_list(path: impl AsRef<Path>) -> Box<[PathBuf]> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let sub_files = generate_file_list(path);
            files.extend_from_slice(&sub_files);
        } else {
            files.push(path);
        }
    }

    files.into_boxed_slice()
}
