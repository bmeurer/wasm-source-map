// Copyright 2019 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::borrow::Cow;

fn is_absolute(p: &str) -> bool {
  let mut bytes = p.bytes();
  let b = match bytes.next() {
    Some(b) => b,
    None => return false,
  };
  match b {
    // Unix paths
    b'/' => true,
    // Windows paths starting with drive letter like `C:`
    b'A'..=b'Z' | b'a'..=b'z' => bytes.next() == Some(b':'),
    // Windows paths starting with `\\` (UNC)
    b'\\' => bytes.next() == Some(b'\\'),
    _ => false,
  }
}

fn strip_prefix<'a>(p: &'a str, prefix: &'static str) -> Option<&'a str> {
  if p.starts_with(prefix) {
    Some(unsafe { p.get_unchecked(prefix.len()..) })
  } else {
    None
  }
}

pub struct Path<'a>(Cow<'a, str>);

impl<'a> Path<'a> {
  pub fn new(s: Cow<'a, str>) -> Self {
    assert!(is_absolute(&s));
    Path(s)
  }

  pub fn push(&mut self, p2: Cow<'a, str>) {
    if is_absolute(&p2) {
      self.0 = p2;
    } else {
      let p1 = self.0.to_mut();
      if p1.starts_with('/') {
        // Assume Unix absolute path, add slash if it's not there yet.
        if !p1.ends_with('/') {
          p1.push('/');
        }
      } else {
        // Assume Windows absolute path, add backslash if not there yet.
        if !p1.ends_with('\\') {
          p1.push('\\');
        }
      }
      p1.push_str(&p2);
    }
  }

  pub fn borrow(&self) -> Path {
    Path(Cow::Borrowed(&self.0))
  }

  pub fn to_uri(&self) -> String {
    let path = &self.0;

    if let Some(path) = strip_prefix(&path, "/rustc/") {
      // TODO: avoid hardcoding this, and instead let users configure
      // path replacements in DevTools UI.
      format!("https://raw.githubusercontent.com/rust-lang/rust/{}", path)
    } else if path.starts_with('/') {
      // Unix-style path
      format!("file://{}", path)
    } else {
      // Windows-style path
      format!("file:///{}", path)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn test_is_absolute_empty() {
    assert!(!is_absolute(""));
  }

  #[test]
  pub fn test_is_absolute_unix() {
    assert!(is_absolute("/"));
    assert!(is_absolute("/sbin"));
    assert!(is_absolute("/home/user"));
    assert!(!is_absolute(""));
    assert!(!is_absolute("usr/local"));
  }

  #[test]
  pub fn test_is_absolute_windows() {
    assert!(is_absolute("a:\\"));
    assert!(is_absolute("A:\\"));
    assert!(is_absolute("c:\\Windows\\System32"));
    assert!(is_absolute("C:\\Windows\\System32"));
    assert!(!is_absolute("\\User"));
    assert!(!is_absolute("User\\Someone Special"));
  }

  #[test]
  pub fn test_path_unix() {
    let mut path = Path::new(Cow::from("/"));
    path.push(Cow::from("etc"));
    path.push(Cow::from("passwd"));
    assert_eq!(path.to_uri(), "file:///etc/passwd");

    let mut path = Path::new(Cow::from("/etc"));
    path.push(Cow::from("passwd"));
    path.push(Cow::from("/etc/hosts"));
    assert_eq!(path.to_uri(), "file:///etc/hosts");
  }

  #[test]
  pub fn test_path_windows() {
    let mut path = Path::new(Cow::from("C:\\"));
    path.push(Cow::from("Windows"));
    path.push(Cow::from("System32"));
    assert_eq!(path.to_uri(), "file:///C:\\Windows\\System32");

    let mut path = Path::new(Cow::from("\\\\"));
    path.push(Cow::from("Server"));
    path.push(Cow::from("Share"));
    assert_eq!(path.to_uri(), "file:///\\\\Server\\Share");

    let mut path = Path::new(Cow::from("a:\\"));
    path.push(Cow::from("Folder"));
    path.push(Cow::from("F:\\Directory\\File.html"));
    assert_eq!(path.to_uri(), "file:///F:\\Directory\\File.html");
  }

  #[test]
  pub fn test_path_rustc() {
    let path = Path::new(Cow::from("/rustc/folder/file.rs"));
    assert_eq!(
      path.to_uri(),
      "https://raw.githubusercontent.com/rust-lang/rust/folder/file.rs"
    );
  }
}
