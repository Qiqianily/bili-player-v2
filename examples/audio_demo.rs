use bili_player::{
    logger::init_logger,
    player::{audio_player::AudioPlayer, command::PlayerCommand},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger("info").await?;
    let (mut player, sender) = AudioPlayer::new().await?;

    // 启动播放器后台任务（注意语法！）
    tokio::spawn(async move {
        if let Err(e) = player.run().await {
            eprintln!("Player error: {}", e);
        }
    });
    // 发送命令
    sender.send(PlayerCommand::Play).await?;
    // ⚠️ 重要：不要立即退出！否则程序结束，播放器任务被 kill
    // 你可以：
    //   - 等待用户输入
    //   - 等待某个信号
    //   - 无限等待（用于测试）

    // 👇 这一行必须执行，并且程序要停在这里等待
    tracing::info!("Running... Press Ctrl+C to exit");
    tokio::signal::ctrl_c().await?; // ✅ 这里会阻塞，直到收到 SIGINT

    tracing::info!("Shutting down...");
    Ok(())
}
