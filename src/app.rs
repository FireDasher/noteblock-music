use std::{fs::File, io::{BufReader, BufWriter}, path::PathBuf, u32};

use eframe::egui::{self, Color32, Key, Pos2, Rect, Stroke, pos2, vec2};
use lewton::inside_ogg::OggStreamReader;
use rodio::{OutputStream, Source, buffer::SamplesBuffer};

use crate::project::{self, Layer, Project};

const SOUND_FILE_NAMES: [&str; 16] = ["harp", "dbass", "bdrum", "sdrum", "click", "guitar", "flute", "bell", "icechime", "xylobone", "iron_xylophone", "cow_bell", "didgeridoo", "bit", "banjo", "pling"];

pub struct App {
	project: Project,
	project_path: Option<PathBuf>,

	current_layer: usize,

	playback_time: f32,
	playing: bool,

	scroll: f32,
	vscroll: f32,

	stream: OutputStream,
	noteblock_sounds: Vec<SamplesBuffer>,
	
	tps: f32,
	last_played_note: u8, last_playback_time_tick: u32,
	noteblock_texture: egui::TextureHandle,

	selection_start: Pos2,
	selection_end: Pos2,
	selected_notes: Vec<usize>,
	
	unsaved_changes: bool,
	show_unsaved_changes_confirmation_dialogue_modal_because_exit: bool,
	show_unsaved_changes_confirmation_dialogue_modal_because_new: bool,
}

impl App {
	pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
		cc.egui_ctx.set_visuals(egui::Visuals::dark());
		let mut stream = rodio::OutputStreamBuilder::open_default_stream().expect("Failed to open stream!");
		stream.log_on_drop(false);

		let mut noteblock_sounds: Vec<SamplesBuffer> = Vec::new();

		for sound in SOUND_FILE_NAMES {
			noteblock_sounds.push(Self::load_ogg(&format!("sounds/{sound}.ogg")));
		}

