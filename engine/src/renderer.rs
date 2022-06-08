use erupt::vk;

#[derive(Debug)]
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    pub fn draw(&mut self, context: &super::context::Context) -> Result<(), vk::Result> {
        let device = &context.device;
        let command_buffer = context.command_buffers[0]; // TODO: Remake for the double buffering in the future.

        let image_index = unsafe {
            device
                .acquire_next_image_khr(
                    context.swapchain,
                    u64::MAX,
                    context.render_semaphore,
                    vk::Fence::null(),
                )
                .result()? as usize
        };

        let swapchain_image_attachment = vk::RenderingAttachmentInfoBuilder::new()
            .image_view(context.image_views[image_index])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 1.0, 1.0, 1.0],
                },
            });
        let color_attachments = [swapchain_image_attachment];
        let render_area = vk::Rect2D {
            offset: Default::default(),
            extent: context.image_extent,
        };
        let rendering_info = vk::RenderingInfoBuilder::new()
            .color_attachments(&color_attachments)
            .render_area(render_area)
            .layer_count(1)
            .build_dangling();

        unsafe {
            Self::prepare_frame(device, context, command_buffer, image_index)?;
            device.cmd_begin_rendering(command_buffer, &rendering_info);

            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                context.pipeline,
            );
            device.cmd_draw(command_buffer, 3, 1, 0, 0);

            device.cmd_end_rendering(command_buffer);
            device.end_command_buffer(command_buffer).result()?;
        }

        let command_buffer_info =
            [vk::CommandBufferSubmitInfoBuilder::new().command_buffer(command_buffer)];
        let wait_semaphores =
            [vk::SemaphoreSubmitInfoBuilder::new().semaphore(context.render_semaphore)];
        let signal_semaphores =
            [vk::SemaphoreSubmitInfoBuilder::new().semaphore(context.present_semaphore)];

        let submit_info = vk::SubmitInfo2Builder::new()
            .command_buffer_infos(&command_buffer_info)
            .wait_semaphore_infos(&wait_semaphores)
            .signal_semaphore_infos(&signal_semaphores);
        unsafe {
            device
                .queue_submit2(context.graphics_queue, &[submit_info], context.render_fence)
                .result()?;
        }

        let wait_semaphores = [context.present_semaphore];
        let image_indices = [image_index as u32];
        let swapchains = [context.swapchain];

        let present_info = vk::PresentInfoKHRBuilder::new()
            .wait_semaphores(&wait_semaphores)
            .image_indices(&image_indices)
            .swapchains(&swapchains);
        unsafe {
            device
                .queue_present_khr(context.graphics_queue, &present_info)
                .result()?
        };

        Ok(())
    }

    unsafe fn prepare_frame(
        device: &erupt::DeviceLoader,
        context: &super::context::Context,
        command_buffer: vk::CommandBuffer,
        image_index: usize,
    ) -> Result<(), vk::Result> {
        unsafe {
            device
                .wait_for_fences(&[context.render_fence], true, u64::MAX)
                .result()?;
            device.reset_fences(&[context.render_fence]).result()?;
            device
                .reset_command_pool(
                    context.command_pool,
                    vk::CommandPoolResetFlags::RELEASE_RESOURCES,
                )
                .result()?;
        }

        let command_buffer_begin_info = vk::CommandBufferBeginInfoBuilder::new()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .result()?;
        }

        let image = context.images[image_index];
        unsafe {
            Self::set_pipeline_barrier(
                device,
                command_buffer,
                &[
                    vk::ImageMemoryBarrier2 {
                        src_stage_mask: vk::PipelineStageFlags2::TOP_OF_PIPE,
                        dst_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                        dst_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                        new_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .into_builder(),
                    vk::ImageMemoryBarrier2 {
                        src_stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
                        dst_stage_mask: vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
                        src_access_mask: vk::AccessFlags2::COLOR_ATTACHMENT_READ,
                        old_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        new_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
                        image,
                        subresource_range: vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            level_count: 1,
                            layer_count: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    }
                    .into_builder(),
                ],
            );
        }

        Ok(())
    }

    #[inline(always)]
    unsafe fn set_pipeline_barrier(
        device: &erupt::DeviceLoader,
        command_buffer: vk::CommandBuffer,
        image_memory_barriers: &[vk::ImageMemoryBarrier2Builder],
    ) {
        unsafe {
            device.cmd_pipeline_barrier2(
                command_buffer,
                &vk::DependencyInfoKHRBuilder::new().image_memory_barriers(image_memory_barriers),
            );
        }
    }
}
