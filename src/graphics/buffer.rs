use std::ptr::null;

pub struct Buffer {
    handle: u32,
    size: isize,
}

impl Buffer {
    pub fn new(size: isize) -> Buffer {
        let mut handle = 0;
        unsafe {
            gl::GenBuffers(1, &mut handle);

            if handle == 0 {
                panic!("Failed creating GL buffer ({} bytes)", size);
            }

            gl::BindBuffer(gl::ARRAY_BUFFER, handle);
            gl::BufferData(gl::ARRAY_BUFFER, size, null(), gl::DYNAMIC_DRAW);
        }

        Buffer { handle, size }
    }

    pub fn size(self: &Self) -> isize {
        self.size
    }

    pub fn resize(self: &mut Self, new_size: isize) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
            gl::BufferData(gl::ARRAY_BUFFER, new_size, null(), gl::DYNAMIC_DRAW);

            self.size = new_size;
        }
    }

    pub fn set_data<T>(self: &mut Self, offset: isize, data: &[T]) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.handle);
            gl::BufferSubData(gl::ARRAY_BUFFER, offset, (data.len() * size_of::<T>()) as isize, data.as_ptr() as *const _);
        }
    }

    pub fn handle(self: &Self) -> u32 {
        self.handle
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}