		Self {
			project: Project::new(), project_path: None,
			current_layer: 0,
			playback_time: f32::MIN, playing: false,
			scroll: 0.0, vscroll: 54.0,
			tps: 10.0,
			last_played_note: 255, last_playback_time_tick: 0,
			noteblock_texture: cc.egui_ctx.load_texture("noteblock", egui::ColorImage::from_rgba_unmultiplied([16, 16], include_bytes!("noteblock.bin")), egui::TextureOptions::default()),
			unsaved_changes: false,
			stream, noteblock_sounds,
			selection_start: pos2(0.0, 0.0), selection_end: pos2(0.0, 0.0), selected_notes: Vec::new(),
			show_unsaved_changes_confirmation_dialogue_modal_because_exit: false, show_unsaved_changes_confirmation_dialogue_modal_because_new: false,
		}
	}

	fn get_note_name(note: u8) -> String {
		const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
		format!("{}{}", NOTE_NAMES[(note % 12) as usize], (note / 12) as i32 - 1)
	}

	fn load_ogg(path: &str) -> SamplesBuffer {
		let mut reader = OggStreamReader::new(File::open(path).expect("Audio file not found!")).expect("Failed to read audio file!");
		let channels: u16 = 1;
		let sample_rate = reader.ident_hdr.audio_sample_rate;

		let mut sampels: Vec<f32> = Vec::new();
		while let Some(packet) = reader.read_dec_packet_generic::<Vec<Vec<f32>>>().unwrap() {
			sampels.extend(&packet[0]);
		}

		SamplesBuffer::new(channels, sample_rate, sampels)
	}

	fn play_note(&self, note: u8, instrument: u8) {
		let sink = rodio::Sink::connect_new(self.stream.mixer());

		let sound = self.noteblock_sounds[instrument as usize].clone() .speed(2.0_f32.powf((note as f32 - 66.0) / 12.0));
		sink.append(sound);

		sink.detach();
	}

	fn reset(&mut self) {
		self.project = Project::new();
		self.project_path = None;
		self.current_layer = 0;
		self.unsaved_changes = false;

		self.scroll = 0.0;
		self.vscroll = 54.0;
	}

	fn open(&mut self) {
		if let Some(path) = rfd::FileDialog::new().set_title("Choose project (THIS WILL DELETE YOUR CURRENT PROJECT IF IT'S NOT SAVED!!!)").add_filter("NoteBlockMusic files", &["nbm"]).pick_file() {
			self.project_path = Some(path);
			if let Some(path) = &self.project_path {
				self.project = serde_json::from_reader(BufReader::new(File::open(path).expect("Failed to open file!"))).expect("Failed to load project!");
				self.current_layer = 0;
				self.unsaved_changes = false;
				self.scroll = 0.0;
				self.vscroll = 54.0;
			}
		}
	}

	fn save(&mut self) -> bool {
		if let Some(path) = &self.project_path {
			serde_json::to_writer(BufWriter::new(File::create(path).expect("Failed to open filke!")), &self.project).expect("Failed to save!!!");
		} else {
			if let Some(path) = rfd::FileDialog::new().set_title("Save new project").add_filter("NoteBlockMusic files", &["nbm"]).save_file() {
				self.project_path = Some(path);
				if let Some(path) = &self.project_path {
					serde_json::to_writer(BufWriter::new(File::create_new(path).expect("Failed to open filke!")), &self.project).expect("Failed to save!!!");
				}
			} else {
				return false;
			}
		}
		self.unsaved_changes = false;
		true
	}
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if ctx.input(|i| i.viewport().close_requested()) {
			if self.unsaved_changes {
				self.show_unsaved_changes_confirmation_dialogue_modal_because_exit = true;
				ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
			}
		}

		egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
			egui::MenuBar::new().ui(ui, |ui| {
				ui.menu_button("File", |ui| {
					if ui.button("New").clicked() {
						if self.unsaved_changes {
							self.show_unsaved_changes_confirmation_dialogue_modal_because_new = true;
						} else {
							self.reset();
						}
					}
					// messy saving system
					if ui.button("Open").clicked() {
						self.open();
					}
					if ui.button("Save").clicked() {
						self.save();
					}
				});
				ui.menu_button("Edit", |ui| {
					if ui.button("Undo").clicked() {
						
					}
					if ui.button("Redo").clicked() {
						
					}
				});
				if ui.input(|i| i.modifiers.ctrl && i.key_pressed(Key::S)) { // More convenient save button because why not
					self.save();
				}
			});
		});
		egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
			ui.add_space(5.0);
			ui.horizontal(|ui| {
				if ui.button(if self.playing {"⏸"} else {"▶"}).clicked() || ui.input(|i| i.key_pressed(Key::Space)) {
					self.playing = !self.playing;
					if self.playback_time < 0.0 {
						self.playback_time = 0.0;
						self.last_playback_time_tick = u32::MAX; // allow notes on beat 0 to play
					}
				}
				if ui.button("⏹").clicked() || ui.input(|i| i.key_pressed(Key::Enter)) {
					self.playing = false;
					self.playback_time = f32::MIN;
				}
				ui.add(egui::DragValue::new(&mut self.tps).speed(0.1).suffix(" TPS"));
				ui.separator();
				for i in 0..self.noteblock_sounds.len() as u8 {
					let response = ui.button(SOUND_FILE_NAMES[i as usize]);
					if (if i == self.project.layers[self.current_layer].instrument {response.highlight()} else {response}).clicked() {
						self.project.layers[self.current_layer].instrument = i;
						self.unsaved_changes = true;
						self.play_note(66, i);
					}
				}
			});
			ui.add_space(5.0);
		});
		egui::TopBottomPanel::bottom("layersbar").show(ctx, |ui| {
			ui.add_space(10.0);
			egui::ScrollArea::horizontal().show(ui, |ui| {
				ui.horizontal(|ui| {
					let mut to_delete: usize = usize::MAX;
					for (index, layer) in self.project.layers.iter_mut().enumerate() {
						let response = ui.add(egui::TextEdit::singleline(&mut layer.name).desired_width(100.0));
						if (if index == self.current_layer {response.highlight()} else {response}).clicked() {
							self.current_layer = index;
							self.selected_notes.clear();
						};
						if ui.button("x").clicked() {
							to_delete = index;
						}
						ui.separator();
					}
					if to_delete != usize::MAX && self.project.layers.len() > 1 {
						self.project.layers.remove(to_delete);
						self.selected_notes.clear();
						if self.current_layer >= to_delete {
							self.current_layer = self.current_layer.saturating_sub(1);
						}
						self.unsaved_changes = true;
					}
					if ui.button("+").clicked() {
						self.current_layer = self.project.layers.len();
						self.project.layers.push(Layer::new(format!("Layer {}", self.project.layers.len() + 1), 0));
						self.selected_notes.clear();
						self.unsaved_changes = true;
					}
				});
			});
			ui.add_space(10.0);
		});
		egui::CentralPanel::default().show(ctx, |ui| {
			ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
				ui.spacing_mut().item_spacing.x = 0.0;

				let size = ui.available_size();
				let pitch_scale = size.y * 0.04; // 1.0 / 25.0 = 0.04
				let time_scale = pitch_scale;
				let input = ui.input(|i| i.to_owned());

				{
					// player
					if self.playing {
						self.playback_time += input.stable_dt * self.tps;
					}

					let playback_tick = if self.playback_time < 0.0 {u32::MAX} else {self.playback_time as u32};

					if playback_tick != self.last_playback_time_tick {
						// Playback marker has crossed one of the beat lines
						for layer in &self.project.layers {
							for note in &layer.notes {
								if note.time == playback_tick {
									self.play_note(note.note, layer.instrument);
								}
							}
						}
					}

					self.last_playback_time_tick = playback_tick;
				}

				{
					// piano
					let (rect, response) = ui.allocate_exact_size(vec2(50.0, size.y), egui::Sense::drag());
					let painter = ui.painter_at(rect);
	
					let holding = response.dragged_by(egui::PointerButton::Primary);

					if !holding {
						self.last_played_note = 255;
					}
	
					for note in 0..128 {
						let y = rect.bottom() - ((note as f32 - self.vscroll) * pitch_scale);
						let rect2 = Rect::from_min_size(pos2(rect.left(), y - pitch_scale), vec2(50.0, pitch_scale));
						if holding && let Some(mouse_pos) = input.pointer.interact_pos() && rect2.contains(mouse_pos) {
							painter.rect_filled(rect2, 2.0, Color32::BLUE);
							if note != self.last_played_note {
								self.play_note(note, self.project.layers[self.current_layer].instrument);
								self.last_played_note = note;
							}
						} else {
							painter.rect(rect2, 2.0, Color32::from_gray(40), egui::Stroke::new(1.0, Color32::from_gray(60)), egui::StrokeKind::Inside);
						}
						painter.text(pos2(rect.left(), y), egui::Align2::LEFT_BOTTOM, Self::get_note_name(note), egui::FontId::default(),
							if note >= 54 && note <= 78
							{Color32::WHITE} else {Color32::GRAY}
						);
					}
				}
				{
					// notes
					let (rect, response) = ui.allocate_exact_size(vec2(size.x - 50.0, size.y), egui::Sense::click_and_drag());
					let painter = ui.painter_at(rect);
	
					let left = rect.left() - self.scroll * time_scale;
					// let right = left + (127.0 * time_scale);
					let bottom = rect.bottom() + self.vscroll * time_scale;
					let top = bottom - pitch_scale * 128.0;
	
					// grid
					for beat in (self.scroll as i32 - 1)..=(self.scroll as i32 + 128) {
						let x = left + (beat as f32 * time_scale);
						painter.line_segment([pos2(x, top), pos2(x, bottom)], egui::Stroke::new(1.0, Color32::from_gray(
							if beat % 16 == 0 {100} else if beat % 4 == 0 {60} else {40}
						)));
					}
					for note in 0..=128 {
						let y = bottom - (note as f32 * pitch_scale);
						painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], egui::Stroke::new(1.0, Color32::from_gray(
							if note % 12 == 0 {100} else {40}
						)));
					}
					
					// notes
					for (index, layer) in self.project.layers.iter().enumerate() {
						for (idx, note) in layer.notes.iter().enumerate() {
							let x = left + (note.time as f32 * time_scale);
							let y = bottom - ((note.note as f32) * pitch_scale);
							let rect2 = Rect::from_min_size(pos2(x, y - pitch_scale), vec2(time_scale, pitch_scale));
							painter.image(self.noteblock_texture.id(), rect2, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
								if index == self.current_layer {Color32::WHITE}
								else {Color32::from_rgba_unmultiplied( 255, 255, 255, 128 )}
							);
							if index == self.current_layer && self.selected_notes.contains(&idx) {
								painter.rect_stroke(rect2, 2.0, Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Inside);
							}
						}
					}
	
					// Playback Line
					let pblx = self.playback_time * time_scale;
					painter.line_segment([pos2(left + pblx, rect.top()), pos2(left + pblx, rect.bottom())], egui::Stroke::new(3.0, Color32::from_rgb(0, 128, 255)));
	
					// interactive
					self.vscroll -= input.smooth_scroll_delta.x * 0.05;
					self.scroll -= input.smooth_scroll_delta.y * 0.05;
					if response.dragged_by(egui::PointerButton::Middle) || response.clicked_by(egui::PointerButton::Middle) {
						// self.scroll -= response.drag_delta().x / time_scale;
						if let Some(mouse_pos) = input.pointer.interact_pos() {
							self.playback_time = ( mouse_pos.x - left ) / time_scale;
						}
					}
					if response.clicked_by(egui::PointerButton::Primary) {
						if let Some(mouse_pos) = input.pointer.interact_pos() && mouse_pos.y < bottom && mouse_pos.y > top {
							// create Note
							let time = (( mouse_pos.x - left ) / time_scale) as u32;
							let note = (( bottom - mouse_pos.y ) / pitch_scale) as u8;
							self.selected_notes.clear();
							if self.project.layers[self.current_layer].notes.iter().find(|x| x.time == time && x.note == note).is_none() {
								self.project.layers[self.current_layer].notes.push(project::Note::new( time, note ));
							};
							self.play_note(note, self.project.layers[self.current_layer].instrument);
							self.unsaved_changes = true;
						}
					} else if response.clicked_by(egui::PointerButton::Secondary) {
						if let Some(mouse_pos) = input.pointer.interact_pos() {
							let time = (( mouse_pos.x - left ) / time_scale) as u32;
							let note = (( bottom - mouse_pos.y ) / pitch_scale) as u8;
							self.selected_notes.clear();
							if let Some(found) = self.project.layers[self.current_layer].notes.iter_mut().position(|x| x.time == time && x.note == note) {
								self.project.layers[self.current_layer].notes.remove(found);
							}
							self.unsaved_changes = true;
						}
					}
					if response.dragged_by(egui::PointerButton::Primary) {
						if let Some(mouse_pos) = input.pointer.interact_pos() {
							if response.drag_started() {
								self.selection_start.x = mouse_pos.x - left;
								self.selection_start.y = bottom - mouse_pos.y;
							}
							self.selection_end.x = mouse_pos.x - left;
							self.selection_end.y = bottom - mouse_pos.y;
							painter.rect(Rect::from_points(&[pos2(left + self.selection_start.x, bottom - self.selection_start.y), mouse_pos]), 2.0, Color32::from_rgba_unmultiplied(0, 80, 255, 128), Stroke::new(2.0, Color32::from_rgb(0, 0, 255)), egui::StrokeKind::Inside);
						}
					} else if response.drag_stopped_by(egui::PointerButton::Primary) {
						let selection_rect = Rect::from_points(&[self.selection_start, self.selection_end]);
						for (index, note) in self.project.layers[self.current_layer].notes.iter().enumerate() {
							let x = note.time as f32 * time_scale;
							let y = (note.note as f32) * pitch_scale;
							let rect2 = Rect::from_min_size(pos2(x, y - pitch_scale), vec2(time_scale, pitch_scale));
							if !self.selected_notes.contains(&index) && rect2.intersects(selection_rect) {
								self.selected_notes.push(index);
							}
						}
					}
					if self.scroll < 0.0 {
						self.scroll = 0.0;
					}

					if input.modifiers.ctrl && input.key_pressed(Key::A) {
						self.selected_notes = self.project.layers[self.current_layer].notes.iter().enumerate().map(|(index, _)| index).collect();
					} else if input.key_pressed(Key::Escape) {
						self.selected_notes.clear();
					}
					if input.modifiers.ctrl && input.key_pressed(Key::D) {
						for index in &mut self.selected_notes {
							let mut note = self.project.layers[self.current_layer].notes[*index];
							note.time += 2;
							note.note += 2;
							self.project.layers[self.current_layer].notes.push(note);
							*index = self.project.layers[self.current_layer].notes.len() - 1;
						}
						self.unsaved_changes = true;
					}
					if input.key_pressed(Key::Delete) {
						self.selected_notes.sort();
						for index in self.selected_notes.iter().rev() {
							self.project.layers[self.current_layer].notes.remove(*index);
						}
						self.selected_notes.clear();
						self.unsaved_changes = true;
					}
					if input.key_pressed(Key::ArrowRight) {
						for index in &self.selected_notes {
							self.project.layers[self.current_layer].notes[*index].time = self.project.layers[self.current_layer].notes[*index].time.saturating_add(1);
						}
						self.unsaved_changes = true;
					}
					if input.key_pressed(Key::ArrowLeft) {
						for index in &self.selected_notes {
							self.project.layers[self.current_layer].notes[*index].time = self.project.layers[self.current_layer].notes[*index].time.saturating_sub(1);
						}
						self.unsaved_changes = true;
					}
					if input.key_pressed(Key::ArrowUp) {
						for index in &self.selected_notes {
							self.project.layers[self.current_layer].notes[*index].note = self.project.layers[self.current_layer].notes[*index].note.saturating_add(1);
						}
						self.unsaved_changes = true;
					}
					if input.key_pressed(Key::ArrowDown) {
						for index in &self.selected_notes {
							self.project.layers[self.current_layer].notes[*index].note = self.project.layers[self.current_layer].notes[*index].note.saturating_sub(1);
						}
						self.unsaved_changes = true;
					}

					if input.key_pressed(Key::R) {
						self.scroll = 0.0;
						self.vscroll = 54.0;
					}
				}
			});
		});

		if self.show_unsaved_changes_confirmation_dialogue_modal_because_exit {
			egui::Window::new("Unsaved Changes!!").collapsible(false).resizable(false).show(ctx, |ui| {
				ui.heading("You have unsaved changes!!! Are you sure you want to exit?");
				ui.horizontal(|ui| {
					if ui.button("Save & exit").clicked() {
						if self.save() { // onlu close window if save was succesfull
							ctx.send_viewport_cmd(egui::ViewportCommand::Close);
						}
					}
					if ui.button("Discard changes :(").clicked() {
						self.unsaved_changes = false; // Prevent preventing close
						ctx.send_viewport_cmd(egui::ViewportCommand::Close);
					}
					if ui.button("stay :)").clicked() {
						self.show_unsaved_changes_confirmation_dialogue_modal_because_exit = false;
					}
				});
			});
		}
		if self.show_unsaved_changes_confirmation_dialogue_modal_because_new {
			egui::Window::new("Unsaved Changes!!").collapsible(false).resizable(false).show(ctx, |ui| {
				ui.heading("You have unsaved changes!!! Are you sure you want to create a new project? (this will delete any unsaved changes)");
				ui.horizontal(|ui| {
					if ui.button("Save & new").clicked() {
						if self.save() { // onlu reset if save was succesfull
							self.reset();
							self.show_unsaved_changes_confirmation_dialogue_modal_because_new = false;
						}
					}
					if ui.button("Discard changes :(").clicked() {
						self.unsaved_changes = false; // Prevent preventing reset
						self.reset();
						self.show_unsaved_changes_confirmation_dialogue_modal_because_new = false;
					}
					if ui.button("Cancel").clicked() {
						self.show_unsaved_changes_confirmation_dialogue_modal_because_new = false;
					}
				});
			});
		}

		ctx.send_viewport_cmd(egui::ViewportCommand::Title(if self.unsaved_changes {"Note Block Music*".to_string()} else {"Note Block Music".to_string()}));
	}
}