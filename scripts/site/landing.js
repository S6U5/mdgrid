// 入口のページの動き。scripts/pages.py が入口の <script> に埋める。
// JS が無くても、画面・文・コマンドは全部読める(ここは切り替えと図を足すだけ)。
(function () {
  var still = window.matchMedia && matchMedia("(prefers-reduced-motion: reduce)").matches;
  function all(sel, root) { return Array.prototype.slice.call((root || document).querySelectorAll(sel)); }

  // コマンドを写す。
  all(".cmd button[data-copy]").forEach(function (b) {
    var label = b.querySelector("span");
    var said = label.textContent;
    b.addEventListener("click", function () {
      var text = b.parentNode.querySelector("code").getAttribute("data-text");
      function done() {
        b.classList.add("done");
        label.textContent = b.getAttribute("data-done");
        setTimeout(function () { b.classList.remove("done"); label.textContent = said; }, 1600);
      }
      function fallback() {
        var t = document.createElement("textarea");
        t.value = text; t.style.position = "fixed"; t.style.opacity = "0";
        document.body.appendChild(t); t.select();
        try { document.execCommand("copy"); done(); } catch (e) {}
        document.body.removeChild(t);
      }
      if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(text).then(done, fallback);
      else fallback();
    });
  });

  // 最初の画面: 開く → 直す → 保存 の3場面。押すまでは順に進み、押したらそこで止まる。
  (function () {
    var steps = all(".steps .step"), frames = all(".frames a");
    if (!steps.length) return;
    var at = 0, timer = null, auto = !still, DUR = 5200, seen = false;
    function show(i) {
      at = i;
      steps.forEach(function (s, k) { s.setAttribute("aria-pressed", k === i ? "true" : "false"); s.classList.remove("run"); });
      frames.forEach(function (f, k) { f.classList.toggle("on", k === i); f.tabIndex = k === i ? 0 : -1; });
    }
    function tick() {
      clearTimeout(timer);
      if (!auto || !seen || document.hidden) return;
      var s = steps[at];
      s.style.setProperty("--dur", DUR + "ms");
      void s.offsetWidth;  // 幅の動きを頭からやり直す
      s.classList.add("run");
      timer = setTimeout(function () { show((at + 1) % steps.length); tick(); }, DUR);
    }
    steps.forEach(function (s, i) {
      s.addEventListener("click", function () { auto = false; clearTimeout(timer); show(i); });
    });
    show(0);
    if ("IntersectionObserver" in window) {
      new IntersectionObserver(function (es) {
        seen = es[0].isIntersecting;
        if (seen) tick(); else { clearTimeout(timer); steps[at].classList.remove("run"); }
      }, { threshold: 0.35 }).observe(document.querySelector(".stage"));
    }
    document.addEventListener("visibilitychange", function () { if (!document.hidden) tick(); });
  })();

  // タブ(自分の場面)。← → でも動く。
  all("[role=tablist]").forEach(function (list) {
    var tabs = all("[role=tab]", list);
    function pick(i, focus) {
      tabs.forEach(function (t, k) {
        var on = k === i;
        t.setAttribute("aria-selected", on ? "true" : "false");
        t.tabIndex = on ? 0 : -1;
        document.getElementById(t.getAttribute("aria-controls")).hidden = !on;
      });
      if (focus) tabs[i].focus();
    }
    tabs.forEach(function (t, i) {
      t.addEventListener("click", function () { pick(i); });
      t.addEventListener("keydown", function (e) {
        var d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
        if (d) { e.preventDefault(); pick((i + d + tabs.length) % tabs.length, true); }
      });
    });
    pick(0);
  });

  // テーマ: 押した名前の本物の画面に差し替え、設定の1行も書き換える。
  (function () {
    var img = document.getElementById("look-img");
    if (!img) return;
    var link = img.parentNode, code = document.getElementById("look-code");
    all("[data-look]").forEach(function (b) {
      b.addEventListener("click", function (e) {
        e.preventDefault();
        var id = b.getAttribute("data-look");
        all("[data-look]").forEach(function (x) { x.setAttribute("aria-pressed", x === b ? "true" : "false"); });
        img.src = img.src.replace(/theme-[a-z-]+\.svg/, "theme-" + id + ".svg");
        img.alt = b.textContent;
        link.href = link.href.replace(/#theme-[a-z-]+/, "#theme-" + id);
        var line = 'theme = "' + id + '"';
        code.setAttribute("data-text", "[look]\n" + line);
        code.querySelector(".v").textContent = line;
      });
    });
  })();

  // 仕組みの図: ノートのファイルと表。行を選ぶとノートが替わり、status を選び直すと書かれる1行が出る。
  (function () {
    var box = document.getElementById("model");
    var data = document.getElementById("model-data");
    if (!box || !data) return;
    var notes = JSON.parse(data.textContent);
    var say = JSON.parse(box.getAttribute("data-say"));
    var pre = box.querySelector(".file pre"), name = box.querySelector(".file header b");
    var rows = all(".sheet tbody tr", box), tally = box.querySelector(".tally");
    var count = tally.querySelector("b"), text = tally.querySelector("span"), undo = tally.querySelector("button");
    var sel = 0, changed = {};

    function line(cls, key, rest) {
      var s = document.createElement("span");
      s.className = "l" + (cls ? " " + cls : "");
      if (key) {
        var k = document.createElement("span");
        k.className = "k"; k.textContent = key; k.setAttribute("data-key", key);
        s.appendChild(k);
      }
      s.appendChild(document.createTextNode(rest));
      return s;
    }
    function file() {
      var n = notes[sel], now = changed[sel];
      name.textContent = n.path;
      pre.textContent = "";
      var code = document.createElement("code"), inside = false, seenEnd = false;
      n.lines.forEach(function (raw) {
        if (raw === "---") { inside = !inside && !seenEnd; if (!inside) seenEnd = true; code.appendChild(line("fm", "", raw)); return; }
        if (!inside) { code.appendChild(line("body", "", raw)); return; }
        var m = /^([A-Za-z_]+)(:.*)$/.exec(raw);
        if (!m) { code.appendChild(line("", "", raw)); return; }
        if (m[1] === "status" && now) {
          code.appendChild(line("del", "- status", m[2]));
          code.appendChild(line("add", "+ status", ": " + now));
        } else code.appendChild(line("", m[1], m[2]));
      });
      pre.appendChild(code);
    }
    function sum() {
      var n = Object.keys(changed).length;
      count.textContent = say.unsaved + " " + n;
      text.textContent = n ? say.dirty.replace("{n}", n) : say.clean;
      tally.classList.toggle("dirty", n > 0);
      undo.hidden = n === 0;
    }
    function choose(i) {
      sel = i;
      rows.forEach(function (r, k) { r.classList.toggle("sel", k === i); });
      file();
    }
    rows.forEach(function (r, i) {
      var pick = r.querySelector(".pick"), s = pick.querySelector("select");
      r.addEventListener("click", function () { choose(i); });
      s.addEventListener("focus", function () { choose(i); });
      s.addEventListener("change", function () {
        if (s.value === notes[i].status) delete changed[i]; else changed[i] = s.value;
        pick.setAttribute("data-v", s.value);
        pick.classList.toggle("chg", i in changed);
        choose(i); sum();
      });
    });
    undo.addEventListener("click", function () {
      changed = {};
      rows.forEach(function (r, i) {
        var pick = r.querySelector(".pick");
        pick.querySelector("select").value = notes[i].status;
        pick.setAttribute("data-v", notes[i].status);
        pick.classList.remove("chg");
      });
      file(); sum();
    });
    // キーに乗ると、表の同じ列が光る(キーが列になる、を見せる)。
    function glow(key) {
      all("[data-col]", box).forEach(function (c) { c.classList.toggle("col-hot", key !== null && c.getAttribute("data-col") === key); });
      all(".file .l", box).forEach(function (l) {
        var k = l.querySelector(".k");
        l.classList.toggle("hot", key !== null && !!k && k.getAttribute("data-key") === key);
      });
    }
    pre.addEventListener("mouseover", function (e) {
      var l = e.target.closest ? e.target.closest(".l") : null, k = l && l.querySelector(".k");
      glow(k ? k.getAttribute("data-key") : null);
    });
    pre.addEventListener("mouseleave", function () { glow(null); });
    all("thead th[data-col]", box).forEach(function (th) {
      th.addEventListener("mouseenter", function () { glow(th.getAttribute("data-col")); });
      th.addEventListener("mouseleave", function () { glow(null); });
    });
    choose(0); sum();
  })();

  // 段が見えたら、そっと出す。
  (function () {
    var els = all(".rise");
    if (still || !("IntersectionObserver" in window)) { els.forEach(function (e) { e.classList.add("seen"); }); return; }
    var io = new IntersectionObserver(function (es) {
      es.forEach(function (e) { if (e.isIntersecting) { e.target.classList.add("seen"); io.unobserve(e.target); } });
    }, { rootMargin: "0px 0px -8% 0px" });
    els.forEach(function (e) { io.observe(e); });
    // 印刷や、見えないまま読まれるときに備えて、しばらくしたら全部出す。
    setTimeout(function () { els.forEach(function (e) { e.classList.add("seen"); }); }, 4000);
  })();
})();
