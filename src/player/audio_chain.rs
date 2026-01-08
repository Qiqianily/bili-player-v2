use gstreamer::{
    Element, Pipeline, Structure,
    glib::{WeakRef, object::ObjectExt},
    prelude::{ElementExt, ElementExtManual, GstBinExtManual, PadExt},
};

use crate::errors::{PlayerError, PlayerResult};

pub struct AudioChainBuilder {
    pub url: String,
    pub volume: f64,
}
impl Default for AudioChainBuilder {
    fn default() -> Self {
        Self {
            url: String::new(),
            volume: 1.0,
        }
    }
}

impl AudioChainBuilder {
    /// 音频链构建器的构造方法
    pub fn new() -> Self {
        Self::default()
    }

    /// 从URL创建音频链
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = url.to_string();
        self
    }

    /// 设置音量
    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = volume.clamp(0.0, 2.0);
        self
    }

    /// 构建音频 pipeline（返回 Pipeline + source 元素，用于后续控制）
    pub fn build(self) -> PlayerResult<(gstreamer::Pipeline, gstreamer::Element)> {
        if self.url.is_empty() {
            return Err(PlayerError::AudioElement("URL is required".into()));
        }

        let pipeline = Pipeline::new();
        let source = gstreamer::ElementFactory::make("souphttpsrc")
            .property("location", &self.url)
            .build()
            .map_err(|e| {
                PlayerError::AudioElement(format!("Failed to create souphttpsrc: {}", e))
            })?;

        // 设置请求头
        let headers = self.create_headers();
        source.set_property("extra-headers", &headers);

        let decodebin = gstreamer::ElementFactory::make("decodebin")
            .build()
            .map_err(|e| PlayerError::AudioElement(format!("Failed to create decodebin: {}", e)))?;

        pipeline.add_many([&source, &decodebin]).unwrap();
        source.link(&decodebin).unwrap();

        // 存储 volume 设置，用于动态链接
        let volume_val = self.volume;
        let pipeline_weak = pipeline.downgrade();

        decodebin.connect_pad_added(move |_, src_pad| {
            Self::on_pad_added(&pipeline_weak, src_pad, volume_val);
        });

        Ok((pipeline, source))
    }
    /// 添加元素
    fn on_pad_added(pipeline_weak: &WeakRef<Pipeline>, src_pad: &gstreamer::Pad, volume_val: f64) {
        if let Some(caps) = src_pad.current_caps()
            && let Some(structure) = caps.structure(0)
            && structure.name().starts_with("audio/")
            && let Some(pipeline) = pipeline_weak.upgrade()
        {
            // 创建 audio 处理链
            let audioconvert = gstreamer::ElementFactory::make("audioconvert")
                .build()
                .unwrap();
            let audioresample = gstreamer::ElementFactory::make("audioresample")
                .build()
                .unwrap();
            let volume = gstreamer::ElementFactory::make("volume")
                .property("volume", volume_val)
                .name("audio_volume")
                .build()
                .unwrap();
            let sink = gstreamer::ElementFactory::make("autoaudiosink")
                .build()
                .unwrap();

            pipeline
                .add_many([&audioconvert, &audioresample, &volume, &sink])
                .unwrap();

            // 链接
            if src_pad
                .link(&audioconvert.static_pad("sink").unwrap())
                .is_err()
            {
                tracing::error!("Failed to link decodebin to audioconvert");
                return;
            }

            Element::link_many([&audioconvert, &audioresample, &volume, &sink])
                .unwrap_or_else(|_| tracing::error!("Failed to link audio processing chain"));

            // 同步状态
            let elements = [&audioconvert, &audioresample, &volume, &sink];
            for elem in &elements {
                elem.sync_state_with_parent().unwrap();
            }
        }
    }
    /// 构建请求头
    fn create_headers(&self) -> Structure {
        let mut headers = Structure::new_empty("headers");
        headers.set(
            "User-Agent",
            "Mozilla/5.0 BiliDroid/..* (bbcallen@gmail.com)",
        );
        headers.set("Referer", "https://www.bilibili.com");
        headers
    }
}
