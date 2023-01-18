
pub struct MemoryManagementUnit {

}

enum Page {
    Unmapped,
    MemoryMapped,
    IoMapped,
}