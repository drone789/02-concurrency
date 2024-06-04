use std::{
    collections::HashMap,
    fmt,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
};

use fmt::Display;

use anyhow::Result;

pub struct AmapMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

impl AmapMetrics {
    pub fn new(metrics_names: &[&'static str]) -> Self {
        let map = metrics_names
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect::<HashMap<_, _>>();

        AmapMetrics {
            data: Arc::new(map),
        }
    }

    pub fn inc(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self
            .data
            .get(key)
            .ok_or_else(|| anyhow::anyhow!("key {} not found", key))?;

        counter.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}

impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        AmapMetrics {
            data: Arc::clone(&self.data),
        }
    }
}

impl Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for entry in self.data.iter() {
            writeln!(f, "{}: {}", entry.0, entry.1.load(Ordering::Relaxed))?;
        }
        Ok(())
    }
}
