use reqsign_core::Context;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "tokio", not(target_arch = "wasm32")))] {
        #[derive(Debug)]
        pub(crate) struct TokioFileRead;

        impl reqsign_core::FileRead for TokioFileRead {
            async fn file_read(&self, path: &str) -> reqsign_core::Result<Vec<u8>> {
                tokio::fs::read(path).await.map_err(Into::into)
            }
        }

        #[derive(Debug)]
        pub(crate) struct TokioCommandExecute;

        impl reqsign_core::CommandExecute for TokioCommandExecute {
            async fn command_execute(
                &self,
                program: &str,
                args: &[&str],
            ) -> reqsign_core::Result<reqsign_core::CommandOutput> {
                let output = tokio::process::Command::new(program)
                    .args(args)
                    .output()
                    .await
                    .map_err(|e| {
                        reqsign_core::Error::unexpected(format!(
                            "execute command {program}: {e}"
                        ))
                    })?;
                Ok(reqsign_core::CommandOutput {
                    status: output.status.code().unwrap_or(-1),
                    stdout: output.stdout,
                    stderr: output.stderr,
                })
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "tokio-http", not(target_arch = "wasm32")))] {
        #[derive(Debug)]
        pub(crate) struct ReqwestHttpSend {
            client: reqwest::Client,
        }

        impl Default for ReqwestHttpSend {
            fn default() -> Self {
                Self {
                    client: reqwest::Client::new(),
                }
            }
        }

        impl reqsign_core::HttpSend for ReqwestHttpSend {
            async fn http_send(
                &self,
                req: http::Request<bytes::Bytes>,
            ) -> reqsign_core::Result<http::Response<bytes::Bytes>> {
                let (parts, body) = req.into_parts();
                let resp = self
                    .client
                    .request(parts.method, parts.uri.to_string())
                    .headers(parts.headers)
                    .body(body)
                    .send()
                    .await
                    .map_err(|e| {
                        reqsign_core::Error::unexpected(format!("http request failed: {e}"))
                    })?;

                let status = resp.status();
                let headers = resp.headers().clone();
                let body = resp.bytes().await.map_err(|e| {
                    reqsign_core::Error::unexpected(format!("read response body: {e}"))
                })?;

                let mut builder = http::Response::builder().status(status);
                *builder.headers_mut().unwrap() = headers;
                builder.body(body).map_err(|e| {
                    reqsign_core::Error::unexpected(format!("build http response: {e}"))
                })
            }
        }
    }
}

pub(crate) fn default_context() -> Context {
    #[allow(unused_mut)]
    let mut ctx = Context::new();

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "tokio", not(target_arch = "wasm32")))] {
            ctx = ctx
                .with_file_read(TokioFileRead)
                .with_command_execute(TokioCommandExecute);
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(all(feature = "tokio-http", not(target_arch = "wasm32")))] {
            ctx = ctx.with_http_send(ReqwestHttpSend::default());
        }
    }

    ctx.with_env(reqsign_core::OsEnv)
}
