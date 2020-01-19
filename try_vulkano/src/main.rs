use anyhow::Result;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::CpuAccessibleBuffer;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::sync::GpuFuture;

fn main() -> Result<()> {
    println!("example_operation()");
    example_operation()?;

    println!("DONE!");
    Ok(())
}

// http://vulkano.rs/guide/example-operation
// https://github.com/vulkano-rs/vulkano-www/blob/master/examples/guide-example-operation.rs
fn example_operation() -> Result<()> {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)?;
    let physical = PhysicalDevice::enumerate(&instance)
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
    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed to create device")
    };
    let queue = queues.next().unwrap();

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
