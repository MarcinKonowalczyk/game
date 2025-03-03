use raylib_wasm::{__va_list_tag, cstr};

pub type VaList = __va_list_tag;

#[link(name = "vprintf", kind = "static")]
extern "C" {
    // void _vprintf(const char *format, va_list args);
    fn _vprintf(format: *const i8, va_list: *mut VaList);
    // void _vsnprintf(char *str, size_t size, const char *format, va_list args);
    fn _vsnprintf(str: *mut i8, size: usize, format: *const i8, va_list: *mut VaList);
}

// Print to stdout. Flushes the buffer
#[allow(unused)]
pub unsafe fn vprintf(format: &str, va_list: *mut VaList) {
    _vprintf(cstr!(format), va_list);
}

fn buffer_to_string(buffer: &[i8]) -> String {
    let buffer = buffer.iter().map(|&x| x as u8).collect::<Vec<u8>>();
    let buffer = buffer.as_slice();
    let buffer = std::str::from_utf8(&buffer).unwrap();
    buffer.to_string()
}

// Print to a string
#[allow(unused)]
pub unsafe fn vsnprintf(format: &str, va_list: *mut VaList) -> String {
    let mut buffer: [i8; 1024] = [0; 1024];
    _vsnprintf(buffer.as_mut_ptr(), buffer.len(), cstr!(format), va_list);

    buffer_to_string(&buffer)
}

// Jsut like vsnprintf, but you provide the buffer
#[allow(unused)]
pub unsafe fn vsnprintf_buff(format: &str, va_list: *mut VaList, buffer: &mut [i8]) -> String {
    _vsnprintf(buffer.as_mut_ptr(), buffer.len(), cstr!(format), va_list);

    buffer_to_string(&buffer)
}
