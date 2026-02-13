use ash::vk;

pub struct QueuePool {
    queues: Vec<Vec<vk::Queue>>,
    props: Vec<vk::QueueFamilyProperties>,
}

impl QueuePool {
    pub fn get_queue(&self, flags: vk::QueueFlags) -> Option<vk::Queue> {
        let mut family_index = None;

        for (index, prop) in self.props.iter().enumerate() {
            if prop.queue_flags.contains(flags) {
                family_index = Some(index);
            }
        }

        if let Some(index) = family_index {
            return Some(self.queues[index][0]);
        }

        None
    }
}

impl QueuePool {
    pub fn new(device: &ash::Device, props: &[vk::QueueFamilyProperties]) -> Self {
        let mut queues = vec![];

        for (i, prop) in props.iter().enumerate() {
            let mut v = vec![];
            for j in 0..prop.queue_count {
                let queue = unsafe { device.get_device_queue(i as u32, j) };
                log::debug!(
                    "Queue Family: {} Queue: {} Flags: {:?}",
                    i,
                    j,
                    prop.queue_flags
                );
                v.push(queue);
            }
            queues.push(v);
        }

        QueuePool {
            queues,
            props: props.to_vec(),
        }
    }
}
