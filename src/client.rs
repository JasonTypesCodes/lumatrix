use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::daemon::socket_path;

pub async fn send_command(cmd: &str) -> anyhow::Result<()> {
    let path = socket_path();
    let stream = tokio::net::UnixStream::connect(&path)
        .await
        .map_err(|_| anyhow::anyhow!("daemon not running (socket: {})", path.display()))?;

    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);

    writer.write_all(format!("{}\n", cmd).as_bytes()).await?;

    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_owned();
        if line.starts_with("error:") {
            anyhow::bail!("{}", line);
        }
        if !line.is_empty() && line != "ok" {
            println!("{}", line);
        }
    }
    Ok(())
}
