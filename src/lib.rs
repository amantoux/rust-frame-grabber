extern crate libc;
use std::ffi::CStr;

use libc::{c_char, c_int, size_t};

#[link(name = "fgrabber")]
extern "C" {
    fn grab_frame(
        in_data: *const u8,
        in_size: size_t,
        out_data: *mut *mut u8,
        outsize: *mut size_t,
        rotate: *mut *mut c_char,
    ) -> c_int;
}

pub struct FrameResult<'f> {
    pub buffer: Vec<u8>,
    pub rotation: Option<&'f str>,
}

pub fn get_first_frame(video_src: &[u8]) -> Result<FrameResult, FrameError> {
    unsafe {
        let in_len = video_src.len() as size_t;
        let p_in = video_src.as_ptr();

        let mut out_len = 0 as size_t;
        let mut out: *mut u8 = std::ptr::null_mut();
        let p_out = &mut out;

        let mut rotate: *mut c_char = std::ptr::null_mut();
        let p_rotate = &mut rotate;
        let res = grab_frame(p_in, in_len, p_out, &mut out_len, p_rotate);
        if res < 0 {
            return Err(FrameError::Default);
        }

        if rotate.is_null() {
            return Ok(FrameResult {
                buffer: Vec::from_raw_parts(out, out_len, out_len),
                rotation: None,
            });
        }
        let c_rotate = CStr::from_ptr(rotate).to_str();
        // From here deallocation of 'out' is handled by Vec that owns the buffer
        Ok(FrameResult {
            buffer: Vec::from_raw_parts(out, out_len, out_len),
            rotation: c_rotate.map_or(None, |s| Some(s)),
        })
    }
}

#[derive(Debug)]
pub enum FrameError {
    Default,
}

#[cfg(test)]
mod tests {
    use crate::get_first_frame;
    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn test_grab_frame() -> std::io::Result<()> {
        let mut f = File::open("IMG_0010.MOV")?;
        let mut buffer: Vec<u8> = Vec::new();

        f.read_to_end(&mut buffer)?;
        let x = get_first_frame(&buffer).unwrap();
        let mut file = File::create("in_memory.jpeg")?;
        file.write(&x.buffer)?;
        assert_eq!(x.rotation.unwrap(), "180");
        Ok(())
    }
}
