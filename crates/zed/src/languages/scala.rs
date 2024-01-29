use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::StreamExt;
use language::{LanguageServerName, LspAdapter, LspAdapterDelegate};
use lsp::LanguageServerBinary;
use smol::fs;
use std::{any::Any, path::PathBuf};
use util::async_maybe;
use util::github::latest_github_release;
use util::ResultExt;
use async_process::Command;
use serde_json::{json, Value};

pub struct MetalsLspAdapter;

#[async_trait]
impl LspAdapter for MetalsLspAdapter {
    fn name(&self) -> LanguageServerName {
        LanguageServerName("metals".into())
    }

    fn short_name(&self) ->  &'static str {
        "metals"
    }

    async fn fetch_latest_server_version(
        &self,
        delegate: &dyn LspAdapterDelegate,
    ) -> Result<Box<dyn 'static + Send + Any>> {
        let release =
            latest_github_release("scalameta/metals", false, delegate.http_client())
                .await?;

        let version = release.tag_name.trim_start_matches("v").to_string();

        // let asset_name = "eclipse.jdt.ls.tar.gz";
        // let asset = release
        //     .assets
        //     .iter()
        //     .find(|asset| asset.name == asset_name)
        //     .ok_or_else(|| anyhow!("no asset found matching {:?} \n", asset_name))?;
        // let version = GitHubLspBinaryVersion {
        //     name: release.name,
        //     url: asset.browser_download_url.clone(),
        // };

        Ok(Box::new(version) as Box<_>)
    }

    async fn fetch_server_binary(
        &self,
        version: Box<dyn 'static + Send + Any>,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Result<LanguageServerBinary> {
        let version = version.downcast::<String>().unwrap();

        let binary_path = container_dir.join("metals");

        let str = format!("org.scalameta:metals_2.13:{}", version);
        let out = format!("{}", binary_path.display());


        let coursier_args = vec!("bootstrap",
            "--java-opt", "-XX:+UseG1GC",
            "--java-opt", "-XX:+UseStringDeduplication",
            "--java-opt", "-Xss4m",
            "--java-opt", "-Xms100m",
            "--java-opt", "-Dmetals.verbose=on",
            "--java-opt", "-Dmetals.http=on",
            "--java-opt", "-Dmetals.loglevel=debug",
            str.as_str(),
            "-o", out.as_str(), "-f"
        );

        // let args1 = coursier_args.split(' ').filter(|s| !s.is_empty());

        // run process metals_command
        let res = Command::new("coursier")
            .args(coursier_args)
            .status()
            .await?;

        if !res.success() {
            return Err(anyhow!("Failed to download metals"));
        }

        fs::set_permissions(
            &binary_path,
            <fs::Permissions as fs::unix::PermissionsExt>::from_mode(0o755),
        )
        .await?;
        Ok(LanguageServerBinary {
            path: binary_path,
            arguments: vec![],
        })
    }

    async fn cached_server_binary(
        &self,
        container_dir: PathBuf,
        _: &dyn LspAdapterDelegate,
    ) -> Option<LanguageServerBinary> {
        get_cached_server_binary(container_dir).await
    }

    async fn installation_test_binary(
        &self,
        container_dir: PathBuf,
    ) -> Option<LanguageServerBinary> {
        get_cached_server_binary(container_dir)
            .await
            .map(|mut binary| {
                binary.arguments = vec!["--version".into()];
                binary
            })
    }

    fn initialization_options(&self) -> Option<Value> {
        Some(json!({
            "isHttpEnabled": true
        }))
    }
}

async fn get_cached_server_binary(container_dir: PathBuf) -> Option<LanguageServerBinary> {
    async_maybe!({
        let mut last_binary_path = None;
        let mut entries = fs::read_dir(&container_dir).await?;
        while let Some(entry) = entries.next().await {
            let entry = entry?;
            if entry.file_type().await?.is_file()
                && entry
                    .file_name()
                    .to_str()
                    .map_or(false, |name| name == "metals")
            {
                last_binary_path = Some(entry.path());
            }
        }

        if let Some(path) = last_binary_path {
            Ok(LanguageServerBinary {
                path,
                arguments: Vec::new(),
            })
        } else {
            Err(anyhow!("no cached binary"))
        }
    })
    .await
    .log_err()
}
