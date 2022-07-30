use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

/// Get the real path like Python os.path.realpath().
/// 
/// This works even if the path does not exist.
/// 
pub fn realpath<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    match path.canonicalize() {
        Ok(v) => {
            let s = v.to_string_lossy();
            if s.starts_with(r#"\\?\"#) {
                (&s[4..]).into()
            } else {
                s.into_owned().into()
            }
        }
        _ => path.absolutize().map_or(path.to_owned(), |x| {
            x.as_ref().to_string_lossy().into_owned().into()
        }),
    }
}
