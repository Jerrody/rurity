use super::utils::context::check_support;
#[cfg(all(not(feature = "no_log"), feature = "log"))]
use erupt::cstr;
use erupt::{vk, ExtendableFrom};
use raw_window_handle::HasRawWindowHandle;
use smallvec::SmallVec;
use std::ffi::{CStr, CString};
use tracing::error;

const TRIANGLE_VERT: &[u8] = include_bytes!("../shaders/triangle.vert.spv");
const TRIANGLE_FRAG: &[u8] = include_bytes!("../shaders/triangle.frag.spv");

pub struct Context {
    pub render_semaphore: vk::Semaphore,
    pub present_semaphore: vk::Semaphore,
    pub render_fence: vk::Fence,

    shader_modules: Vec<(vk::ShaderModule, vk::ShaderModule)>,
    pub pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,

    pub image_extent: vk::Extent2D,
    pub image_views: SmallVec<[vk::ImageView; 3]>,
    pub images: SmallVec<[vk::Image; 8]>,
    pub swapchain: vk::SwapchainKHR,

    pub command_buffers: SmallVec<[vk::CommandBuffer; 8]>,
    pub command_pool: vk::CommandPool,

    pub graphics_queue: vk::Queue,
    pub device: erupt::DeviceLoader,
    _physical_device: vk::PhysicalDevice,
    _physical_device_properties: vk::PhysicalDeviceProperties,

    _surface_format: vk::SurfaceFormatKHR,
    surface: vk::SurfaceKHR,

    #[cfg(all(
        any(feature = "no_log", feature = "log"),
        not(all(feature = "no_log", feature = "log"))
    ))]
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    instance: erupt::InstanceLoader,
    _entry: erupt::EntryLoader,
}

impl Context {
    pub fn new(
        window: &impl HasRawWindowHandle,
        width: u32,
        height: u32,
    ) -> Result<Self, vk::Result> {
        assert!(
            !(cfg!(feature = "log") && cfg!(feature = "no_log")),
            "Error: Cannot be enabeld at the same time `log` and `no_log` features!"
        );
        assert!(
            cfg!(feature = "no_log") || cfg!(feature = "log"),
            "Error: Should be enabled `no_log` or `log` feature!"
        );

        //* INSTANCE
        //* =======================================================================================================================
        let entry = erupt::EntryLoader::new().unwrap();

        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let application_name = CString::new("Rurity Editor").unwrap();
        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let engine_name = CString::new("Rurity").unwrap();

        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let application_info = vk::ApplicationInfoBuilder::new()
            .application_name(&application_name)
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::API_VERSION_1_3);

        #[allow(unused_mut, unused_assignments)]
        let mut instance: Option<erupt::InstanceLoader> = None;
        #[cfg(all(feature = "no_log", not(feature = "log")))]
        {
            let instance_extensions = unsafe {
                entry
                    .enumerate_instance_extension_properties(None, None)
                    .result()?
            };
            let instance_extensions_names = instance_extensions
                .iter()
                .map(|extension_property| extension_property.extension_name.as_ptr())
                .collect::<Vec<_>>();

            let required_instance_extensions =
                erupt::utils::surface::enumerate_required_extensions(window).result()?;
            check_support(&required_instance_extensions, &instance_extensions_names).unwrap();

            let instance_info = vk::InstanceCreateInfoBuilder::new()
                .application_info(&application_info)
                .enabled_extension_names(&required_instance_extensions);

            instance = Some(unsafe { erupt::InstanceLoader::new(&entry, &instance_info).unwrap() });
        }

        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        {
            // Layers
            let instance_layers =
                unsafe { entry.enumerate_instance_layer_properties(None).result()? };
            let instance_layer_names = instance_layers
                .iter()
                .map(|layer_property| layer_property.layer_name.as_ptr())
                .collect::<Vec<_>>();

            let required_instance_layers = [cstr!("VK_LAYER_KHRONOS_validation")];
            check_support(&required_instance_layers, &instance_layer_names).unwrap();

            // Extensions
            let instance_extensions = unsafe {
                entry
                    .enumerate_instance_extension_properties(None, None)
                    .result()?
            };
            let instance_extensions_names = instance_extensions
                .iter()
                .map(|extension_property| extension_property.extension_name.as_ptr())
                .collect::<Vec<_>>();

            let mut required_instance_extensions =
                erupt::utils::surface::enumerate_required_extensions(window).result()?;
            required_instance_extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION_NAME);
            check_support(&required_instance_extensions, &instance_extensions_names).unwrap();

