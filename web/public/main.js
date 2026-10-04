// main.js - PWA glue code.
//
// Loads the WASM module (built from the shared Rust core), reads vocabulary.csv,
// drives the home + review UI, persists progress in localStorage and speaks
// German via the browser Web Speech API. All vocabulary/scheduling logic lives
// in Rust/WASM; this file is only presentation + storage + audio.

import init, {
  parse_words_json,
  levels_json,
  make_queue_json,
  grade_card_json,
  check_answer,
  due_summary_json,
  stats_json,
  version,
} from "./pkg/wortmeister_wasm.js";


const PROGRESS_KEY = "wortmeister.progress.v1";
const SETTINGS_KEY = "wortmeister.settings.v1";

const state = {
  words: [],          // array of Word objects
  wordsJson: "[]",    // cached JSON string for WASM calls
  byId: new Map(),    // id -> Word
  queue: [],          // array of word IDs
  pos: 0,
  mode: "learn",      // "learn" | "dictation"
  revealed: false,
  // Dictation result for the current card, preserved across re-renders so the
  // "correct/wrong" feedback and the typed text stay visible after checking.
  dictation: { checked: false, correct: false, typed: "" },
};


const $ = (id) => document.getElementById(id);

// --- storage helpers -------------------------------------------------------

function loadProgress() {
  return localStorage.getItem(PROGRESS_KEY) || "null";
}
function saveProgress(json) {
  localStorage.setItem(PROGRESS_KEY, json);
}
function loadSettings() {
  try {
    return JSON.parse(localStorage.getItem(SETTINGS_KEY)) || {};
  } catch {
    return {};
  }
}
function saveSettings(s) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(s));
}

// --- speech (German) -------------------------------------------------------

let germanVoice = null;
function pickGermanVoice() {
  const voices = speechSynthesis.getVoices();
  germanVoice =
    voices.find((v) => v.lang && v.lang.toLowerCase().startsWith("de")) || null;
}
function speak(text) {
  if (!("speechSynthesis" in window)) return;
  const u = new SpeechSynthesisUtterance(text);
  u.lang = "de-DE";
  if (germanVoice) u.voice = germanVoice;
  speechSynthesis.cancel();
  speechSynthesis.speak(u);
}

// --- UI: home --------------------------------------------------------------

function refreshDueBadge() {
  try {
    const summary = JSON.parse(
      due_summary_json(state.wordsJson, loadProgress(), $("level").value, "all")
    );
    $("dueBadge").textContent = `今天到期 ${summary.due} / 共 ${summary.total} 词`;
  } catch (e) {
    $("dueBadge").textContent = "";
  }
}

function populateLevels() {
  const levels = JSON.parse(levels_json(state.wordsJson));
  const sel = $("level");
  sel.innerHTML = "";
  for (const lvl of levels) {
    const opt = document.createElement("option");
    opt.value = lvl;
    opt.textContent = lvl === "all" ? "全部" : lvl;
    sel.appendChild(opt);
  }
  const saved = loadSettings();
  if (saved.level && levels.includes(saved.level)) sel.value = saved.level;
}

// --- UI: review ------------------------------------------------------------

function showScreen(which) {
  $("home").classList.toggle("hidden", which !== "home");
  $("review").classList.toggle("hidden", which !== "review");
  $("stats").classList.toggle("hidden", which !== "stats");
}

// --- UI: statistics --------------------------------------------------------

// Compute stats via the Rust core and paint the stats screen.
function openStats() {
  let s;
  try {
    s = JSON.parse(stats_json(state.wordsJson, loadProgress()));
  } catch (e) {
    $("notice").textContent = "统计计算失败：" + e;
    return;
  }

  $("stReviewsToday").textContent = s.reviews_today;
  $("stStreak").textContent = s.streak;
  $("stDue").textContent = s.due_today;
  $("stLearned").textContent = s.learned;
  $("stStudied").textContent = s.studied;
  $("stNew").textContent = s.new;
  $("stAccuracy").textContent = s.accuracy + "%";
  $("stTotal").textContent = s.total_words;

  const mastery = s.total_words > 0
    ? Math.round((s.learned / s.total_words) * 100)
    : 0;
  $("stMasteryPct").textContent = mastery + "%";
  $("stMasteryFill").style.width = mastery + "%";

  $("stAnswers").textContent =
    `累计答对 ${s.total_correct} 次，答错 ${s.total_wrong} 次`;

  showScreen("stats");
}


