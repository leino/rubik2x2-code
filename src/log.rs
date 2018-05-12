use winapi;
use error;

pub fn log(message: &String)
{
    eprintln!("{}", message);
}

pub fn log_get_last_error_code_and_exit(file_name: &str, line: u32, column: u32)
{
    use ::std::process::exit;
    let error_code = unsafe { winapi::um::errhandlingapi::GetLastError() };
    let message = format!("{}: {}: {}: GetLastError(): {}", file_name, line, column, error_code);
    log(&message);
    error::notify_error(&String::from("A fatal error occurred.\nInformation identifying the error was logged."));
    exit(1);
}
