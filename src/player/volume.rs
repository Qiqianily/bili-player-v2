use std::sync::atomic::{AtomicU32, Ordering};

use gstreamer::{glib::object::ObjectExt, prelude::GstBinExt};

use crate::errors::{PlayerError, PlayerResult};

pub struct VolumeManager {
    pub volume_percentage: AtomicU32,
}
impl Default for VolumeManager {
    fn default() -> Self {
        Self::new()
    }
}
impl VolumeManager {
    const MAX_VOLUME: u32 = 200; // 200%
    const MIN_VOLUME: u32 = 0; // 0%
    // VolumeManager 构建函数
    pub fn new() -> Self {
        Self {
            volume_percentage: AtomicU32::new(10), // 默认10%
        }
    }
    // 设置音量大小
    pub fn set_volume(&self, pipeline: &gstreamer::Pipeline, percentage: u32) -> PlayerResult<()> {
        // 验证范围
        if percentage > Self::MAX_VOLUME {
            return Err(PlayerError::VolumeRange(format!(
                "Volume must be between {} and {}",
                Self::MIN_VOLUME,
                Self::MAX_VOLUME
            )));
        }

        // 存储原始百分比
        self.volume_percentage.store(percentage, Ordering::Relaxed);

        // 转换为 GStreamer 值并应用
        let gst_volume = Self::percentage_to_gstreamer(percentage);
        self.apply_volume(pipeline, gst_volume)?;

        tracing::info!(
            "Volume set to {}% (GStreamer: {:.2})",
            percentage,
            gst_volume
        );

        Ok(())
    }
    /// 获取音量大小，返回百分比
    pub fn get_volume_percentage(&self) -> u32 {
        self.volume_percentage.load(Ordering::Relaxed)
    }
    /// 获取音量大小，返回 GStreamer 值
    pub fn get_gstreamer_volume(&self) -> f64 {
        let percentage = self.get_volume_percentage();
        Self::percentage_to_gstreamer(percentage)
    }
    /// 调节音量大小，增加音量
    pub fn increase_volume(&self, pipeline: &gstreamer::Pipeline, step: u32) -> PlayerResult<u32> {
        let current = self.get_volume_percentage();
        let new_volume = (current + step).min(Self::MAX_VOLUME);

        if new_volume != current {
            self.set_volume(pipeline, new_volume)?;
        }

        Ok(new_volume)
    }
    /// 调节音量大小，减少音量
    pub fn decrease_volume(&self, pipeline: &gstreamer::Pipeline, step: u32) -> PlayerResult<u32> {
        let current = self.get_volume_percentage();
        let new_volume = current.saturating_sub(step);

        if new_volume != current {
            self.set_volume(pipeline, new_volume)?;
        }

        Ok(new_volume)
    }
    /// 静音
    pub fn toggle_mute(&self, pipeline: &gstreamer::Pipeline) -> PlayerResult<bool> {
        let is_muted = self.is_muted(pipeline);

        if is_muted {
            // 取消静音，恢复之前的音量
            let previous_volume = self.get_volume_percentage();
            self.apply_volume(pipeline, Self::percentage_to_gstreamer(previous_volume))?;
            tracing::info!("Unmuted, volume restored to {}%", previous_volume);
            Ok(false)
        } else {
            // 静音
            self.apply_volume(pipeline, 0.0)?;
            tracing::info!("Muted");
            Ok(true)
        }
    }
    /// 是否为静音
    pub fn is_muted(&self, pipeline: &gstreamer::Pipeline) -> bool {
        if let Some(volume_elem) = pipeline.by_name("audio_volume") {
            let current: f64 = volume_elem.property("volume");
            current < f64::EPSILON
        } else {
            false
        }
    }
    /// 设置音量
    fn apply_volume(&self, pipeline: &gstreamer::Pipeline, volume: f64) -> PlayerResult<()> {
        if let Some(volume_elem) = pipeline.by_name("audio_volume") {
            volume_elem.set_property("volume", volume);
            Ok(())
        } else {
            tracing::warn!("Volume element not found in pipeline");
            Ok(())
        }
    }
    /// 把音量转换成 f64
    fn percentage_to_gstreamer(percentage: u32) -> f64 {
        (percentage as f64 / 100.0).clamp(0.0, 2.0)
    }
}
