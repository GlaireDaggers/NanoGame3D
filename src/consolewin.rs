use std::sync::RwLock;

use imgui::{InputTextCallbackHandler, Ui};
use lazy_static::lazy_static;
use log::Level;

lazy_static! {
    static ref LOG_HISTORY: RwLock<Vec<LogItem>> = RwLock::new(Vec::new());
}

enum LogItemType {
    Info,
    Warn,
    Error,
    Command,
}

struct LogItem {
    item_type: LogItemType,
    item: String,
}

impl LogItem {
    pub fn new(item_type: LogItemType, item: String) -> LogItem {
        LogItem { item_type, item }
    }
}

pub struct ConsoleWindow {
    item_spacing_height: f32,
    cmd_string: String,
    scroll_to_bottom: bool,
    history: Vec<String>,
    history_pos: i32,
    exec_queue: Vec<String>,
}

pub struct ConsoleWindowLogger {
}

struct ConsoleWindowCallbackHandler<'a> {
    history: &'a [String],
    history_pos: &'a mut i32,
}

impl<'a> InputTextCallbackHandler for ConsoleWindowCallbackHandler<'a> {
    fn on_history(&mut self, dir: imgui::HistoryDirection, mut data: imgui::TextCallbackData) {
        if self.history.len() == 0 {
            return;
        }

        let prev_history_pos = *self.history_pos;

        match dir {
            imgui::HistoryDirection::Up => {
                if *self.history_pos == -1 {
                    *self.history_pos = (self.history.len() - 1) as i32;
                }
                else if *self.history_pos > 0 {
                    *self.history_pos -= 1;
                }
            },
            imgui::HistoryDirection::Down => {
                if *self.history_pos != -1 {
                    *self.history_pos += 1;

                    if *self.history_pos == self.history.len() as i32 {
                        *self.history_pos = -1;
                    }
                }
            },
        }

        if *self.history_pos != prev_history_pos {
            data.clear();

            if *self.history_pos != -1 {
                data.insert_chars(0, &self.history[*self.history_pos as usize]);
            }
        }
    }
}

impl ConsoleWindow {
    pub fn new(imgui: &imgui::Context) -> ConsoleWindow {
        ConsoleWindow {
            item_spacing_height: imgui.style().item_spacing[1],
            cmd_string: String::new(),
            scroll_to_bottom: false,
            history: Vec::new(),
            history_pos: -1,
            exec_queue: Vec::new(),
        }
    }

    pub fn drain_commands(self: &mut Self) -> std::vec::Drain<'_, String> {
        self.exec_queue.drain(..)
    }

    pub fn draw(self: &mut Self, win_size: (f32, f32), ui: &mut Ui) {
        let overlay_flags = imgui::WindowFlags::NO_DECORATION |
            imgui::WindowFlags::ALWAYS_AUTO_RESIZE |
            imgui::WindowFlags::NO_SAVED_SETTINGS |
            imgui::WindowFlags::NO_FOCUS_ON_APPEARING |
            imgui::WindowFlags::NO_NAV;

        if let Some(console_win) = ui.window("CONSOLE")
            .position([0.0, 0.0], imgui::Condition::Always)
            .size([win_size.0, win_size.1 / 2.0], imgui::Condition::Always)
            .bg_alpha(1.0)
            .flags(overlay_flags)
            .begin()
        {
            let footer_height_to_reserve = self.item_spacing_height + ui.frame_height_with_spacing();
            if let Some(scroll_area) = ui.child_window("CONSOLE_SCROLL_REGION")
                .size([0.0, -footer_height_to_reserve])
                .begin()
            {
                // draw items
                let item_spacing = ui.push_style_var(imgui::StyleVar::ItemSpacing([4.0, 1.0]));
                let items = LOG_HISTORY.read().unwrap();
                for item in items.iter() {
                    let item_color = match item.item_type {
                        LogItemType::Info => ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 1.0, 1.0]),
                        LogItemType::Warn => ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 0.0, 1.0]),
                        LogItemType::Error => ui.push_style_color(imgui::StyleColor::Text, [1.0, 0.1, 0.1, 1.0]),
                        LogItemType::Command => ui.push_style_color(imgui::StyleColor::Text, [1.0, 1.0, 1.0, 0.5]),
                    };
                    ui.text_wrapped(&item.item);
                    item_color.pop();
                }
                item_spacing.pop();

                // scroll to bottom
                if self.scroll_to_bottom || ui.scroll_y() >= ui.scroll_max_y() {
                    ui.set_scroll_here_y_with_ratio(1.0);
                    self.scroll_to_bottom = false;
                }

                scroll_area.end();
            }

            ui.separator();

            let cmd_callback_handler = ConsoleWindowCallbackHandler {
                history: &self.history,
                history_pos: &mut self.history_pos
            };

            ui.set_keyboard_focus_here();
            ui.set_next_item_width(-1.0);
            if ui.input_text("##CMD_INPUT", &mut self.cmd_string)
                .enter_returns_true(true)
                .callback(imgui::InputTextCallback::HISTORY, cmd_callback_handler)
                .build()
            {
                self.history.push(self.cmd_string.clone());
                self.exec_queue.push(self.cmd_string.clone());
                
                let mut log_history = LOG_HISTORY.write().unwrap();
                log_history.push(LogItem::new(LogItemType::Command, format!(">> {}", self.cmd_string)));

                self.scroll_to_bottom = true;
                self.cmd_string.clear();
                self.history_pos = -1;
            }

            console_win.end();
        }
    }
}

impl log::Log for ConsoleWindowLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());

            let mut log_history = LOG_HISTORY.write().unwrap();

            let log_type = match record.level() {
                Level::Error => LogItemType::Error,
                Level::Warn => LogItemType::Warn,
                Level::Info => LogItemType::Info,
                Level::Debug => LogItemType::Info,
                Level::Trace => LogItemType::Info,
            };

            log_history.push(LogItem::new(log_type, format!("{}", record.args())));
        }
    }

    fn flush(&self) {
    }
}