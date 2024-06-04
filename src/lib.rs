mod matrix;
mod vector;

mod metrics;

pub use matrix::{multiply, Matrix};

pub use metrics::{AmapMetrics, CmapMetrics};
pub use vector::dot_product;
