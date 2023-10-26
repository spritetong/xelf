use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, Read, Write},
};

/// Get the real path like Python os.path.realpath().
///
/// This works even if the path does not exist.
///
pub fn realpath<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    match path.canonicalize() {
        Ok(v) => {
            let s = v.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                stripped.into()
            } else {
                s.into_owned().into()
            }
        }
        _ => path.absolutize().map_or(path.to_owned(), |x| {
            x.as_ref().to_string_lossy().into_owned().into()
        }),
    }
}

/// Replace a file if its content is different from the given value.
///
/// If the file does not exist, a new file will be created and filled.
///
pub fn replace_file_if_different<P: AsRef<Path>>(path: P, content: &[u8]) -> io::Result<()> {
    let path = path.as_ref();
    let different = match fs::OpenOptions::new().read(true).open(path) {
        Ok(mut f) => {
            let mut buf = Vec::new();
            let _ = f.read_to_end(&mut buf);
            content != buf
        }
        _ => true,
    };
    if different {
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)?;
        f.write_all(content)?;
    }
    Ok(())
}
