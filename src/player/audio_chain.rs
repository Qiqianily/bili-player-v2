use gstreamer::{
    glib::object::{Cast, ObjectExt},
    prelude::{ElementExt, ElementExtManual, GstBinExt, GstObjectExt, PadExt},
};

use crate::errors::{PlayerError, PlayerResult};

pub struct AudioChainBuilder {
    pub elements: Vec<gstreamer::Element>,
}
impl Default for AudioChainBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioChainBuilder {
    /// 音频链构建器的构造方法
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }
    /// 从URL创建音频链
    pub fn with_url(mut self, url: &str) -> Self {
        let source = gstreamer::ElementFactory::make("souphttpsrc")
            .property("location", url)
            .build()
            .expect("Failed to create souphttpsrc element");

        // 设置请求头
        let headers = self.create_headers();
        source.set_property("extra-headers", &headers);

        self.elements.push(source);
        self
    }
    /// 设置音量
    pub fn with_volume(mut self, volume: f64) -> Self {
        let volume_element = gstreamer::ElementFactory::make("volume")
            .property("volume", volume.clamp(0.0, 2.0))
            .name("audio_volume")
            .build()
            .expect("Failed to create volume element");

        self.elements.push(volume_element);
        self
    }
    /// 构建音频链
    pub fn build(mut self) -> PlayerResult<Vec<gstreamer::Element>> {
        // 确保有必要的元素
        if self.elements.is_empty() {
            return Err(PlayerError::AudioElement("No elements to build".into()));
        }

        // 添加解码器和音频处理链
        self = self.add_decodebin();
        self = self.add_audioconvert();
        self = self.add_audioresample();

        // 添加音频输出
        let sink = gstreamer::ElementFactory::make("autoaudiosink")
            .build()
            .expect("Failed to create autoaudiosink");
        self.elements.push(sink);

        Ok(self.elements)
    }
    /// 添加解码器元素到音频链中
    fn add_decodebin(mut self) -> Self {
        let decodebin = gstreamer::ElementFactory::make("decodebin")
            .build()
            .expect("Failed to create decodebin");

        // 设置回调来处理动态 pads
        let pipeline_weak = if let Some(first) = self.elements.first() {
            first
                .parent()
                .and_then(|p| p.downcast::<gstreamer::Pipeline>().ok())
                .map(|p| p.downgrade())
        } else {
            None
        };

        let volume_weak = self
            .elements
            .iter()
            .find(|e| e.name() == "audio_volume")
            .map(|e| e.downgrade());

        decodebin.connect_pad_added(move |_decodebin, src_pad| {
            if let (Some(pipeline), Some(volume)) = (
                pipeline_weak.as_ref().and_then(|w| w.upgrade()),
                volume_weak.as_ref().and_then(|w| w.upgrade()),
            ) {
                Self::on_pad_added(&pipeline, &volume, src_pad);
            }
        });

        self.elements.push(decodebin);
        self
    }
    /// 添加音频转换元素
    fn add_audioconvert(mut self) -> Self {
        let audioconvert = gstreamer::ElementFactory::make("audioconvert")
            .build()
            .expect("Failed to create audioconvert");
        self.elements.push(audioconvert);
        self
    }
    /// 添加音频重采样元素
    fn add_audioresample(mut self) -> Self {
        let audioresample = gstreamer::ElementFactory::make("audioresample")
            .build()
            .expect("Failed to create audioresample");
        self.elements.push(audioresample);
        self
    }
    /// 添加音频处理链
    fn on_pad_added(
        pipeline: &gstreamer::Pipeline,
        volume: &gstreamer::Element,
        src_pad: &gstreamer::Pad,
    ) {
        // 检查是否为音频 pad
        if let Some(caps) = src_pad.current_caps()
            && let Some(structure) = caps.structure(0)
            && structure.name().starts_with("audio/")
        {
            // 创建音频处理链
            let chain = AudioChainBuilder::create_audio_chain(pipeline);

            // 连接到 volume 元素
            if let Some(audioconvert) = chain.first()
                && let Some(sink_pad) = audioconvert.static_pad("sink")
                && src_pad.link(&sink_pad).is_err()
            {
                tracing::error!("Failed to link decodebin to audioconvert");
                return;
            }

            // 连接 volume 到输出链
            if let Some(last_before_volume) = chain.last()
                && last_before_volume.link(volume).is_err()
            {
                tracing::error!("Failed to link to volume element");
            }
        }
    }

    fn create_audio_chain(pipeline: &gstreamer::Pipeline) -> Vec<gstreamer::Element> {
        let elements = vec![
            gstreamer::ElementFactory::make("audioconvert")
                .build()
                .unwrap(),
            gstreamer::ElementFactory::make("audioresample")
                .build()
                .unwrap(),
        ];

        for element in &elements {
            pipeline.add(element).unwrap();
            element.sync_state_with_parent().unwrap();
        }

        // 连接元素
        gstreamer::Element::link_many(&elements).unwrap();

        elements
    }
    /// 创建请求头
    fn create_headers(&self) -> gstreamer::Structure {
        let mut headers = gstreamer::Structure::new_empty("headers");
        headers.set(
            "User-Agent",
            "Mozilla/5.0 BiliDroid/..* (bbcallen@gmail.com)",
        );
        headers.set("Referer", "https://www.bilibili.com");
        headers
    }
}
