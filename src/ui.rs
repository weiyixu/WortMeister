// All egui rendering for the app: top bar, filters, home dashboard, learning
// and dictation cards, and the vocabulary browser.

use eframe::egui::{self, Color32, RichText};

use crate::app::{App, Mode};
use crate::srs::Grade;
use crate::util::{norm, today};

/// Accent color used for headings and highlighted German words.
const ACCENT: Color32 = Color32::from_rgb(35, 90, 150);

impl App {
    /// Top navigation bar with title and mode tabs.
    fn topbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Deutsch Worttrainer").color(ACCENT));
            ui.separator();
            let tabs = [
                (Mode::Home, "首页"),
                (Mode::Learn, "背诵"),
                (Mode::Dictation, "默写"),
                (Mode::Browse, "词库"),
            ];
            for (m, t) in tabs {
                if ui.selectable_label(self.mode == m, t).clicked() {
                    if m == Mode::Learn || m == Mode::Dictation {
                        self.make_queue(m);
                    } else {
                        self.mode = m;
                    }
                }
            }
        });
        ui.separator();
    }

    /// Level / lesson / daily-limit filter row.
    fn filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("级别");
            egui::ComboBox::from_id_salt("level")
                .selected_text(&self.level)
                .show_ui(ui, |ui| {
                    for x in self.levels() {
                        ui.selectable_value(&mut self.level, x.clone(), x);
                    }
                });

            ui.label("课次");
            egui::ComboBox::from_id_salt("lesson")
                .selected_text(&self.lesson)
                .show_ui(ui, |ui| {
                    for x in self.lessons() {
                        ui.selectable_value(&mut self.lesson, x.clone(), x);
                    }
                });

            ui.label("每日数量");
            ui.add(egui::DragValue::new(&mut self.daily_limit).range(5..=100));
        });
    }

    /// Home dashboard: stats and start buttons.
    fn home(&mut self, ui: &mut egui::Ui) {
        self.filters(ui);
        ui.add_space(16.0);

        let t = today();
        let learned = self
            .progress
            .cards
            .values()
            .filter(|p| p.repetitions > 0)
            .count();
        let due = self
            .words
            .iter()
            .enumerate()
            .filter(|(i, w)| {
                self.filtered(*i)
                    && self
                        .progress
                        .cards
                        .get(&w.id)
                        .map(|p| p.due <= t)
                        .unwrap_or(true)
            })
            .count();
        let today_n = self
            .progress
            .daily_counts
            .get(&t.to_string())
            .copied()
            .unwrap_or(0);

        ui.columns(3, |c| {
            c[0].heading(format!("{}", self.words.len()));
            c[0].label("词库词条");
            c[1].heading(format!("{learned}"));
            c[1].label("已学习");
            c[2].heading(format!("{due}"));
            c[2].label("今日到期");
        });

        ui.add_space(15.0);
        ui.label(format!("今天已复习 {today_n} 次"));
        ui.add(
            egui::ProgressBar::new((today_n as f32 / self.daily_limit as f32).min(1.0))
                .show_percentage(),
        );

        ui.add_space(15.0);
        ui.horizontal(|ui| {
            if ui.button("开始今日背诵").clicked() {
                self.make_queue(Mode::Learn);
            }
            if ui.button("开始今日默写").clicked() {
                self.make_queue(Mode::Dictation);
            }
        });
    }

    /// A single learning or dictation card.
    fn card(&mut self, ui: &mut egui::Ui, dictation: bool) {
        if self.pos >= self.queue.len() {
            ui.heading("本轮完成！");
            ui.label("进度已经保存到 progress.json。");
            if ui.button("返回首页").clicked() {
                self.mode = Mode::Home;
            }
            return;
        }

        let i = self.queue[self.pos];
        let w = self.words[i].clone();

        ui.label(format!(
            "{}/{}  |  {}  |  {}",
            self.pos + 1,
            self.queue.len(),
            w.level,
            w.lesson
        ));
        ui.add(egui::ProgressBar::new(
            self.pos as f32 / self.queue.len() as f32,
        ));
        ui.add_space(20.0);

        if dictation {
            ui.heading(&w.chinese);
            ui.label(&w.example_zh);
            ui.add_space(8.0);

            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("请输入德语单词，名词包含冠词")
                    .desired_width(420.0),
            );
            let enter =
                resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if enter || ui.button("检查答案").clicked() {
                self.feedback = Some(norm(&self.input) == norm(&w.german));
                self.revealed = true;
            }
        } else {
            ui.heading(&w.german);
            if !self.revealed && ui.button("显示释义与例句").clicked() {
                self.revealed = true;
            }
        }

        if self.revealed {
            ui.separator();
            ui.heading(RichText::new(&w.german).color(ACCENT));
            ui.label(RichText::new(&w.chinese).size(18.0));
            ui.add_space(6.0);
            ui.label(RichText::new(&w.example).italics());
            ui.label(&w.example_zh);

            if let Some(ok) = self.feedback {
                let msg = if ok {
                    RichText::new("✓ 正确").color(Color32::DARK_GREEN)
                } else {
                    RichText::new(format!("✗ 你的答案：{}", self.input))
                        .color(Color32::DARK_RED)
                };
                ui.label(msg);
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("重来").clicked() {
                    self.grade(Grade::Again);
                }
                if ui.button("困难").clicked() {
                    self.grade(Grade::Hard);
                }
                if ui.button("记住了").clicked() {
                    self.grade(Grade::Good);
                }
                if ui.button("很容易").clicked() {
                    self.grade(Grade::Easy);
                }
            });
        }
    }

    /// Scrollable list of all filtered vocabulary entries.
    fn browse(&mut self, ui: &mut egui::Ui) {
        self.filters(ui);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, w) in self.words.iter().enumerate() {
                if !self.filtered(i) {
                    continue;
                }
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.strong(&w.german);
                        ui.label(&w.chinese);
                        ui.label(format!("{} / {}", w.level, w.lesson));
                    });
                    ui.label(&w.example);
                    if !w.tags.is_empty() {
                        ui.small(format!("标签: {}", w.tags));
                    }
                });
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.topbar(ui);
            if !self.notice.is_empty() {
                ui.colored_label(Color32::DARK_RED, &self.notice);
                ui.separator();
            }
            match self.mode {
                Mode::Home => self.home(ui),
                Mode::Learn => self.card(ui, false),
                Mode::Dictation => self.card(ui, true),
                Mode::Browse => self.browse(ui),
            }
        });
    }
}
