use anyhow::{Context, Result};
use std::sync::Arc;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::Queue;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::ComputePipeline;
use vulkano::sync::GpuFuture;

fn main() -> Result<()> {
    println!("init()");
    let (device, queue) = init()?;

    println!("example_operation()");
    example_operation(&device, &queue)?;

    println!("example_compute()");
    example_compute(&device, &queue)?;

    println!("DONE!");
    Ok(())
}

fn init() -> Result<(Arc<Device>, Arc<Queue>)> {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)?;
    let physical = PhysicalDevice::enumerate(&instance)
        .filter(|p| {
            dbg!(p.limits().max_uniform_buffer_range());
            true
        })
        .nth(0)
        .expect("no device available");
    for family in physical.queue_families() {
        println!(
            "Found a queue family with {:?} queue(s)",
            family.queues_count()
        );
    }
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");
    let mut extensions = DeviceExtensions::none();
    //extensions.khr_storage_buffer_storage_class = true;
    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &extensions,
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };
    let queue = queues.next().unwrap();
    Ok((device, queue))
}

// http://vulkano.rs/guide/example-operation
// https://github.com/vulkano-rs/vulkano-www/blob/master/examples/guide-example-operation.rs
fn example_operation(device: &Arc<Device>, queue: &Arc<Queue>) -> Result<()> {
    let source_content = 0..64;
    let source =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), source_content)?;

    let dest_content = (0..64).map(|_| 0);
    let dest = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), dest_content)?;

    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())?
        .copy_buffer(source.clone(), dest.clone())?
        .build()?;
    let finished = command_buffer.execute(queue.clone())?;
    finished.then_signal_fence_and_flush()?.wait(None)?;

    let src_content = source.read().unwrap();
    let dest_content = dest.read().unwrap();
    assert_eq!(&*src_content, &*dest_content);

    Ok(())
}

// http://vulkano.rs/guide/compute-intro
// https://github.com/vulkano-rs/vulkano-www/blob/master/examples/guide-compute-operations.rs
fn example_compute(device: &Arc<Device>, queue: &Arc<Queue>) -> Result<()> {
    let data_iter = 0..65536 / 64;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), data_iter)?;
    let shader = cs::Shader::load(device.clone()).with_context(|| "create shader module")?;

    let compute_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &shader.main_entry_point(), &())
            .with_context(|| "create compute pipeline")?,
    );
    let set = Arc::new(
        PersistentDescriptorSet::start(compute_pipeline.clone(), 0)
            .add_buffer(data_buffer.clone())?
            .build()?,
    );
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())?
        .dispatch([1024 / 64, 1, 1], compute_pipeline.clone(), set.clone(), ())?
        .build()?;
    let finished = command_buffer.execute(queue.clone())?;
    finished.then_signal_fence_and_flush()?.wait(None)?;

    let content = data_buffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }

    Ok(())
}

mod cs {
    vulkano_shaders::shader! {
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
    }
}
