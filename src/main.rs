#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{Duration, Local, NaiveDate};
use eframe::egui::{self, Color32, RichText};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::{Path, PathBuf}};
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Debug, Deserialize)]
struct Word {
    id: String,
    level: String,
    lesson: String,
    german: String,
    chinese: String,
    example: String,
    example_zh: String,
    tags: String,
    #[serde(default = "default_true")]
    enabled: bool,
}
fn default_true() -> bool { true }

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CardProgress {
    repetitions: u32,
    interval_days: i64,
    ease: f32,
    due: NaiveDate,
    correct: u32,
    wrong: u32,
    last_review: Option<NaiveDate>,
}
impl CardProgress {
    fn new() -> Self { Self { repetitions: 0, interval_days: 0, ease: 2.5, due: today(), correct: 0, wrong: 0, last_review: None } }
}
#[derive(Default, Serialize, Deserialize)]
struct ProgressFile {
    cards: HashMap<String, CardProgress>,
    daily_counts: HashMap<String, u32>,
}
#[derive(Clone, Copy, PartialEq)]
enum Mode { Home, Learn, Dictation, Browse }
#[derive(Clone, Copy)]
enum Grade { Again, Hard, Good, Easy }

struct App {
    words: Vec<Word>, progress: ProgressFile, progress_path: PathBuf,
    mode: Mode, level: String, lesson: String, daily_limit: usize,
    queue: Vec<usize>, pos: usize, revealed: bool, input: String,
    feedback: Option<bool>, notice: String,
}