            let instance_info = vk::InstanceCreateInfoBuilder::new()
                .application_info(&application_info)
                .enabled_layer_names(&required_instance_layers)
                .enabled_extension_names(&required_instance_extensions);

            instance = Some(unsafe { erupt::InstanceLoader::new(&entry, &instance_info).unwrap() });
        }

        let instance = match instance {
            Some(instance) => instance,
            None => {
                let e =
                    "Error: Failed to initialize an Instance. Please, check the enabled features!";

                error!("{e}"); // Using `error!` for the recording in the logs.
                panic!("{e}");
            }
        };

        #[cfg(all(feature = "no_log", not(feature = "log")))]
        let debug_messenger = None;

        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        let debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR_EXT
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING_EXT
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO_EXT,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::all(),
            pfn_user_callback: Some(super::utils::context::debug_callback),
            ..Default::default()
        };

        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        let debug_messenger = unsafe {
            Some(
                instance
                    .create_debug_utils_messenger_ext(&debug_messenger_info, None)
                    .result()?,
            )
        };

        //* DEVICE
        //* =======================================================================================================================
        let surface =
            unsafe { erupt::utils::surface::create_surface(&instance, window, None).result()? };

        let required_device_extensions = [vk::KHR_SWAPCHAIN_EXTENSION_NAME];

        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        let required_device_layers = [cstr!("VK_LAYER_KHRONOS_validation")];

        let physical_devices = unsafe { instance.enumerate_physical_devices(None).result()? };
        let (
            physical_device,
            queue_family_index,
            surface_format,
            present_mode,
            physical_device_properties,
        ) = physical_devices
            .into_iter()
            .filter_map(|physical_device| unsafe {
                let queue_family_index = match instance
                    .get_physical_device_queue_family_properties(physical_device, None)
                    .into_iter()
                    .enumerate()
                    .position(|(i, queue_family)| {
                        queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                            && queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                            && instance
                                .get_physical_device_surface_support_khr(
                                    physical_device,
                                    i as u32,
                                    surface,
                                )
                                .unwrap()
                    }) {
                    Some(queue_family_index) => queue_family_index as u32,
                    None => return None,
                };

                let present_modes = instance
                    .get_physical_device_surface_present_modes_khr(physical_device, surface, None)
                    .unwrap();
                let present_mode = present_modes
                    .into_iter()
                    .find(|&present_mode| present_mode == vk::PresentModeKHR::FIFO_RELAXED_KHR)
                    .unwrap();

                let mut surface_formats = instance
                    .get_physical_device_surface_formats_khr(physical_device, surface, None)
                    .unwrap();
                let surface_format = surface_formats
                    .clone()
                    .into_iter()
                    .find(|surface_format| {
                        (surface_format.format == vk::Format::R8G8B8_SRGB
                            || surface_format.format == vk::Format::B8G8R8A8_SRGB)
                            && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR_KHR
                    })
                    .unwrap_or_else(|| surface_formats.remove(0));

                #[cfg(all(not(feature = "no_log"), feature = "log"))]
                {
                    let device_layers = instance
                        .enumerate_device_layer_properties(physical_device, None)
                        .unwrap();
                    let device_layer_names = device_layers
                        .iter()
                        .map(|layer_property| layer_property.layer_name.as_ptr())
                        .collect::<Vec<_>>();
                    if check_support(&required_device_layers, &device_layer_names).is_err() {
                        return None;
                    }
                }

                let device_extensions = instance
                    .enumerate_device_extension_properties(physical_device, None, None)
                    .unwrap();
                let device_extension_names = device_extensions
                    .iter()
                    .map(|extension_property| extension_property.extension_name.as_ptr())
                    .collect::<Vec<_>>();
                if check_support(&required_device_extensions, &device_extension_names).is_err() {
                    return None;
                }

                let physical_device_properties =
                    instance.get_physical_device_properties(physical_device);

                Some((
                    physical_device,
                    queue_family_index,
                    surface_format,
                    present_mode,
                    physical_device_properties,
                ))
            })
            .max_by_key(
                |(_, _, _, _, physical_device_properties)| match physical_device_properties
                    .device_type
                {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 2,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                    _ => 0,
                },
            )
            .expect("Error: Failed to find a suitable device.");

        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let queue_infos = [vk::DeviceQueueCreateInfoBuilder::new()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])];

        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let mut dynamic_rendering =
            vk::PhysicalDeviceDynamicRenderingFeaturesKHRBuilder::new().dynamic_rendering(true);
        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let mut sync_2 =
            vk::PhysicalDeviceSynchronization2FeaturesKHRBuilder::new().synchronization2(true);
        #[cfg(all(
            any(feature = "no_log", feature = "log"),
            not(all(feature = "no_log", feature = "log"))
        ))]
        let mut device_features = vk::PhysicalDeviceFeatures2KHRBuilder::new()
            .extend_from(&mut dynamic_rendering)
            .extend_from(&mut sync_2);

        #[allow(unused_mut, unused_assignments)]
        let mut device: Option<erupt::DeviceLoader> = None;
        #[cfg(all(feature = "no_log", not(feature = "log")))]
        {
            let device_info = vk::DeviceCreateInfoBuilder::new()
                .enabled_extension_names(&required_device_extensions)
                .queue_create_infos(&queue_infos)
                .extend_from(&mut device_features);

            device = Some(unsafe {
                erupt::DeviceLoader::new(&instance, physical_device, &device_info).unwrap()
            });
        }

        #[cfg(all(not(feature = "no_log"), feature = "log"))]
        {
            let device_info = vk::DeviceCreateInfoBuilder::new()
                .enabled_extension_names(&required_device_extensions)
                .enabled_layer_names(&required_device_layers)
                .queue_create_infos(&queue_infos)
                .extend_from(&mut device_features);

            device = Some(unsafe {
                erupt::DeviceLoader::new(&instance, physical_device, &device_info).unwrap()
            });
        }

        let device = match device {
            Some(device) => device,
            None => {
                let e =
                    "Error: Failed to initialize a DeviceLoader. Please, check the enabled features!";

                error!("{e}"); // Using `error!` for the recording in the logs.
                panic!("{e}");
            }
        };
        let graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };

        //* COMMANDS
        //* =======================================================================================================================
        // TODO: Don't forget about the RESET_COMMAND_BUFFER flag when adding support for the multiple buffering.
        let command_pool_info =
            vk::CommandPoolCreateInfoBuilder::new().queue_family_index(queue_family_index);
        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .result()?
        };

        let command_buffer_info = vk::CommandBufferAllocateInfoBuilder::new()
            .command_pool(command_pool)
            .command_buffer_count(1); // TODO: Add the multiple buffering in the future.

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_info)
                .result()?
        };

        //* SWAPCHAIN
        //* =======================================================================================================================
        let surface_capabilities = unsafe {
            instance
                .get_physical_device_surface_capabilities_khr(physical_device, surface)
                .result()?
        };

        // TODO: Add support for the multiple iamges variant.
        let image_count = match surface_capabilities.min_image_count {
            min_image_count if min_image_count < surface_capabilities.min_image_count => 2,
            2 => 2,
            3 => 3,
            _ => 2,
        };

        let swapchain_info = vk::SwapchainCreateInfoKHRBuilder::new()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(vk::Extent2D { width, height })
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
            .present_mode(present_mode)
            .clipped(true);
        let swapchain = unsafe {
            device
                .create_swapchain_khr(&swapchain_info, None)
                .result()?
        };

        let swapchain_images =
            unsafe { device.get_swapchain_images_khr(swapchain, None).result()? };
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|&image| {
                let image_view_info = vk::ImageViewCreateInfoBuilder::new()
                    .format(surface_format.format)
                    .image(image)
                    .view_type(vk::ImageViewType::_2D)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    })
                    .build_dangling();

                match unsafe { device.create_image_view(&image_view_info, None).result() } {
                    Ok(image_view) => image_view,
                    Err(e) => panic!("Error: Failed to create ImageView: {e:?}"),
                }
            })
            .collect::<SmallVec<[vk::ImageView; 3]>>();

        //* PIPELINE
        //* =======================================================================================================================
        let entry_point = CString::new("main").unwrap();
        let (shader_module_vertex, shader_module_fragment) = Self::create_shader_modules(&device)?;
        let shader_stage_vertex = Self::create_shader_stage_info(
            shader_module_vertex,
            &entry_point,
            vk::ShaderStageFlagBits::VERTEX,
        );
        let shader_stage_fragment = Self::create_shader_stage_info(
            shader_module_fragment,
            &entry_point,
            vk::ShaderStageFlagBits::FRAGMENT,
        );
        let stages = [shader_stage_vertex, shader_stage_fragment];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfoBuilder::new();
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfoBuilder::new()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport {
            width: width as f32,
            height: height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
            ..Default::default()
        }
        .into_builder()];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D { width, height },
        }
        .into_builder()];
        let viewport_state = vk::PipelineViewportStateCreateInfoBuilder::new()
            .viewports(&viewports)
            .scissors(&scissors);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfoBuilder::new()
            .cull_mode(vk::CullModeFlags::FRONT)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisample_state = vk::PipelineMultisampleStateCreateInfoBuilder::new()
            .rasterization_samples(vk::SampleCountFlagBits::_1);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentStateBuilder::new()
            .color_write_mask(vk::ColorComponentFlags::all())];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfoBuilder::new()
            .attachments(&color_blend_attachments);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfoBuilder::new();
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .result()?
        };

        let color_attachment_formats = [surface_format.format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfoKHRBuilder::new()
            .color_attachment_formats(&color_attachment_formats);

        let pipeline_infos = [vk::GraphicsPipelineCreateInfoBuilder::new()
            .vertex_input_state(&vertex_input_state)
            .stages(&stages)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .extend_from(&mut pipeline_rendering_info)];
        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .result()?[0]
        };

        let semaphore_info = vk::SemaphoreCreateInfoBuilder::new();

        let render_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).result()? };
        let present_semaphore = unsafe { device.create_semaphore(&semaphore_info, None).result()? };

        let render_fence_info =
            vk::FenceCreateInfoBuilder::new().flags(vk::FenceCreateFlags::SIGNALED);
        let render_fence = unsafe { device.create_fence(&render_fence_info, None).result()? };

        // TODO: Reorder fields for corresponding with the struct's definition.
        Ok(Self {
            image_extent: vk::Extent2D { width, height },
            render_semaphore,
            present_semaphore,
            render_fence,
            command_buffers,
            command_pool,
            instance,
            _entry: entry,
            shader_modules: [(shader_module_vertex, shader_module_fragment)].to_vec(),
            pipeline,
            pipeline_layout,
            image_views: swapchain_image_views,
            images: swapchain_images,
            swapchain,
            graphics_queue,
            device,
            _physical_device: physical_device,
            _physical_device_properties: physical_device_properties,
            _surface_format: surface_format,
            surface,
            #[cfg(all(
                any(feature = "no_log", feature = "log"),
                not(all(feature = "no_log", feature = "log"))
            ))]
            debug_messenger,
        })
    }

    // TODO: Unify function for the possibility to automate the process of creation and binding ShaderModules.
    fn create_shader_modules(
        device: &erupt::DeviceLoader,
    ) -> Result<(vk::ShaderModule, vk::ShaderModule), vk::Result> {
        //* VERTEX
        let decoded = erupt::utils::decode_spv(TRIANGLE_VERT).unwrap();
        let shader_module_info = vk::ShaderModuleCreateInfoBuilder::new().code(&decoded);
        let shader_module_vertex = unsafe {
            device
                .create_shader_module(&shader_module_info, None)
                .result()?
        };

        //* FRAGMENT
        let decoded = erupt::utils::decode_spv(TRIANGLE_FRAG).unwrap();
        let shader_module_info = vk::ShaderModuleCreateInfoBuilder::new().code(&decoded);
        let shader_module_fragment = unsafe {
            device
                .create_shader_module(&shader_module_info, None)
                .result()?
        };

        Ok((shader_module_vertex, shader_module_fragment))
    }

    fn create_shader_stage_info(
        shader_module: vk::ShaderModule,
        entry_point: &CStr,
        stage: vk::ShaderStageFlagBits,
    ) -> vk::PipelineShaderStageCreateInfoBuilder {
        vk::PipelineShaderStageCreateInfoBuilder::new()
            .module(shader_module)
            .name(entry_point)
            .stage(stage)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device;
            device.device_wait_idle().unwrap();

            device.destroy_semaphore(self.render_semaphore, None);
            device.destroy_semaphore(self.present_semaphore, None);
            device.destroy_fence(self.render_fence, None);
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.shader_modules.iter().copied().for_each(
                |(shader_module_vert, shader_module_frag)| {
                    device.destroy_shader_module(shader_module_vert, None);
                    device.destroy_shader_module(shader_module_frag, None);
                },
            );
            device.destroy_command_pool(self.command_pool, None);
            self.image_views
                .iter()
                .for_each(|&image_view| device.destroy_image_view(image_view, None));
            device.destroy_swapchain_khr(self.swapchain, None);
            device.destroy_device(None);

            #[cfg(all(
                any(feature = "no_log", feature = "log"),
                not(all(feature = "no_log", feature = "log"))
            ))]
            if let Some(debug_messenger) = self.debug_messenger {
                self.instance
                    .destroy_debug_utils_messenger_ext(debug_messenger, None);
            }
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
