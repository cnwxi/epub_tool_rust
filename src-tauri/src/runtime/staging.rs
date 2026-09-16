//! Stream staging shared by the Android adapter and host-side regression tests.
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static NEXT_IMPORT: AtomicU64 = AtomicU64::new(0);

fn import_directory(root: &Path) -> io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    loop {
        let sequence = NEXT_IMPORT.fetch_add(1, Ordering::Relaxed);
        let directory = root.join(format!(
            "import-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn staged_file_name(source: &str, extension: &str) -> Result<String, String> {
    let extension = extension.trim_start_matches('.');
    if extension.is_empty() || !extension.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("暂存文件缺少有效扩展名。".to_string());
    }
    let source = source.split(['?', '#']).next().unwrap_or(source);
    let decoded = percent_encoding::percent_decode_str(source).decode_utf8_lossy();
    let candidate = decoded
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty() && *name != "." && *name != "..")
        .unwrap_or("input");
    let mut name: String = candidate
        .chars()
        .map(|c| if c == ':' || c.is_control() { '_' } else { c })
        .collect();
    if !name
        .rsplit_once('.')
        .is_some_and(|(_, suffix)| suffix.eq_ignore_ascii_case(extension))
    {
        name.push('.');
        name.push_str(extension);
    }
    Ok(name)
}

pub fn stage_reader(
    root: &Path,
    source: &str,
    extension: &str,
    reader: &mut impl Read,
) -> Result<PathBuf, String> {
    let name = staged_file_name(source, extension)?;
    // Keep task suffixes intact, and never share a destination between imports.
    let directory = import_directory(root).map_err(|e| format!("创建暂存目录失败: {e}"))?;
    let destination = directory.join(name);
    let copied = (|| -> io::Result<()> {
        let mut file = fs::File::create_new(&destination)?;
        io::copy(reader, &mut file)?;
        Ok(())
    })();
    if let Err(error) = copied {
        // This directory belongs exclusively to this failed import.
        let _ = fs::remove_dir_all(&directory);
        return Err(format!("暂存所选文件失败: {error}"));
    }
    Ok(destination)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            Self(import_directory(&std::env::temp_dir()).unwrap())
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn same_named_imports_preserve_bytes_and_task_suffix_without_overwrite() {
        let scratch = Scratch::new();
        let source = "content://provider/document/primary%3ABooks%2F%E4%B9%A6_reformat_epub.epub";
        let first = stage_reader(&scratch.0, source, "epub", &mut Cursor::new(b"first")).unwrap();
        let second = stage_reader(&scratch.0, source, "epub", &mut Cursor::new(b"second")).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            first.file_name(),
            Some(std::ffi::OsStr::new("书_reformat_epub.epub"))
        );
        assert_eq!(first.file_name(), second.file_name());
        assert_eq!(fs::read(first).unwrap(), b"first");
        assert_eq!(fs::read(second).unwrap(), b"second");
    }

    #[test]
    fn uri_names_cannot_escape_staging_and_keep_encoded_punctuation() {
        for (source, expected) in [
            (
                "content://provider/document/..%2F..%2Fbook.EPUB?token=x#ignored",
                "book.EPUB",
            ),
            ("content://provider/document/%2E%2E", "input.epub"),
            ("content://provider/document/book%3F%23.epub", "book?#.epub"),
            ("content://provider/document/book%00.epub", "book_.epub"),
        ] {
            let scratch = Scratch::new();
            let output =
                stage_reader(&scratch.0, source, ".epub", &mut Cursor::new(b"book")).unwrap();
            assert_eq!(output.file_name().unwrap(), expected);
            assert_eq!(output.parent().unwrap().parent().unwrap(), scratch.0);
        }
    }

    #[test]
    fn failed_stream_leaves_no_partial_import() {
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("provider disconnected"))
            }
        }
        let scratch = Scratch::new();
        let mut reader = Cursor::new(b"partial").chain(Broken);
        assert!(stage_reader(
            &scratch.0,
            "content://provider/book.epub",
            "epub",
            &mut reader
        )
        .is_err());
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 0);
    }

    #[test]
    fn invalid_extension_is_rejected_before_creating_an_import() {
        let scratch = Scratch::new();
        for extension in ["", ".", "../epub", "epub/cover"] {
            assert!(
                stage_reader(&scratch.0, "book", extension, &mut Cursor::new(b"book")).is_err()
            );
        }
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 0);
    }
}
