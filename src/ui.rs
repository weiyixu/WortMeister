// All egui rendering for the app: top bar, filters, home dashboard, learning
// and dictation cards, and the vocabulary browser.

use eframe::egui::{self, Color32, RichText};

use crate::app::{App, Mode};
use crate::srs::Grade;
use crate::util::{norm, today};

/// Accent color used for headings and highlighted German words.
const ACCENT: Color32 = Color32::from_rgb(35, 90, 150);

/// Application version, taken from Cargo.toml at compile time.
const VERSION: &str = env!("CARGO_PKG_VERSION");

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

            // Right-aligned Help menu with an About item.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.menu_button("帮助", |ui| {
                    if ui.button("关于").clicked() {
                        self.show_about = true;
                        ui.close_menu();
                    }
                });
            });
        });
        ui.separator();
    }

    /// Modal "About" window showing the app name, version and a short blurb.
    fn about_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_about;
        egui::Window::new("关于")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.heading(RichText::new("Deutsch Worttrainer").color(ACCENT));
                ui.label(format!("版本 {VERSION}"));
                ui.add_space(8.0);
                ui.label("本地离线德语词汇学习与默写工具。");
                ui.label("基于 SM-2 间隔重复算法。");
                ui.add_space(8.0);
                ui.label("作者：wei.y.xu@gmail.com");
                ui.add_space(8.0);
                ui.separator();
                ui.small("词库存储于 vocabulary.csv，学习进度存储于 progress.json。");
            });
        self.show_about = open;
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

        // Classify every word inside the current filter (level / lesson) into
        // three mutually exclusive buckets so the numbers add up to the total:
        //   - new:      no learning record yet
        //   - due:      studied, and the review date has arrived (<= today)
        //   - mastered: studied, but scheduled for a future date
        let (mut new_n, mut due_n, mut mastered_n) = (0usize, 0usize, 0usize);
        for (i, w) in self.words.iter().enumerate() {
            if !self.filtered(i) {
                continue;
            }
            match self.progress.cards.get(&w.id) {
                None => new_n += 1,
                Some(p) if p.due <= t => due_n += 1,
                Some(_) => mastered_n += 1,
            }
        }
        let total_n = new_n + due_n + mastered_n;

        let today_n = self
            .progress
            .daily_counts
            .get(&t.to_string())
            .copied()
            .unwrap_or(0);

        ui.columns(4, |c| {
            c[0].heading(format!("{total_n}"));
            c[0].label("筛选范围");
            c[1].heading(format!("{new_n}"));
            c[1].label("未学新词");
            c[2].heading(format!("{due_n}"));
            c[2].label("今日待复习");
            c[3].heading(format!("{mastered_n}"));
            c[3].label("已掌握");
        });
        ui.small("三者之和等于筛选范围内的词条数。切换级别 / 课次可改变统计范围。");

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

        ui.add_space(20.0);
        ui.separator();
        self.stats(ui);
    }

    /// Learning statistics: overall accuracy and a simple 7-day review bar
    /// chart derived from the persisted daily counts.
    fn stats(&mut self, ui: &mut egui::Ui) {
        ui.heading("学习统计");
        ui.add_space(6.0);

        // Overall accuracy across all graded cards.
        let (correct, wrong) = self
            .progress
            .cards
            .values()
            .fold((0u32, 0u32), |(c, w), p| (c + p.correct, w + p.wrong));
        let total = correct + wrong;
        let accuracy = if total > 0 {
            correct as f32 / total as f32 * 100.0
        } else {
            0.0
        };
        ui.label(format!(
            "累计答对 {correct} · 答错 {wrong} · 正确率 {accuracy:.0}%"
        ));

        ui.add_space(10.0);
        ui.label("最近 7 天复习次数");
        ui.add_space(4.0);

        // Gather counts for the last 7 days in chronological order
        // (oldest -> newest). Iterating d from 6 down to 0 yields today - 6
        // days first and today last.
        let today = today();
        let days: Vec<(String, u32)> = (0..7)
            .rev()
            .map(|d| {
                let date = today - chrono::Duration::days(d);
                let key = date.to_string();
                let count = self.progress.daily_counts.get(&key).copied().unwrap_or(0);
                (date.format("%m-%d").to_string(), count)
            })
            .collect();

        let max = days.iter().map(|(_, n)| *n).max().unwrap_or(0).max(1);
        for (label, count) in &days {
            ui.horizontal(|ui| {
                ui.monospace(label);
                ui.add(
                    egui::ProgressBar::new(*count as f32 / max as f32)
                        .desired_width(260.0)
                        .text(format!("{count}")),
                );
            });
        }
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
            ui.horizontal(|ui| {
                ui.heading(&w.chinese);
                if self.tts.is_some() && ui.button("🔊 播放读音").clicked() {
                    self.speak(&w.german);
                }
            });
            ui.checkbox(&mut self.speak_on_dictation, "出题时自动播放读音");
            ui.label(&w.example_zh);
            ui.add_space(8.0);

            // Auto-play the German word once per card when enabled, tracking the
            // queue position so it does not repeat on every UI frame.
            if self.speak_on_dictation
                && self.tts.is_some()
                && self.spoken_pos != Some(self.pos)
            {
                self.speak(&w.german);
                self.spoken_pos = Some(self.pos);
            }

            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.input)
                    .hint_text("请输入德语单词，名词包含冠词")
                    .desired_width(420.0),
            );
            // Auto-focus the input so the user can type immediately, until the
            // answer has been checked (so grade shortcuts are not swallowed).
            if !self.revealed {
                resp.request_focus();
            }
            let enter =
                resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if enter || ui.button("检查答案").clicked() {
                self.feedback = Some(norm(&self.input) == norm(&w.german));
                self.revealed = true;
            }
        } else {
            ui.horizontal(|ui| {
                ui.heading(&w.german);
                if self.tts.is_some() && ui.button("🔊 播放读音").clicked() {
                    self.speak(&w.german);
                }
            });
            ui.checkbox(&mut self.speak_on_dictation, "显示单词时自动播放读音");

            // Auto-play the German word once per card when enabled, tracking the
            // queue position so it does not repeat on every UI frame.
            if self.speak_on_dictation
                && self.tts.is_some()
                && self.spoken_pos != Some(self.pos)
            {
                self.speak(&w.german);
                self.spoken_pos = Some(self.pos);
            }

            if !self.revealed {
                // Enter is equivalent to clicking the reveal button.
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                if ui.button("显示释义与例句").clicked() || enter {
                    self.revealed = true;
                }
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
                if ui.button("重来 (1)").clicked() {
                    self.grade(Grade::Again);
                }
                if ui.button("困难 (2)").clicked() {
                    self.grade(Grade::Hard);
                }
                if ui.button("记住了 (3)").clicked() {
                    self.grade(Grade::Good);
                }
                if ui.button("很容易 (4)").clicked() {
                    self.grade(Grade::Easy);
                }
            });
            ui.add_space(4.0);
            ui.small("快捷键：1 重来 · 2 困难 · 3 记住了 · 4 很容易");

            // Keyboard shortcuts 1..=4 mirror the grade buttons. Only active
            // once the answer is revealed and the grade buttons are shown.
            if let Some(g) = ui.input(|i| {
                if i.key_pressed(egui::Key::Num1) {
                    Some(Grade::Again)
                } else if i.key_pressed(egui::Key::Num2) {
                    Some(Grade::Hard)
                } else if i.key_pressed(egui::Key::Num3) {
                    Some(Grade::Good)
                } else if i.key_pressed(egui::Key::Num4) {
                    Some(Grade::Easy)
                } else {
                    None
                }
            }) {
                self.grade(g);
            }
        }
    }

    /// Scrollable list of all filtered vocabulary entries, with a search box
    /// that matches against German, Chinese and tags.
    fn browse(&mut self, ui: &mut egui::Ui) {
        self.filters(ui);
        ui.horizontal(|ui| {
            ui.label("搜索");
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("德语 / 中文 / 标签")
                    .desired_width(300.0),
            );
            if ui.button("清除").clicked() {
                self.search.clear();
            }
        });
        ui.separator();

        let query = self.search.trim().to_lowercase();
        let mut shown = 0usize;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, w) in self.words.iter().enumerate() {
                if !self.filtered(i) {
                    continue;
                }
                if !query.is_empty()
                    && !w.german.to_lowercase().contains(&query)
                    && !w.chinese.to_lowercase().contains(&query)
                    && !w.tags.to_lowercase().contains(&query)
                {
                    continue;
                }
                shown += 1;
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
            if shown == 0 {
                ui.label("没有匹配的词条。");
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

        if self.show_about {
            self.about_window(ctx);
        }
    }
}
