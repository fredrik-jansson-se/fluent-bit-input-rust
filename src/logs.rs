use std::collections::HashMap;

pub(crate) fn start_log_collection(
    cfg: &crate::Config,
    msg_tx: std::sync::mpsc::SyncSender<Vec<u8>>,
) {
    let cfg = cfg.clone();
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        if let Err(e) = rt.block_on(log_main(cfg, msg_tx)) {
            eprintln!("{}", e);
        }
    });
}

async fn log_main(
    cfg: crate::Config,
    msg_tx: std::sync::mpsc::SyncSender<Vec<u8>>,
) -> anyhow::Result<()> {
    let avassa_client = avassa_client::ClientBuilder::new()
        .danger_accept_invalid_certs()
        .danger_accept_invalid_hostnames()
        .login(&cfg.api_url, &cfg.username.unwrap(), &cfg.password.unwrap())
        .await?;

    let site_name = avassa_client::utilities::state::site_name(&avassa_client).await?;
    tracing::debug!("site name: {}", site_name);

    let mut log_check_interval = tokio::time::interval(std::time::Duration::from_secs(60));

    let mut active_logs: HashMap<String, tokio_util::sync::DropGuard> = HashMap::new();

    loop {
        log_check_interval.tick().await;

        let logs = get_volga_streams(&avassa_client, &site_name, LOG_STREAM_PREFIX).await?;

        active_logs.retain(|k, _| logs.contains(k));

        for log in logs {
            if active_logs.contains_key(&log) {
                continue;
            }
            let cancel_token = tokio_util::sync::CancellationToken::new();
            let child_token = cancel_token.child_token();

            let avassa_client = avassa_client.clone();
            let site_name = site_name.clone();
            let log_name = log.clone();
            let msg_tx = msg_tx.clone();
            tracing::debug!("Starting log collection: {}", log_name);

            tokio::spawn(async move {
                if let Err(e) =
                    monitor_logs(log_name, avassa_client, site_name, child_token, msg_tx).await
                {
                    tracing::error!(%e);
                }
            });

            active_logs.insert(log, cancel_token.drop_guard());
        }
    }
}

#[tracing::instrument(skip(avassa_client, cancel_token, msg_tx))]
async fn monitor_logs(
    log_name: String,
    avassa_client: avassa_client::Client,
    site_name: String,
    cancel_token: tokio_util::sync::CancellationToken,
    msg_tx: std::sync::mpsc::SyncSender<Vec<u8>>,
) -> anyhow::Result<()> {
    tracing::debug!("start monitoring");

    let opts = avassa_client::volga::ConsumerOptions {
        position: avassa_client::volga::ConsumerPosition::End,
        ..Default::default()
    };

    let mut consumer = avassa_client
        .volga_open_consumer("gcp-exporter", &log_name, opts)
        .await?;

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                tracing::info!("Cancelled");
                break
            },

            msg = consumer.consume() => match msg {
                Ok(msg) => {
                    let mut mp = Vec::new();
                    rmp::encode::write_array_len(&mut mp, 2).unwrap();

                    // Encode time
                    rmp::encode::write_ext_meta(&mut mp, 8, 0).unwrap();
                    const NS: i64 = 1000000000;
                    let now_sec = msg.time.timestamp_nanos() / NS;
                    let now_nsec = msg.time.timestamp_nanos() - now_sec * NS;
                    mp.extend_from_slice(&(now_sec as u32).to_be_bytes());
                    mp.extend_from_slice(&(now_nsec as u32).to_be_bytes());
                    let mut data = rmp_serde::encode::to_vec_named(&msg)?;
                    mp.append(&mut data);
                    msg_tx.send(mp)?;
                },
                Err(e) => return Err(e.into()),
            },
        }
    }

    Ok(())
}

const LOG_STREAM_PREFIX: &str = "system:container-logs:";

async fn get_volga_streams(
    client: &avassa_client::Client,
    site_name: &str,
    stream_prefix: &str,
) -> anyhow::Result<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct Topic {
        topic: String,
    }
    let json: Vec<Topic> = client
        .get_json("/v1/state/volga/topics", Some(&[("site", site_name)]))
        .await?;

    Ok(json
        .into_iter()
        .map(|t| t.topic)
        .filter(|topic| topic.starts_with(stream_prefix))
        .collect())
}
