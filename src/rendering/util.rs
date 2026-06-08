pub fn wait_device(device: &wgpu::Device, id: wgpu::SubmissionIndex) {
    match device.poll(wgpu::PollType::Wait {
        submission_index: Some(id),
        timeout: None,
    }) {
        Ok(wgpu::PollStatus::WaitSucceeded) | Ok(wgpu::PollStatus::QueueEmpty) => return,
        Ok(wgpu::PollStatus::Poll) => {
            panic!("This shouldn't occur!");
        }

        Err(e) => {
            panic!("Cannot wait for device to complete submission: {e}");
        }
    }
}
