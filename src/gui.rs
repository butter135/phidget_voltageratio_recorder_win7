use crate::model::{Command, RecordingMode, Sample, UiEvent};
use crate::plot;

use native_windows_gui as nwg;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub fn run(cmd_tx: Sender<Command>, ui_rx: Receiver<UiEvent>) {
    nwg::init().expect("GUI initializaation failed");

    let mut window = nwg::Window::default();

    nwg::Window::builder()
        .title("Phidget Recorder Win7")
        .size((520, 600))
        .position((300, 300))
        .build(&mut window)
        .unwrap();

    let mut start_button = nwg::Button::default();
    let mut stop_button = nwg::Button::default();

    nwg::Button::builder()
        .text("START")
        .position((20, 10))
        .size((100, 50))
        .parent(&window)
        .build(&mut start_button)
        .unwrap();

    nwg::Button::builder()
        .text("STOP")
        .position((140, 10))
        .size((100, 50))
        .parent(&window)
        .build(&mut stop_button)
        .unwrap();

    stop_button.set_enabled(false);

    let mut mode_label = nwg::Label::default();

    nwg::Label::builder()
        .text("Recording Mode")
        .position((260, 10))
        .size((140, 18))
        .parent(&window)
        .build(&mut mode_label)
        .unwrap();

    let mut mode_combo = nwg::ComboBox::<&'static str>::default();

    nwg::ComboBox::builder()
        .collection(vec!["Round", "Raw", "Truncate"])
        .selected_index(Some(0))
        .position((260, 30))
        .size((140, 110))
        .parent(&window)
        .build(&mut mode_combo)
        .unwrap();

    let mut value_label = nwg::Label::default();

    nwg::Label::builder()
        .text("VoltageRatio: ---")
        .position((20, 75))
        .size((450, 25))
        .parent(&window)
        .build(&mut value_label)
        .unwrap();

    let mut count_label = nwg::Label::default();

    nwg::Label::builder()
        .text("Samples: 0")
        .position((20, 105))
        .size((450, 25))
        .parent(&window)
        .build(&mut count_label)
        .unwrap();

    let mut canvas = nwg::ExternCanvas::default();

    nwg::ExternCanvas::builder()
        .position((20, 150))
        .size((470, 200))
        .parent(Some(&window))
        .build(&mut canvas)
        .unwrap();

    let mut messages = nwg::TextBox::default();

    nwg::TextBox::builder()
        .text("Ready\r\n")
        .position((20, 380))
        .size((470, 150))
        .readonly(true)
        .parent(&window)
        .build(&mut messages)
        .unwrap();

    let mut timer = nwg::AnimationTimer::default();

    nwg::AnimationTimer::builder()
        .parent(&window)
        .interval(Duration::from_millis(20))
        .build(&mut timer)
        .unwrap();

    let plot_samples = Rc::new(RefCell::new(VecDeque::<Sample>::new()));

    let start_handle = start_button.handle.clone();
    let stop_handle = stop_button.handle.clone();
    let timer_handle = timer.handle.clone();
    let canvas_handle = canvas.handle.clone();
    let window_handle = window.handle.clone();

    let handler =
        nwg::full_bind_event_handler(
            &window.handle,
            move |event, event_data, handle| match event {
                nwg::Event::OnButtonClick if handle == start_handle => {
                    let mode = match mode_combo.selection() {
                        Some(0) => RecordingMode::Round,
                        Some(1) => RecordingMode::Raw,
                        Some(2) => RecordingMode::Truncate,
                        _ => RecordingMode::Round,
                    };
                    if cmd_tx.send(Command::Start(mode)).is_ok() {
                        start_button.set_enabled(false);
                        mode_combo.set_enabled(false);
                        stop_button.set_enabled(false);
                        plot_samples.borrow_mut().clear();
                        canvas.invalidate();
                        messages.appendln("Connecting...");
                    }
                }

                nwg::Event::OnButtonClick if handle == stop_handle => {
                    if cmd_tx.send(Command::Stop).is_ok() {
                        stop_button.set_enabled(false);
                        messages.appendln("Stopping...");
                    }
                }

                nwg::Event::OnPaint if handle == canvas_handle => {
                    let paint = event_data.on_paint();
                    let ps = paint.begin_paint();

                    let (w, h) = canvas.physical_size();

                    {
                        let data = plot_samples.borrow();

                        plot::draw(ps.hdc, w as i32, h as i32, &data);
                    }

                    paint.end_paint(&ps);
                }

                nwg::Event::OnTimerTick if handle == timer_handle => {
                    for notification in ui_rx.try_iter() {
                        match notification {
                            UiEvent::Started => {
                                messages.appendln("Recording started");
                                stop_button.set_enabled(true);
                            }

                            UiEvent::Stopped => {
                                messages.appendln("Recording stopped");
                            }

                            UiEvent::Progress {
                                count,
                                ratio,
                                new_samples,
                            } => {
                                count_label.set_text(&format!("Samples: {}", count));

                                value_label.set_text(&format!("VoltageRatio: {:.5}", ratio));

                                {
                                    let mut data = plot_samples.borrow_mut();

                                    data.extend(new_samples);

                                    if let Some(last) = data.back() {
                                        let cutoff = last.time - 5.0;
                                        while data.front().map_or(false, |s| s.time < cutoff) {
                                            data.pop_front();
                                        }
                                    }
                                }

                                canvas.invalidate();
                            }

                            UiEvent::Saved(filename) => {
                                messages.appendln(&format!("Saved: {}", filename));

                                start_button.set_enabled(true);
                                mode_combo.set_enabled(true);
                            }

                            UiEvent::Error(error) => {
                                messages.appendln(&format!("Error: {}", error));

                                start_button.set_enabled(true);
                                stop_button.set_enabled(false);
                                mode_combo.set_enabled(true);
                            }
                        }
                    }
                }

                nwg::Event::OnWindowClose if handle == window_handle => {
                    let _ = cmd_tx.send(Command::Exit);
                    nwg::stop_thread_dispatch();
                }

                _ => {}
            },
        );

    timer.start();

    nwg::dispatch_thread_events();

    timer.stop();
    nwg::unbind_event_handler(&handler);
}
