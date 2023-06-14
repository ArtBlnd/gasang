/// A trait for IO operations.
///
/// This can represent any kind of IO, including disk IO, network IO, MMIO, etc.
pub trait IoDevice {
    /// Read from the given offset into the buffer.
    unsafe fn read_at(&self, offset: u64, buf: &mut [u8]) -> usize;

    /// Write to the given offset from the buffer.
    unsafe fn write_at(&self, offset: u64, buf: &[u8]) -> usize;

    /// Read all from the given offset into the buffer.
    unsafe fn read_all_at(&self, offset: u64, buf: &mut [u8]) -> usize {
        let mut read = 0;
        while read < buf.len() {
            let len = self.read_at(offset + read as u64, &mut buf[read..]);
            assert!(len > 0);
            read += len;
        }
        read
    }

    /// Write all to the given offset from the buffer.
    unsafe fn write_all_at(&self, offset: u64, buf: &[u8]) -> usize {
        let mut written = 0;
        while written < buf.len() {
            let len = self.write_at(offset + written as u64, &buf[written..]);
            assert!(len > 0);
            written += len;
        }
        written
    }
}
