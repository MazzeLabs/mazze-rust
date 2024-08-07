

use std::time::Duration;

#[derive(Clone)]
pub struct Configuration {
    // header sync parameters
    pub header_request_batch_size: Option<usize>,
    pub header_request_timeout: Option<Duration>,
    pub max_headers_in_flight: Option<usize>,

    // epoch sync parameters
    pub epoch_request_batch_size: Option<usize>,
    pub epoch_request_timeout: Option<Duration>,
    pub max_parallel_epochs_to_request: Option<usize>,
    pub num_epochs_to_request: Option<usize>,
    pub num_waiting_headers_threshold: Option<usize>,
}
