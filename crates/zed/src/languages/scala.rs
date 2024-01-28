use anyhow::Result;
use lsp::LanguageServerBinary;
use std::{any::Any, path::PathBuf};
use language::{LanguageServerName, LspAdapter, LspAdapterDelegate};
use async_trait::async_trait;

pub struct MetalsLspAdapter;

#[async_trait]
impl LspAdapter for MetalsLspAdapter {
    fn name(&self) -> LanguageServerName {
        LanguageServerName("scala-metals".into())
    }

    fn short_name(&self) ->  &'static str {
        "scala-metals"
    }

    async fn fetch_latest_server_version(
        &self,
        delegate: &dyn LspAdapterDelegate,
    ) -> Result<Box<dyn 'static + Send + Any>> {
        todo!()
    }

    async fn fetch_server_binary(
        &self,
        version: Box<dyn 'static + Send + Any>,
        container_dir: PathBuf,
        delegate: &dyn LspAdapterDelegate,
    ) -> Result<LanguageServerBinary> {
        todo!()
    }

    async fn cached_server_binary(
        &self,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Option<LanguageServerBinary> {
        todo!()
    }

    async fn installation_test_binary(
        &self,
        container_dir: PathBuf,
    ) -> Option<LanguageServerBinary> {
        todo!()
    }
}
