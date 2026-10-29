/// GPU buffer that is reused between frames and grows on demand.
pub struct GpuBuffer {
    buffer: wgpu::Buffer,
    label: &'static str,
    usage: wgpu::BufferUsages,
    len: u64,
}

// Small start capacity; growth is by powers of two, so it happens rarely.
const MIN_CAPACITY: u64 = 4096;

impl GpuBuffer {
    pub fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        Self {
            buffer: create(device, label, usage, MIN_CAPACITY),
            label,
            usage,
            len: 0,
        }
    }

    /// `data` length must be a multiple of `wgpu::COPY_BUFFER_ALIGNMENT`.
    pub fn upload(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) {
        let len = data.len() as u64;
        if len > self.buffer.size() {
            let capacity = len.next_power_of_two().max(MIN_CAPACITY);
            self.buffer = create(device, self.label, self.usage, capacity);
        }
        if len > 0 {
            queue.write_buffer(&self.buffer, 0, data);
        }
        self.len = len;
    }

    /// The part written by the last `upload`.
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..self.len)
    }
}

fn create(
    device: &wgpu::Device,
    label: &'static str,
    usage: wgpu::BufferUsages,
    size: u64,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}
