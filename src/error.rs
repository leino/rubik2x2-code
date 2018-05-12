use std::ptr::null_mut;

#[cfg(windows)]
pub fn notify_error(message: &String)
{
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::winuser::*;
    let title = "Error";
    let title_wide: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    let message_wide: Vec<u16> = OsStr::new(message.as_str()).encode_wide().chain(once(0)).collect();
    unsafe {
        MessageBoxW(null_mut(), message_wide.as_ptr(), title_wide.as_ptr(), MB_OK | MB_ICONEXCLAMATION)
    };
}

pub fn report_fatal_error_and_exit(message: &String)
{
    use ::std::process::exit;
    notify_error(message);
    exit(1);
}
