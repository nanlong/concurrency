use dashmap::DashMap;
use std::{fmt, sync::Arc};

#[derive(Debug, Clone)]
pub struct CmapMetrics {
    data: Arc<DashMap<String, i64>>,
}

impl Default for CmapMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl CmapMetrics {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) {
        self.data
            .entry(key.into())
            .and_modify(|v| *v += 1)
            .or_insert(1);
    }

    pub fn dsc(&self, key: impl Into<String>) {
        self.data
            .entry(key.into())
            .and_modify(|v| *v -= 1)
            .or_insert(-1);
    }
}

impl fmt::Display for CmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CmapMetrics {{ \r\n")?;

        for entry in self.data.iter() {
            write!(f, "  {}: {}\r\n", entry.key(), entry.value())?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmap_inc() {
        let metrics = CmapMetrics::new();
        metrics.inc("foo");
        metrics.inc("bar");
        metrics.inc("foo");

        if let Some(v) = metrics.data.get("foo") {
            assert_eq!(*v, 2);
        } else {
            panic!("foo not found");
        };

        if let Some(v) = metrics.data.get("bar") {
            assert_eq!(*v, 1);
        } else {
            panic!("bar not found");
        };
    }

    #[test]
    fn test_cmap_dsc() {
        let metrics = CmapMetrics::new();
        metrics.dsc("foo");
        metrics.dsc("bar");
        metrics.dsc("foo");

        if let Some(v) = metrics.data.get("foo") {
            assert_eq!(*v, -2);
        } else {
            panic!("foo not found");
        };

        if let Some(v) = metrics.data.get("bar") {
            assert_eq!(*v, -1);
        } else {
            panic!("bar not found");
        };
    }
}
