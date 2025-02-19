use std::os::raw::c_int;

const M_MMAP_THRESHOLD: c_int = -3;

extern "C" {
    fn malloc_trim(pad: usize);

    fn mallopt(param: c_int, value: c_int) -> c_int;
}

pub fn trim(pad: usize) {
    unsafe {
        malloc_trim(pad);
    }
}

/// Prevents glibc from hoarding memory via memory fragmentation.
pub fn limit_mmap_threshold(threshold: i32) {
    unsafe {
        mallopt(M_MMAP_THRESHOLD, threshold as c_int);
    }
}