fn today() -> NaiveDate { Local::now().date_naive() }
fn norm(s: &str) -> String {
    let punctuation = ['.', ',', '!', '?', ';', ':', '"', '\'', '„', '“'];
    s.trim().to_lowercase().nfc().filter(|c| !punctuation.contains(c))
        .collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ")
}
fn app_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok()).unwrap_or_else(|| PathBuf::from("."))
}
fn load_words(base: &Path) -> Result<Vec<Word>, String> {
    let candidates = [base.join("vocabulary.csv"), base.join("data").join("vocabulary.csv"), PathBuf::from("data/vocabulary.csv")];
    let path = candidates.iter().find(|p| p.exists()).ok_or("找不到 vocabulary.csv。请把它放在程序旁边或 data 文件夹中。")?;
    let mut rdr = csv::ReaderBuilder::new().flexible(true).from_path(path).map_err(|e| e.to_string())?;
    let mut words = Vec::new();
    for row in rdr.deserialize() { let w: Word = row.map_err(|e| format!("CSV 格式错误: {e}"))?; if w.enabled { words.push(w); } }
    if words.is_empty() { return Err("CSV 中没有启用的词条。".into()); }
    Ok(words)
}
impl App {
    fn new() -> Self {
        let base = app_dir(); let progress_path = base.join("progress.json");
        let progress = fs::read_to_string(&progress_path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        match load_words(&base) {
            Ok(words) => Self { words, progress, progress_path, mode: Mode::Home, level:"全部".into(), lesson:"全部".into(), daily_limit:20, queue:vec![], pos:0, revealed:false, input:String::new(), feedback:None, notice:String::new() },
            Err(e) => Self { words:vec![], progress, progress_path, mode:Mode::Home, level:"全部".into(), lesson:"全部".into(), daily_limit:20, queue:vec![], pos:0, revealed:false, input:String::new(), feedback:None, notice:e },
        }
    }
    fn save(&mut self) { if let Ok(s)=serde_json::to_string_pretty(&self.progress) { if let Err(e)=fs::write(&self.progress_path,s) { self.notice=format!("保存失败: {e}"); } } }
    fn filtered(&self, i:usize) -> bool { (self.level=="全部" || self.words[i].level==self.level) && (self.lesson=="全部" || self.words[i].lesson==self.lesson) }
    fn make_queue(&mut self, mode:Mode) {
        let t=today(); let mut due:Vec<usize>=(0..self.words.len()).filter(|&i| self.filtered(i) && self.progress.cards.get(&self.words[i].id).map(|p|p.due<=t).unwrap_or(true)).collect();
        due.shuffle(&mut rand::thread_rng()); due.truncate(self.daily_limit);
        self.queue=due; self.pos=0; self.mode=mode; self.revealed=false; self.input.clear(); self.feedback=None;
        self.notice=if self.queue.is_empty(){"今天没有到期词条。可以扩大筛选范围，或明天再复习。".into()}else{String::new()};
    }
    fn grade(&mut self, g:Grade) {
        if self.pos>=self.queue.len(){return} let id=self.words[self.queue[self.pos]].id.clone(); let p=self.progress.cards.entry(id).or_insert_with(CardProgress::new);
        match g { Grade::Again=>{p.repetitions=0;p.interval_days=1;p.ease=(p.ease-0.2).max(1.3);p.wrong+=1}, Grade::Hard=>{p.repetitions+=1;p.interval_days=(p.interval_days.max(1) as f32*1.2).round() as i64;p.ease=(p.ease-0.15).max(1.3);p.correct+=1}, Grade::Good=>{p.repetitions+=1;p.interval_days=if p.repetitions==1{1}else if p.repetitions==2{3}else{(p.interval_days as f32*p.ease).round().max(1.0) as i64};p.correct+=1}, Grade::Easy=>{p.repetitions+=1;p.ease=(p.ease+0.15).min(3.2);p.interval_days=if p.repetitions<=1{4}else{(p.interval_days as f32*p.ease*1.3).round().max(4.0) as i64};p.correct+=1} }
        p.last_review=Some(today()); p.due=today()+Duration::days(p.interval_days);
        *self.progress.daily_counts.entry(today().to_string()).or_insert(0)+=1; self.save(); self.pos+=1; self.revealed=false;self.input.clear();self.feedback=None;
    }
    fn levels(&self)->Vec<String>{let mut v:Vec<_>=self.words.iter().map(|w|w.level.clone()).collect();v.sort();v.dedup();v.insert(0,"全部".into());v}
    fn lessons(&self)->Vec<String>{let mut v:Vec<_>=self.words.iter().filter(|w|self.level=="全部"||w.level==self.level).map(|w|w.lesson.clone()).collect();v.sort();v.dedup();v.insert(0,"全部".into());v}
    fn topbar(&mut self,ui:&mut egui::Ui){ui.horizontal(|ui|{ui.heading(RichText::new("Deutsch Worttrainer").color(Color32::from_rgb(35,90,150)));ui.separator();for(m,t)in[(Mode::Home,"首页"),(Mode::Learn,"背诵"),(Mode::Dictation,"默写"),(Mode::Browse,"词库")]{if ui.selectable_label(self.mode==m,t).clicked(){if m==Mode::Learn||m==Mode::Dictation{self.make_queue(m)}else{self.mode=m}}}});ui.separator();}
    fn filters(&mut self,ui:&mut egui::Ui){ui.horizontal(|ui|{ui.label("级别");egui::ComboBox::from_id_salt("level").selected_text(&self.level).show_ui(ui,|ui|for x in self.levels(){ui.selectable_value(&mut self.level,x.clone(),x);});ui.label("课次");egui::ComboBox::from_id_salt("lesson").selected_text(&self.lesson).show_ui(ui,|ui|for x in self.lessons(){ui.selectable_value(&mut self.lesson,x.clone(),x);});ui.label("每日数量");ui.add(egui::DragValue::new(&mut self.daily_limit).range(5..=100));});}
    fn home(&mut self,ui:&mut egui::Ui){self.filters(ui);ui.add_space(16.0);let t=today();let learned=self.progress.cards.values().filter(|p|p.repetitions>0).count();let due=self.words.iter().enumerate().filter(|(i,w)|self.filtered(*i)&&self.progress.cards.get(&w.id).map(|p|p.due<=t).unwrap_or(true)).count();let today_n=self.progress.daily_counts.get(&t.to_string()).copied().unwrap_or(0);ui.columns(3,|c|{c[0].heading(format!("{}",self.words.len()));c[0].label("词库词条");c[1].heading(format!("{learned}"));c[1].label("已学习");c[2].heading(format!("{due}"));c[2].label("今日到期");});ui.add_space(15.0);ui.label(format!("今天已复习 {today_n} 次"));ui.add(egui::ProgressBar::new((today_n as f32/self.daily_limit as f32).min(1.0)).show_percentage());ui.add_space(15.0);ui.horizontal(|ui|{if ui.button("开始今日背诵").clicked(){self.make_queue(Mode::Learn)}if ui.button("开始今日默写").clicked(){self.make_queue(Mode::Dictation)}});}
    fn card(&mut self,ui:&mut egui::Ui,dictation:bool){if self.pos>=self.queue.len(){ui.heading("本轮完成！");ui.label("进度已经保存到 progress.json。");if ui.button("返回首页").clicked(){self.mode=Mode::Home}return}let i=self.queue[self.pos];let w=self.words[i].clone();ui.label(format!("{}/{}  |  {}  |  {}",self.pos+1,self.queue.len(),w.level,w.lesson));ui.add(egui::ProgressBar::new(self.pos as f32/self.queue.len() as f32));ui.add_space(20.0);if dictation{ui.heading(&w.chinese);ui.label(&w.example_zh);ui.add_space(8.0);let resp=ui.add(egui::TextEdit::singleline(&mut self.input).hint_text("请输入德语单词，名词包含冠词").desired_width(420.0));if resp.lost_focus()&&ui.input(|i|i.key_pressed(egui::Key::Enter)){self.feedback=Some(norm(&self.input)==norm(&w.german));self.revealed=true}if ui.button("检查答案").clicked(){self.feedback=Some(norm(&self.input)==norm(&w.german));self.revealed=true}}else{ui.heading(&w.german);if !self.revealed&&ui.button("显示释义与例句").clicked(){self.revealed=true}}
        if self.revealed{ui.separator();ui.heading(RichText::new(&w.german).color(Color32::from_rgb(35,90,150)));ui.label(RichText::new(&w.chinese).size(18.0));ui.add_space(6.0);ui.label(RichText::new(&w.example).italics());ui.label(&w.example_zh);if let Some(ok)=self.feedback{ui.label(if ok{RichText::new("✓ 正确").color(Color32::DARK_GREEN)}else{RichText::new(format!("✗ 你的答案：{}",self.input)).color(Color32::DARK_RED)});}ui.add_space(12.0);ui.horizontal(|ui|{if ui.button("重来").clicked(){self.grade(Grade::Again)}if ui.button("困难").clicked(){self.grade(Grade::Hard)}if ui.button("记住了").clicked(){self.grade(Grade::Good)}if ui.button("很容易").clicked(){self.grade(Grade::Easy)}});}}
    fn browse(&mut self,ui:&mut egui::Ui){self.filters(ui);ui.separator();egui::ScrollArea::vertical().show(ui,|ui|for (i,w) in self.words.iter().enumerate(){if self.filtered(i){ui.group(|ui|{ui.horizontal(|ui|{ui.strong(&w.german);ui.label(&w.chinese);ui.label(format!("{} / {}",w.level,w.lesson));});ui.label(&w.example);if !w.tags.is_empty(){ui.small(format!("标签: {}",w.tags));}});}});}
}
impl eframe::App for App {fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){egui::CentralPanel::default().show(ctx,|ui|{self.topbar(ui);if !self.notice.is_empty(){ui.colored_label(Color32::DARK_RED,&self.notice);ui.separator();}match self.mode{Mode::Home=>self.home(ui),Mode::Learn=>self.card(ui,false),Mode::Dictation=>self.card(ui,true),Mode::Browse=>self.browse(ui)}});}}
fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    for path in [r"C:\Windows\Fonts\msyh.ttc", r"C:\Windows\Fonts\simhei.ttf"] {
        if let Ok(data) = fs::read(path) {
            fonts.font_data.insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "cjk".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("cjk".to_owned());
            break;
        }
    }
    ctx.set_fonts(fonts);
}
fn main()->eframe::Result<()> {let options=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size([900.0,650.0]).with_min_inner_size([720.0,520.0]),..Default::default()};eframe::run_native("Deutsch Worttrainer",options,Box::new(|cc|{setup_fonts(&cc.egui_ctx);cc.egui_ctx.set_pixels_per_point(1.15);Ok(Box::new(App::new()))}))}
