#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::c_void;
    use std::ptr;

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hwnd: *mut c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn GetClipboardData(uformat: u32) -> *mut c_void;
        fn SetClipboardData(uformat: u32, hmem: *mut c_void) -> *mut c_void;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(uflags: u32, dwbytes: usize) -> *mut c_void;
        fn GlobalLock(hmem: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(hmem: *mut c_void) -> i32;
        fn GlobalFree(hmem: *mut c_void) -> *mut c_void;
        fn GlobalSize(hmem: *mut c_void) -> usize;
    }

    pub fn get_text() -> Option<String> {
        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return None;
            }
            let result = (|| {
                let handle = GetClipboardData(CF_UNICODETEXT);
                if handle.is_null() {
                    return None;
                }
                let byte_len = GlobalSize(handle);
                if byte_len < 2 {
                    return None;
                }
                let locked = GlobalLock(handle) as *const u16;
                if locked.is_null() {
                    return None;
                }
                let max_units = byte_len / 2;
                let slice = std::slice::from_raw_parts(locked, max_units);
                let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                let text = String::from_utf16_lossy(&slice[..end]);
                GlobalUnlock(handle);
                Some(text)
            })();
            CloseClipboard();
            result
        }
    }

    pub fn set_text(text: &str) -> bool {
        let mut utf16: Vec<u16> = text.encode_utf16().collect();
        utf16.push(0);
        let byte_len = utf16.len() * std::mem::size_of::<u16>();

        unsafe {
            if OpenClipboard(ptr::null_mut()) == 0 {
                return false;
            }
            if EmptyClipboard() == 0 {
                CloseClipboard();
                return false;
            }
            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
            if hmem.is_null() {
                CloseClipboard();
                return false;
            }
            let locked = GlobalLock(hmem) as *mut u16;
            if locked.is_null() {
                GlobalFree(hmem);
                CloseClipboard();
                return false;
            }
            ptr::copy_nonoverlapping(utf16.as_ptr(), locked, utf16.len());
            GlobalUnlock(hmem);
            let res = SetClipboardData(CF_UNICODETEXT, hmem);
            if res.is_null() {
                GlobalFree(hmem);
                CloseClipboard();
                return false;
            }
            CloseClipboard();
            true
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod imp {
    use std::io::Write;
    use std::process::{Command, Stdio};

    pub fn get_text() -> Option<String> {
        #[cfg(target_os = "macos")]
        let cmds: &[(&str, &[&str])] = &[("pbpaste", &[])];
        #[cfg(not(target_os = "macos"))]
        let cmds: &[(&str, &[&str])] = &[
            ("wl-paste", &["-n"]),
            ("xclip", &["-o", "-selection", "clipboard"]),
        ];

        for (bin, args) in cmds {
            if let Ok(out) = Command::new(bin).args(*args).output() {
                if out.status.success() {
                    return Some(String::from_utf8_lossy(&out.stdout).to_string());
                }
            }
        }
        None
    }

    pub fn set_text(text: &str) -> bool {
        #[cfg(target_os = "macos")]
        let cmds: &[(&str, &[&str])] = &[("pbcopy", &[])];
        #[cfg(not(target_os = "macos"))]
        let cmds: &[(&str, &[&str])] = &[
            ("wl-copy", &[]),
            ("xclip", &["-i", "-selection", "clipboard"]),
        ];

        for (bin, args) in cmds {
            if let Ok(mut child) = Command::new(bin).args(*args).stdin(Stdio::piped()).spawn() {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if let Ok(st) = child.wait() {
                    if st.success() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub fn get_clipboard_text() -> Option<String> {
    imp::get_text()
}

pub fn set_clipboard_text(text: &str) -> bool {
    imp::set_text(text)
}

