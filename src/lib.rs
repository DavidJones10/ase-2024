use comb_filter::CombFilter;
use nih_plug::prelude::*;
use std::sync::{mpsc::channel, Arc};
mod comb_filter;
mod ring_buffer;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct AseExample {
    params: Arc<AseExampleParams>,
    filt: Option<CombFilter>,

}

#[derive(Params)]
struct AseExampleParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "delay"]
    pub delay: FloatParam,
    #[id = "filter_type"]
    pub filter_type: EnumParam<comb_filter::FilterType>,
}

impl Default for AseExample {
    fn default() -> Self {
        Self {
            params: Arc::new(AseExampleParams::default()),
            filt: None,
        }
    }
}

impl Default for AseExampleParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Linear {
                    min: util::db_to_gain(-24.0),
                    max: util::db_to_gain(0.0),
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Logarithmic(100.0))
            .with_unit(" dB")
            // There are many predefined formatters we can use here. If the gain was stored as
            // decibels instead of as a linear gain value, we could have also used the
            // `.with_step_size(0.1)` function to get internal rounding.
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            delay: FloatParam::new(
                "Time",
                200.0,
                FloatRange::Linear { min: 0.0, max: 2.0 },
            ).with_smoother(SmoothingStyle::Linear(300.0))
             .with_unit("sec")
             .with_step_size(0.001),
            filter_type: EnumParam::new("Filter Type", comb_filter::FilterType::FIR),

        }
    }
}

impl Plugin for AseExample {
    const NAME: &'static str = "Simple Comb Filter";
    const VENDOR: &'static str = "David Jones";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "ijc@gatech.edu";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        let mut filt_ = CombFilter::new(comb_filter::FilterType::IIR,2.0,_buffer_config.sample_rate,2);
        self.filt = Some(filt_);
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut samples = buffer.as_slice();
        
       
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            

            for (channel_id,sample) in channel_samples.into_iter().enumerate() {
                
                let gain = self.params.gain.smoothed.next();
                let delay = self.params.delay.smoothed.next();
                let filter_type = self.params.filter_type.value();
                if let Some(filt) = self.filt.as_mut() {
                    // Call methods on filt without moving it
                    filt.set_param(comb_filter::FilterParam::Delay, delay);
                    filt.set_param(comb_filter::FilterParam::Gain, gain);
                    filt.filterType = filter_type;
                    filt.process(sample,channel_id);
                }
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for AseExample {
    const CLAP_ID: &'static str = "edu.gatech.ase-example";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple example plugin for Audio Software Engineering.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for AseExample {
    const VST3_CLASS_ID: [u8; 16] = *b"ASE-2024-Example";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(AseExample);
nih_export_vst3!(AseExample);
