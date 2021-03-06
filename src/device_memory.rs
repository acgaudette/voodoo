
use std::sync::Arc;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::slice;
use std::marker::PhantomData;
use vks;
use ::{VdResult, Device, Handle, MemoryAllocateInfo, MemoryMapFlags};


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct DeviceMemoryHandle(pub(crate) vks::VkDeviceMemory);

impl DeviceMemoryHandle {
    pub fn to_raw(&self) -> vks::VkDeviceMemory {
        self.0
    }
}

unsafe impl Handle for DeviceMemoryHandle {
    type Target = DeviceMemoryHandle;

    fn handle(&self) -> Self::Target {
        *self
    }
}

/// A slice of mapped memory.
///
/// Use `DeviceMemory::unmap` to unmap.
pub struct MemoryMapping<'m, T> {
    ptr: *mut T,
    len: usize,
    mem_handle: DeviceMemoryHandle,
    _p: PhantomData<&'m ()>,
}

impl<'m, T> MemoryMapping<'m, T> {
    /// Returns a new `MemoryMapping`
    fn new(ptr: *mut T, len: usize, mem_handle: DeviceMemoryHandle) -> MemoryMapping<'m, T> {
        MemoryMapping {ptr, len, mem_handle, _p: PhantomData}
    }
}

impl<'m, T> Deref for MemoryMapping<'m, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<'m, T> DerefMut for MemoryMapping<'m, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}


#[derive(Debug)]
struct Inner {
    handle: DeviceMemoryHandle,
    device: Device,
    allocation_size: u64,
    memory_type_index: u32,
}

impl Drop for Inner {
    fn drop(&mut self) {
        unsafe { self.device.free_memory(self.handle, None); }
    }
}


/// A region of device memory.
///
///
/// ### Destruction
/// 
/// Dropping this `DeviceMemory` will cause `Device::free_memory` to be called, 
/// automatically releasing any resources associated with it.
///
#[derive(Debug, Clone)]
pub struct DeviceMemory {
    inner: Arc<Inner>,
}

impl DeviceMemory {
    /// Returns a new `DeviceMemoryBuilder`.
    pub fn builder<'b>() -> DeviceMemoryBuilder<'b> {
        DeviceMemoryBuilder::new()
    }

    /// Returns a new `DeviceMemory`.
    pub fn new(device: Device, allocation_size: u64, memory_type_index: u32)
            -> VdResult<DeviceMemory> {
        DeviceMemoryBuilder::new()
            .allocation_size(allocation_size)
            .memory_type_index(memory_type_index)
            .build(device)
    }

    /// Maps a region of this memory object to a pointer.
    ///
    /// Use `::unmap_ptr` to unmap this memory.
    ///
    /// The `flags` argument is reserved for future use and is ignored.
    pub unsafe fn map_to_ptr<T>(&self, offset_bytes: u64, size_bytes: u64,
            flags: MemoryMapFlags)
            -> VdResult<*mut T> {
        self.inner.device.map_memory(self.inner.handle, offset_bytes, size_bytes, flags)
    }

    /// Unmaps memory.
    ///
    /// Do not use this unless memory was mapped using `::map_to_ptr`.
    ///
    /// Use `::unmap` to unmap memory mapped by `::map`.
    pub unsafe fn unmap_ptr(&self) {
        self.inner.device.unmap_memory(self.inner.handle);
    }

    /// Maps a region of memory and returns a mutable reference to it.
    ///
    /// Use `::unmap` to unmap.
    ///
    /// Use `::copy_from_slice` on the returned slice to easily copy data into
    /// the mapped memory.
    ///
    /// ## Example
    ///
    /// ```text
    /// let mut mem = self.uniform_buffer_memory.map(0, ubo_bytes, 0)?;
    /// mem.copy_from_slice(&[ubo]);
    /// self.uniform_buffer_memory.unmap(mem);
    /// ```
    ///
    /// Note/Reminder: The above example uses a dedicated buffer and memory
    /// allocation for demonstration purposes. It is best practice to allocate
    /// all memory from one large buffer and use offsets to specify particular
    /// parts.
    ///
    /// The `flags` argument is reserved for future use and is ignored.
    ///
    /// ## Safety
    ///
    /// The caller must ensure that care is taken when mapping a buffer
    /// multiple times simultaneously. Use an appropriate synchronization
    /// mechanism such as a `std::sync::atomic::AtomicBool` (in the simplest
    /// case) to help coordinate this.
    ///
    /// The caller must also ensure that:
    ///
    /// * `offset_bytes` plus `size_bytes` is less than the size of this
    ///   region of memory.
    /// * This memory region has been created with the
    ///   `MemoryPropertyFlags::HOST_VISIBLE` flag.
    ///
    pub unsafe fn map<'m, T>(&'m self, offset_bytes: u64, size_bytes: u64, flags: MemoryMapFlags)
            -> VdResult<MemoryMapping<'m, T>> {
        let ptr = self.map_to_ptr(offset_bytes, size_bytes, flags)?;
        let len = size_bytes as usize / mem::size_of::<T>();
        Ok(MemoryMapping::new(ptr, len, self.inner.handle))
    }

    /// Unmaps memory.
    pub fn unmap<'m, T>(&self, mapping: MemoryMapping<'m, T>) {
        assert!(mapping.mem_handle == self.inner.handle,
            "cannot unmap memory: memory mapping is from a different memory object");
        unsafe { self.unmap_ptr() }
    }

    /// Returns this object's handle.
    pub fn handle(&self) -> DeviceMemoryHandle {
        self.inner.handle
    }

    /// Returns a reference to the associated device.
    pub fn device(&self) -> &Device {
        &self.inner.device
    }
}

unsafe impl<'h> Handle for &'h DeviceMemory {
    type Target = DeviceMemoryHandle;

    fn handle(&self) -> Self::Target {
        self.inner.handle
    }
}


/// A builder for `DeviceMemory`.
#[derive(Debug, Clone)]
pub struct DeviceMemoryBuilder<'b> {
    allocate_info: MemoryAllocateInfo<'b>,
}

impl<'b> DeviceMemoryBuilder<'b> {
    /// Returns a new render pass builder.
    pub fn new() -> DeviceMemoryBuilder<'b> {
        DeviceMemoryBuilder {
            allocate_info: MemoryAllocateInfo::default(),
        }
    }

    /// Specifies the size of the allocation in bytes
    pub fn allocation_size<'s>(&'s mut self, allocation_size: vks::VkDeviceSize)
            -> &'s mut DeviceMemoryBuilder<'b> {
        self.allocate_info.set_allocation_size(allocation_size);
        self
    }

    /// Specifies the memory type index, which selects the properties of the
    /// memory to be allocated, as well as the heap the memory will come from.
    pub fn memory_type_index<'s>(&'s mut self, memory_type_index: u32)
            -> &'s mut DeviceMemoryBuilder<'b> {
        self.allocate_info.set_memory_type_index(memory_type_index);
        self
    }

    /// Creates and returns a new `DeviceMemory`
    pub fn build(&self, device: Device) -> VdResult<DeviceMemory> {
        let handle = unsafe { device.allocate_memory(&self.allocate_info, None)? };

        Ok(DeviceMemory {
            inner: Arc::new(Inner {
                handle,
                device,
                allocation_size: self.allocate_info.allocation_size(),
                memory_type_index: self.allocate_info.memory_type_index(),
            })
        })
    }
}