function currentWord() {
  if (state.pos >= state.queue.length) return null;
  return state.byId.get(state.queue[state.pos]);
}

function startSession(mode) {
  state.mode = mode;
  const limit = parseInt($("limit").value, 10);
  const queueJson = make_queue_json(
    state.wordsJson,
    loadProgress(),
    $("level").value,
    "all",
    limit
  );
  state.queue = JSON.parse(queueJson);
  state.pos = 0;
  saveSettings({ level: $("level").value, limit });

  if (state.queue.length === 0) {
    $("notice").textContent = "今天没有到期词条。可扩大级别范围，或明天再来。";
    return;
  }
  $("notice").textContent = "";
  showScreen("review");
  gotoCard();
}

/// Move to the current card position, resetting per-card state (reveal flag and
/// dictation result). Call this when advancing to a new card.
function gotoCard() {
  state.revealed = false;
  state.dictation = { checked: false, correct: false, typed: "" };
  renderCard();
}

// Paint the current card from state. This is idempotent and must NOT clear the
// user's typed text or dictation result - those live in state.dictation and are
// only reset by gotoCard() when moving to the next card.
function renderCard() {
  const word = currentWord();
  if (!word) return renderDone();

  $("counter").textContent = `${state.pos + 1} / ${state.queue.length}`;

  const isDictation = state.mode === "dictation";
  // Before the answer is revealed in dictation, hide the German word.
  const hideWord = isDictation && !state.revealed;

  $("prompt").textContent = hideWord ? "听写：输入你听到的词" : word.german;

  // Answer block (meaning + examples) visible only after reveal.
  $("answer").classList.toggle("hidden", !state.revealed);
  if (state.revealed) {
    $("chinese").textContent = word.chinese;
    $("example").textContent = word.example;
    $("exampleZh").textContent = word.example_zh;
  }

  // Dictation input box: shown throughout dictation mode (input while typing,
  // then kept visible so the user can see what they typed vs. the answer).
  $("dictationBox").classList.toggle("hidden", !isDictation);
  if (isDictation) {
    const inp = $("typed");
    inp.value = state.dictation.typed;
    inp.readOnly = state.dictation.checked; // lock after checking
    const r = $("dictationResult");
    if (state.dictation.checked) {
      r.textContent = state.dictation.correct
        ? "正确 ✓"
        : `不对，正确答案：${word.german}`;
      r.className = "result " + (state.dictation.correct ? "good" : "bad");
    } else {
      r.textContent = "";
      r.className = "result";
    }
  }

  // Auto-play the German word (dictation: before checking; learn: show+play).
  if (isDictation && !state.dictation.checked) {
    speak(word.german);
    setTimeout(() => $("typed").focus(), 50);
  }

  renderControls();
}

function renderControls() {
  const controls = $("controls");
  controls.innerHTML = "";
  controls.classList.remove("single");

  const word = currentWord();
  if (!word) return;

  // Dictation, not yet checked: show the Check button.
  if (state.mode === "dictation" && !state.dictation.checked) {
    controls.classList.add("single");
    controls.appendChild(makeButton("检查", "primary", checkDictation));
    return;
  }

  // Learn mode, answer still hidden: show the Reveal button.
  if (state.mode !== "dictation" && !state.revealed) {
    controls.classList.add("single");
    controls.appendChild(makeButton("显示答案", "primary", () => {
      state.revealed = true;
      renderCard();
    }));
    return;
  }

  // Otherwise (answer revealed): show the four SM-2 grading buttons.
  controls.appendChild(makeButton("Again", "grade-again", () => grade("again")));
  controls.appendChild(makeButton("Hard", "grade-hard", () => grade("hard")));
  controls.appendChild(makeButton("Good", "grade-good", () => grade("good")));
  controls.appendChild(makeButton("Easy", "grade-easy", () => grade("easy")));
}

