use cli_dev::logging::LoggingConfig;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;

pub fn initialize_logger(
    config: &LoggingConfig,
    is_machine: bool,
) -> fmt::SubscriberBuilder<fmt::format::DefaultFields, fmt::format::Format<fmt::format::Compact, ()>>
{
    let format = fmt::format()
        .with_level(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(!is_machine)
        .with_source_location(false)
        .without_time()
        .compact();
    let mut subscriber = tracing_subscriber::fmt().event_format(format);
    if config.verbose {
        subscriber = subscriber.with_max_level(LevelFilter::TRACE);
    } else if config.quiet {
        subscriber = subscriber.with_max_level(LevelFilter::ERROR);
    } else {
        subscriber = subscriber.with_max_level(config.log_level);
    }
    subscriber
}
