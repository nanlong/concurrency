use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::{fmt, sync::Arc};

#[derive(Debug, Clone)]
pub struct AmapMetrics {
    data: Arc<HashMap<String, AtomicI64>>,
}

impl AmapMetrics {
    pub fn new(names: &[&str]) -> Self {
        let data = names
            .iter()
            .map(|name| (name.to_string(), AtomicI64::new(0)))
            .collect();

        Self {
            data: Arc::new(data),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let key = key.into();
        let counter = self
            .data
            .get(&key)
            .ok_or_else(|| anyhow!("key {} not found", &key))?;
        counter.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    pub fn dsc(&self, key: impl Into<String>) -> Result<()> {
        let key = key.into();
        let counter = self
            .data
            .get(&key)
            .ok_or_else(|| anyhow!("key {} not found", &key))?;
        counter.fetch_sub(1, Ordering::SeqCst);
        Ok(())
    }
}

impl fmt::Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AmapMetrics {{ \r\n")?;

        for (k, v) in self.data.iter() {
            write!(f, "  {}: {:?}\r\n", k, v)?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amap_inc() -> Result<()> {
        let metrics = AmapMetrics::new(&["foo", "bar"]);
        metrics.inc("foo")?;
        metrics.inc("bar")?;
        metrics.inc("foo")?;

        if let Some(v) = metrics.data.get("foo") {
            assert_eq!(v.load(Ordering::SeqCst), 2);
        } else {
            panic!("foo not found");
        };

        if let Some(v) = metrics.data.get("bar") {
            assert_eq!(v.load(Ordering::SeqCst), 1);
        } else {
            panic!("bar not found");
        };

        Ok(())
    }

    #[test]
    fn test_amap_dsc() -> Result<()> {
        let metrics = AmapMetrics::new(&["foo", "bar"]);
        metrics.dsc("foo")?;
        metrics.dsc("bar")?;
        metrics.dsc("foo")?;

        if let Some(v) = metrics.data.get("foo") {
            assert_eq!(v.load(Ordering::SeqCst), -2);
        } else {
            panic!("foo not found");
        };

        if let Some(v) = metrics.data.get("bar") {
            assert_eq!(v.load(Ordering::SeqCst), -1);
        } else {
            panic!("bar not found");
        };

        Ok(())
    }
}
