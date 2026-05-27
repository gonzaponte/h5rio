#[cfg(test)]
use tempfile::{TempDir, tempdir};

#[cfg(test)]
pub fn tempfile(stem: &str) -> (TempDir, String) {
    let dir  = tempdir().unwrap();
    let file = dir.path()
                  .join(stem)
                  .to_str()
                  .unwrap()
                  .to_string();
    (dir, file)
}
