use crate::scanner::ScanResult;

pub trait Scanner {}

#[async_trait::async_trait]
pub trait MetadataProcessor {
    type Item;
    async fn process(&self, scan: ScanResult) -> anyhow::Result<Vec<Self::Item>>;
}

pub trait Storage {}
