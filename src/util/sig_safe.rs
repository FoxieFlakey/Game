
// This function is signal safe, because using
// libc::write which is safe. this wont return error, it is ignored
// if there error
pub fn write_str_to_stdout(string: &str) {
    let mut bytes_to_write = string.as_bytes();

    while !bytes_to_write.is_empty() {
        let written_count = unsafe { libc::write(libc::STDOUT_FILENO, bytes_to_write.as_ptr().cast(), bytes_to_write.len()) };
        if written_count == -1 {
            // Error occured because we're in signal, lets ignore
            return;
        }

        bytes_to_write = &bytes_to_write[written_count as usize..];
    }
}

pub fn exit(code: u16) -> ! {
    unsafe { libc::_exit(code.into()) };
}