// Check the dictation answer against the current word using the Rust core's
// lenient comparison, store the result in state and reveal the answer. We save
// the result in state so a re-render keeps the feedback on screen.
function checkDictation() {
  const word = currentWord();
  if (!word) return;
  const typed = $("typed").value;
  const correct = check_answer(word.german, typed);
  state.dictation = { checked: true, correct, typed };
  state.revealed = true;
  renderCard();
}

function grade(g) {
  const word = currentWord();
  if (!word) return;
  const updated = grade_card_json(loadProgress(), word.id, g);
  saveProgress(updated);
  state.pos += 1;
  gotoCard();
}


function renderDone() {
  $("counter").textContent = "";
  $("prompt").innerHTML = '<div class="done"><div class="big">✓</div>本轮完成</div>';
  $("answer").classList.add("hidden");
  $("dictationBox").classList.add("hidden");
  $("controls").innerHTML = "";
  refreshDueBadge();
}

function makeButton(label, cls, onClick) {
  const b = document.createElement("button");
  b.textContent = label;
  if (cls) b.className = cls;
  b.addEventListener("click", onClick);
  return b;
}

// --- bootstrap -------------------------------------------------------------

async function main() {
  await init(); // load + instantiate the wasm module

  // Load vocabulary CSV (bundled next to index.html).
  try {
    const res = await fetch("vocabulary.csv");
    const csv = await res.text();
    state.wordsJson = parse_words_json(csv);
    state.words = JSON.parse(state.wordsJson);
    state.byId = new Map(state.words.map((w) => [w.id, w]));
  } catch (e) {
    $("notice").textContent = "加载 vocabulary.csv 失败：" + e;
    return;
  }

  populateLevels();

  // Restore saved daily limit.
  const settings = loadSettings();
  if (settings.limit) {
    $("limit").value = settings.limit;
    $("limitValue").textContent = settings.limit;
  }
  refreshDueBadge();

  // Home events.
  $("limit").addEventListener("input", (e) => {
    $("limitValue").textContent = e.target.value;
  });
  $("level").addEventListener("change", refreshDueBadge);
  $("startLearn").addEventListener("click", () => startSession("learn"));
  $("startDictation").addEventListener("click", () => startSession("dictation"));
  $("openStats").addEventListener("click", openStats);

  // Stats screen events.
  $("statsBack").addEventListener("click", () => {
    showScreen("home");
    refreshDueBadge();
  });
  $("resetProgress").addEventListener("click", () => {
    if (confirm("确定清除所有学习进度吗？此操作无法撤销。")) {
      localStorage.removeItem(PROGRESS_KEY);
      openStats();      // repaint with empty stats
      refreshDueBadge();
    }
  });


  // Review events.
  $("back").addEventListener("click", () => {
    showScreen("home");
    refreshDueBadge();
  });
  $("speak").addEventListener("click", () => {
    const w = currentWord();
    if (w) speak(w.german);
  });
  $("typed").addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      const btn = $("controls").querySelector("button");
      if (btn) btn.click();
    }
  });

  // Speech voices load asynchronously in some browsers.
  pickGermanVoice();
  if ("speechSynthesis" in window) {
    speechSynthesis.onvoiceschanged = pickGermanVoice;
  }

  console.log("WortMeister core version", version());

  // Register the service worker for offline use (PWA).
  if ("serviceWorker" in navigator) {
    try {
      await navigator.serviceWorker.register("sw.js");
    } catch (e) {
      console.warn("SW registration failed", e);
    }
  }
}

main();
