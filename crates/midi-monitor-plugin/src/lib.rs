pub const NOTE_COUNT: usize = 128;

mod params;
mod ui;

use params::{ChordRootParam, RootNoteParam, ScaleParam};
use truce::prelude::*;
use truce_egui::EguiEditor;
use ui::{draw_editor, visuals};

const INTER_REGULAR: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");

#[derive(Params)]
pub struct MidiMonitorParams {
    #[param(name = "Root", default = 0)]
    pub root: EnumParam<RootNoteParam>,
    #[param(name = "Scale", default = 0)]
    pub scale: EnumParam<ScaleParam>,
    #[param(name = "Chord Root", default = 0)]
    pub chord_root: EnumParam<ChordRootParam>,
    #[param(name = "Chord Label", default = true)]
    pub chord_label: BoolParam,
    #[param(name = "Piano", default = true)]
    pub piano: BoolParam,
    #[param(name = "Chord Scale", default = false)]
    pub chord_scale: BoolParam,
    #[meter]
    pub meter_seed: MeterSlot,
}

pub struct MidiMonitor {
    params: Arc<MidiMonitorParams>,
    note_counts: [u16; NOTE_COUNT],
    note_levels: [f32; NOTE_COUNT],
}

impl MidiMonitor {
    pub fn new(params: Arc<MidiMonitorParams>) -> Self {
        Self {
            params,
            note_counts: [0; NOTE_COUNT],
            note_levels: [0.0; NOTE_COUNT],
        }
    }

    fn note_on(&mut self, note: u8, level: f32) {
        if level <= 0.0 {
            self.note_off(note);
            return;
        }

        let note = note as usize;
        if note < NOTE_COUNT {
            self.note_counts[note] = self.note_counts[note].saturating_add(1);
            self.note_levels[note] = level.max(0.05);
        }
    }

    fn note_off(&mut self, note: u8) {
        let note = note as usize;
        if note < NOTE_COUNT {
            self.note_counts[note] = self.note_counts[note].saturating_sub(1);
            if self.note_counts[note] == 0 {
                self.note_levels[note] = 0.0;
            }
        }
    }
}

impl PluginLogic for MidiMonitor {
    fn bus_layouts() -> Vec<BusLayout> {
        vec![BusLayout::new().with_output("Main", ChannelConfig::Stereo)]
    }

    fn reset(&mut self, _sr: f64, _bs: usize) {
        self.note_counts = [0; NOTE_COUNT];
        self.note_levels = [0.0; NOTE_COUNT];
    }

    fn process(&mut self, buffer: &mut AudioBuffer, events: &EventList, context: &mut ProcessContext) -> ProcessStatus {
        for ch in 0..buffer.num_output_channels() {
            buffer.output(ch).fill(0.0);
        }

        for event in events.iter() {
            match event.body {
                EventBody::NoteOn { note, velocity, .. } => {
                    self.note_on(note, f32::from(velocity) / 127.0);
                }
                EventBody::NoteOff { note, .. } => self.note_off(note),
                EventBody::NoteOn2 { note, velocity, .. } => {
                    self.note_on(note, f32::from(velocity) / 65_535.0);
                }
                EventBody::NoteOff2 { note, .. } => self.note_off(note),
                _ => {}
            }
        }

        for (note, level) in self.note_levels.iter().enumerate() {
            context.set_meter(note_meter_id(note), *level);
        }

        ProcessStatus::Normal
    }

    fn editor(&self) -> Box<dyn Editor> {
        EguiEditor::new(self.params.clone(), (960, 288), draw_editor)
            .with_visuals(visuals())
            .with_font(INTER_REGULAR)
            .resizable(true)
            .into_editor()
    }
}

pub fn note_meter_id(note: usize) -> u32 {
    truce::params::METER_ID_BASE + note as u32 + 1
}

truce::plugin! {
    logic: MidiMonitor,
    params: MidiMonitorParams,
}
