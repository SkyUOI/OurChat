//! Metrics collection module
//!
//! This module provides the custom metrics recorder that implements the `metrics::Recorder` trait.
//! The recorder stores metrics for:
//! - gRPC API responses (via `OurChatRecorder::get_monitoring_metrics()`)
//! - Application-level metric tracking using the standard `metrics` facade

pub mod recorder;

pub use recorder::OurChatRecorder;
