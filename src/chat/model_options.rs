#[allow(dead_code)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Default)]

pub struct Options {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_keep: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    typical_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_last_n: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    repeat_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat_tau: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirostat_eta: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    penalize_newline: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    numa: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_batch: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_gpu: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    main_gpu: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    low_vram: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    vocab_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_mmap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_mlock: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_thread: Option<i32>,
}

pub struct OptionsBuilder {
    options: Options,
}

impl OptionsBuilder {
    pub fn new() -> Self {
        OptionsBuilder {
            options: Options::default(),
        }
    }

    pub fn num_keep(mut self, num_keep: i32) -> Self {
        self.options.num_keep = Some(num_keep);
        self
    }

    pub fn seed(mut self, seed: i32) -> Self {
        self.options.seed = Some(seed);
        self
    }

    pub fn num_predict(mut self, num_predict: i32) -> Self {
        self.options.num_predict = Some(num_predict);
        self
    }

    pub fn top_k(mut self, top_k: i32) -> Self {
        self.options.top_k = Some(top_k);
        self
    }

    pub fn top_p(mut self, top_p: f32) -> Self {
        self.options.top_p = Some(top_p);
        self
    }

    pub fn min_p(mut self, min_p: f32) -> Self {
        self.options.min_p = Some(min_p);
        self
    }

    pub fn typical_p(mut self, typical_p: f32) -> Self {
        self.options.typical_p = Some(typical_p);
        self
    }

    pub fn repeat_last_n(mut self, repeat_last_n: i32) -> Self {
        self.options.repeat_last_n = Some(repeat_last_n);
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.options.temperature = Some(temperature);
        self
    }

    pub fn repeat_penalty(mut self, repeat_penalty: f32) -> Self {
        self.options.repeat_penalty = Some(repeat_penalty);
        self
    }

    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.options.presence_penalty = Some(presence_penalty);
        self
    }

    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.options.frequency_penalty = Some(frequency_penalty);
        self
    }

    pub fn mirostat(mut self, mirostat: i32) -> Self {
        self.options.mirostat = Some(mirostat);
        self
    }

    pub fn mirostat_tau(mut self, mirostat_tau: f32) -> Self {
        self.options.mirostat_tau = Some(mirostat_tau);
        self
    }

    pub fn mirostat_eta(mut self, mirostat_eta: f32) -> Self {
        self.options.mirostat_eta = Some(mirostat_eta);
        self
    }

    pub fn penalize_newline(mut self, penalize_newline: bool) -> Self {
        self.options.penalize_newline = Some(penalize_newline);
        self
    }

    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.options.stop = Some(stop);
        self
    }

    pub fn numa(mut self, numa: bool) -> Self {
        self.options.numa = Some(numa);
        self
    }

    pub fn num_ctx(mut self, num_ctx: i32) -> Self {
        self.options.num_ctx = Some(num_ctx);
        self
    }

    pub fn num_batch(mut self, num_batch: i32) -> Self {
        self.options.num_batch = Some(num_batch);
        self
    }

    pub fn num_gpu(mut self, num_gpu: i32) -> Self {
        self.options.num_gpu = Some(num_gpu);
        self
    }

    pub fn main_gpu(mut self, main_gpu: i32) -> Self {
        self.options.main_gpu = Some(main_gpu);
        self
    }

    pub fn low_vram(mut self, low_vram: bool) -> Self {
        self.options.low_vram = Some(low_vram);
        self
    }

    pub fn vocab_only(mut self, vocab_only: bool) -> Self {
        self.options.vocab_only = Some(vocab_only);
        self
    }

    pub fn use_mmap(mut self, use_mmap: bool) -> Self {
        self.options.use_mmap = Some(use_mmap);
        self
    }

    pub fn use_mlock(mut self, use_mlock: bool) -> Self {
        self.options.use_mlock = Some(use_mlock);
        self
    }

    pub fn num_thread(mut self, num_thread: i32) -> Self {
        self.options.num_thread = Some(num_thread);
        self
    }

    pub fn build(self) -> Options {
        self.options
    }
}
