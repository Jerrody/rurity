pub mod context {
    #[cfg(not(feature = "no_log"))]
    use erupt::vk;
    use std::ffi::CStr;

    // An utility that checks availability of layers or extensions.
    pub fn check_support(
        required: &[*const std::os::raw::c_char],
        list_of: &[*const std::os::raw::c_char],
    ) -> Result<(), String> {
        if required.is_empty() {
            return Ok(());
        }

        let mut list_of_unsupported = Vec::new();

        let is_support = required.iter().all(|&required| {
            let required = unsafe { CStr::from_ptr(required) };

            let is_founded = list_of.iter().any(|&available| {
                let available = unsafe { CStr::from_ptr(available) };

                required == available
            });

            if !is_founded {
                list_of_unsupported.push(required.to_string_lossy().to_string());
            }

            is_founded
        });

        match is_support {
            true => Ok(()),
            false => {
                println!("Unsupported:");
                list_of_unsupported
                    .into_iter()
                    .for_each(|unsupported| println!("\t- {unsupported}"));

                Err("Error: Unsupported required extensions or layers of Vulkan.".to_string())
            }
        }
    }

    #[cfg(not(feature = "no_log"))]
    pub unsafe extern "system" fn debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagBitsEXT,
        message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        use tracing::{error, info, warn};

        let message_type = format!("{message_types:?}");
        let message_type = message_type.strip_suffix("_EXT").unwrap();

        // NOTE: Spaces between `\n` and {} need for alignment with `tracing` messages.
        let message = format!("\n  [{message_type}]\n  {:?}", unsafe {
            CStr::from_ptr((*p_callback_data).p_message)
        });

        match message_severity {
            vk::DebugUtilsMessageSeverityFlagBitsEXT::ERROR_EXT => error!("{message}"),
            vk::DebugUtilsMessageSeverityFlagBitsEXT::WARNING_EXT => warn!("{message}"),
            vk::DebugUtilsMessageSeverityFlagBitsEXT::INFO_EXT => info!("{message}"),
            _ => warn!("[UNKNOWN]: {:?}", unsafe {
                CStr::from_ptr((*p_callback_data).p_message)
            }),
        }

        vk::FALSE
    }
}

#[cfg(all(not(feature = "no_log"), feature = "log"))]
pub mod logging {
    use tracing::Level;
    use tracing_subscriber::{fmt, layer::SubscriberExt};

    pub fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
        let file_appender = tracing_appender::rolling::daily("engine/logs", "engine.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let offset_time = fmt::time::OffsetTime::new(
            time::UtcOffset::current_local_offset().unwrap(),
            time::macros::format_description!("[hour]:[minute]:[second]"),
        );
        let subsciber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::default().add_directive(Level::TRACE.into()))
            .with(
                fmt::Layer::new()
                    .pretty()
                    .with_writer(non_blocking)
                    .with_ansi(false)
                    .with_thread_names(true)
                    .with_thread_ids(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_timer(offset_time.clone()),
            )
            .with(
                fmt::Layer::new()
                    .pretty()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
                    .with_thread_names(true)
                    .with_thread_ids(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_timer(offset_time),
            );

        tracing::subscriber::set_global_default(subsciber).expect("Failed to init logging.");

        guard
    }
}
