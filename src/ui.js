import {
  t as c,
  h as E,
  i as y,
  d as x,
  s as $
} from "./i18n.js";
const t = e => `<path d="${e}"/>`,
  L = {
    grid: '<rect x="2.5" y="2.5" width="4" height="4" rx="1"/><rect x="9.5" y="2.5" width="4" height="4" rx="1"/><rect x="2.5" y="9.5" width="4" height="4" rx="1"/><rect x="9.5" y="9.5" width="4" height="4" rx="1"/>',
    list: t("M6 4h8M6 8h8M6 12h8M2 4h.1M2 8h.1M2 12h.1"),
    search: '<circle cx="7" cy="7" r="4.5"/>' + t("M10.4 10.4 14 14"),
    eye: t("M1.3 8s2.4-4.2 6.7-4.2 6.7 4.2 6.7 4.2-2.4 4.2-6.7 4.2S1.3 8 1.3 8z") + '<circle cx="8" cy="8" r="2"/>',
    eyeOff: t("M1.5 1.5 14.5 14.5") + t("M6.1 3.9A7.3 7.3 0 0 1 8 3.7c4.3 0 6.7 4.3 6.7 4.3a10 10 0 0 1-2 2.5") + t("M9.9 12.1A7.3 7.3 0 0 1 8 12.3C3.7 12.3 1.3 8 1.3 8a10 10 0 0 1 2-2.5"),
    chevron: t("m6 4 4 4-4 4"),
    plus: t("M8 3v10M3 8h10"),
    trash: t("M2.5 4.5h11M6 2.5h4M4 4.5l.6 9h6.8l.6-9M6.5 7v4M9.5 7v4"),
    gauge: t("M2.5 12.5h11") + t("M3.6 12.5a4.4 4.4 0 0 1 8.8 0") + t("M8 12.5 10.8 8.2"),
    token: '<circle cx="8" cy="8" r="5.5"/><path d="M5.2 6.2h5.6M5.2 9.8h5.6M8 3.7v8.6"/>',
    activity: t("M2 8h2.3l1.3-3.2 2.4 6.4 1.5-3.2H14"),
    pen: t("M2.5 13.5l.9-3.3 7.6-7.6 2.4 2.4-7.6 7.6z") + t("M9.9 4.9l2.4 2.4"),
    export: t("M8 9V2") + t("M5.2 4.8 8 2l2.8 2.8") + t("M2.5 10.5v1.5a1.5 1.5 0 0 0 1.5 1.5h8a1.5 1.5 0 0 0 1.5-1.5v-1.5"),
    import: t("M8 2v7") + t("M5.2 6.2 8 9l2.8-2.8") + t("M2.5 10.5v1.5a1.5 1.5 0 0 0 1.5 1.5h8a1.5 1.5 0 0 0 1.5-1.5v-1.5"),
    exportAll: t("M2.5 13.5h11") + t("M4.5 11h7") + t("M8 2v6") + t("M5.5 5.5 8 8l2.5-2.5"),
    x: t("M3.5 3.5l9 9") + t("M12.5 3.5l-9 9"),
    swap: t("M2.5 5.5h9.5") + t("M9.5 3 12 5.5 9.5 8") + t("M13.5 10.5H4") + t("M6.5 8 4 10.5 6.5 13"),
    capture: t("M3 2.5h10") + t("M8 5.2v5.8") + t("M5.5 8.5 8 11l2.5-2.5") + t("M3 13.5h10"),
    gift: t("M2.5 6.5h11") + '<rect x="3.5" y="8" width="9" height="5.5"/>' + t("M8 6.5v7") + t("M8 6.5 6 4.6") + t("M8 6.5 10 4.6"),
    refresh: t("M13.5 8a5.5 5.5 0 1 1-1.6-3.9") + t("M13.5 2.5V6.5h-4"),
    sun: '<circle cx="8" cy="8" r="2.6"/>' + t("M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3 3l1.1 1.1M11.9 11.9 13 13M13 3l-1.1 1.1M4.1 11.9 3 13"),
    moon: t("M12.9 10.8A5.6 5.6 0 0 1 5.2 3.1 5.6 5.6 0 1 0 12.9 10.8z"),
    userPlus: t("M6 6.8a2.3 2.3 0 1 0 0-4.6 2.3 2.3 0 0 0 0 4.6") + t("M2 13.5v-.9a4 4 0 0 1 8 0v.9") + t("M12 5.5v4") + t("M10 7.5h4"),
    play: '<path d="M5 3.2v9.6l8.2-4.8z" fill="currentColor" stroke="none"/>',
    power: t("M8 2v5.5") + t("M4.6 4.3a5 5 0 1 0 6.8 0"),
    sliders: t("M2 4.5h12") + t("M2 11.5h12") + '<rect x="8.5" y="2.5" width="4" height="4"/><rect x="3.5" y="9.5" width="4" height="4"/>',
    lock: '<rect x="3.5" y="7.2" width="9" height="6.3"/>' + t("M5.5 7.2V5a2.5 2.5 0 0 1 5 0v2.2") + t("M8 9.7v1.8"),
    lockOpen: '<rect x="3.5" y="7.2" width="9" height="6.3"/>' + t("M5.5 7.2V5a2.5 2.5 0 0 1 4.9-.6") + t("M8 9.7v1.8"),
    check: t("M3 8.6l3.3 3.2L13 4.6"),
    alert: t("M8 2.2 14.8 13.8H1.2z") + t("M8 6.4v3.2") + t("M8 11.6v.2"),
    empty: t("M3 3h10") + t("M3 13h10") + '<path d="M8 6.2v3.6M6.2 8h3.6" stroke-dasharray="2 1.6"/>'
  };

function h(e, n = 16, s = "") {
  const a = L[e];
  return a ? `<svg class="ic${s?" "+s:""}" width="${n}" height="${n}" viewBox="0 0 16 16"
    fill="none" stroke="currentColor" stroke-width="1.5"
    stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${a}</svg>` : ""
}
document.addEventListener("contextmenu", e => e.preventDefault());
document.addEventListener("keydown", e => {
  e.key === "F12" && e.preventDefault(), e.ctrlKey && e.shiftKey && ["I", "i", "J", "j", "C", "c"].includes(e.key) && e.preventDefault(), e.ctrlKey && ["u", "s"].includes(e.key.toLowerCase()) && e.preventDefault()
});

function p(e) {
  return String(e).replace(/[&<>"']/g, n => ({
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;"
  })[n])
}

function A() {
  const e = document.getElementById("splash");
  e && (e.style.opacity = "0", setTimeout(() => e.remove(), 220))
}

function g(e, n = "ok", s = "") {
  let a = document.querySelector(".toast-zone");
  a || (a = document.createElement("div"), a.className = "toast-zone", document.body.appendChild(a));
  const o = document.createElement("div");
  o.className = `toast ${n}`, o.innerHTML = `<span class="t-ic">${h(n==="ok"?"check":"alert",16)}</span>
    <span class="t-body">${p(e)}${s?`<span class="detail">${p(s)}</span>`:""}</span>`, a.appendChild(o), setTimeout(() => o.remove(), s ? 5200 : 3200)
}

function v(e, n) {
  try {
    const s = e.indexOf("(");
    if (s < 0 || !e.trimEnd().endsWith(")")) return;
    let a = window;
    for (const r of e.slice(0, s).trim().split(".")) a = a?.[r];
    if (typeof a != "function") return;
    const o = e.slice(s + 1, e.trimEnd().length - 1).trim(),
      u = o ? o.split(/\s*,\s*/).map(r => {
        if (r === "event") return n;
        const l = r.match(/^'([^']*)'$/);
        return l ? l[1] : JSON.parse(r)
      }) : [];
    a(...u)
  } catch (s) {
    console.warn("attr handler error:", e, s)
  }
}

function C() {
  document.addEventListener("input", e => {
    const n = e.target.closest("[input]");
    n && v(n.getAttribute("input") || "", e)
  }), document.addEventListener("click", e => {
    const n = e.target.closest("[click]");
    n && (e.preventDefault(), v(n.getAttribute("click") || "", e))
  }), document.addEventListener("keydown", e => {
    const n = e.target.closest("[keydown]");
    n && v(n.getAttribute("keydown") || "", e)
  }), document.addEventListener("blur", e => {
    const n = e.target.closest("[blur]");
    n && v(n.getAttribute("blur") || "", e)
  }, !0)
}

function T(e) {
  document.querySelector(".cm-mask")?.remove();
  const n = e.kind || "plain",
    s = document.createElement("div");
  s.className = "cm-mask", s.innerHTML = `
    <div class="cm-panel" role="alertdialog" aria-modal="true" aria-label="${p(e.title)}">
      <div class="cm-strip ${n}"></div>
      <div class="cm-head">
        <span class="cm-ic ${n}">${h(e.icon||"alert",21)}</span>
        <div class="cm-title">${p(e.title)}</div>
      </div>
      ${e.desc?`<div class="cm-desc">${e.desc}</div>`:""}
      <div class="cm-actions">
        <button class="btn-ghost cm-no">${p(e.noLabel||c("common.cancel"))}</button>
        <button class="cm-yes ${n}">${p(e.yesLabel||c("common.confirm"))}</button>
      </div>
    </div>`, document.body.appendChild(s);
  const a = s.querySelector(".cm-panel"),
    o = s.querySelector(".cm-yes"),
    u = s.querySelector(".cm-no"),
    r = () => {
      document.removeEventListener("keydown", s._key), s.remove()
    };
  a.addEventListener("click", l => l.stopPropagation()), s.addEventListener("click", r), u.addEventListener("click", r), o.addEventListener("click", async () => {
    o.disabled = !0, u.disabled = !0;
    try {
      await e.onYes?.()
    } finally {
      r()
    }
  }), s._key = l => {
    l.key === "Escape" && r(), l.key === "Enter" && l.target === document.body && !o.disabled && o.click()
  }, document.addEventListener("keydown", s._key), (e.focusNo || n === "danger" ? u : o).focus()
}

function D(e) {
  document.querySelector(".pw-mask")?.remove();
  const n = e.mode === "export" || e.mode === "exportAll",
    s = e.mode === "export" ? c("pw.exportTitle", {
      name: p(e.name)
    }) : e.mode === "exportAll" ? c("pw.exportAllTitle", {
      count: e.count
    }) : c("pw.importTitle", {
      count: e.files.length
    }),
    a = n ? c("pw.exportSub", {
      path: p(e.path)
    }) : c("pw.importSub", {
      files: e.files.map(([i]) => p(i)).join(c("common.listSep"))
    }),
    o = document.createElement("div");
  o.className = "pw-mask", o.innerHTML = `
    <div class="pw-panel">
      <div class="pw-title">${s}</div>
      <div class="pw-sub">${a}</div>
      <input class="pw-input" type="password" id="pw1" placeholder="${n?c("pw.setPw"):c("pw.pw")}" autocomplete="off">
      ${n?`<input class="pw-input" type="password" id="pw2" placeholder="${c("pw.pwAgain")}" autocomplete="off">`:""}
      <div class="pw-err"></div>
      <div class="pw-actions">
        <button class="btn-ghost pw-cancel">${c("common.cancel")}</button>
        <button class="btn-primary pw-go has-ic">${n?h("lock",14)+" "+c("pw.encryptExport"):h("lockOpen",14)+" "+c("pw.decryptImport")}</button>
      </div>
    </div>`, document.body.appendChild(o);
  const u = o.querySelector(".pw-panel"),
    r = o.querySelector(".pw-err"),
    l = o.querySelector(".pw-go"),
    m = o.querySelector("#pw1"),
    w = o.querySelector("#pw2");
  u.addEventListener("click", i => i.stopPropagation()), o.addEventListener("click", () => o.remove()), o.querySelectorAll(".pw-cancel").forEach(i => i.addEventListener("click", () => o.remove()));
  const b = i => {
    i.key === "Escape" && o.remove(), i.key === "Enter" && f()
  };
  [m, w].forEach(i => i?.addEventListener("keydown", b)), m.focus();
  async function f() {
    const i = m?.value || "",
      M = w?.value;
    if (i.length < 6) return r.textContent = c("pw.errMinLen");
    if (M != null && i !== M) return r.textContent = c("pw.errMismatch");
    l.disabled = !0;
    try {
      if (e.mode === "export") {
        const d = await y("export_finalize", {
          path: e.path,
          id: e.id,
          password: i
        });
        g(c("pw.exportedToast", {
          path: d.path
        }), "ok", c("pw.exportedToastDetail")), e.onDone?.(d)
      } else if (e.mode === "exportAll") {
        const d = await y("export_all_finalize", {
          path: e.path,
          password: i
        });
        g(c("pw.exportedAllToast", {
          count: d.count
        }), "ok", d.path), e.onDone?.(d)
      } else {
        let d = await y("import_sealed", {
          files: e.files,
          password: i
        });
        if (e.preErrors?.length && (d.errors = [...e.preErrors, ...d.errors || []]), (d.errors || []).some(k => x(k) === "wrong_password" || String(k).includes("密码错误"))) {
          r.textContent = c("pw.errWrong"), l.disabled = !1;
          return
        }
        e.onDone?.(d)
      }
      o.remove()
    } catch (d) {
      r.textContent = $(d), l.disabled = !1
    }
  }
  l.addEventListener("click", f)
}

function P(e) {
  document.querySelector(".pv-mask")?.remove();
  const n = document.createElement("div");
  n.className = "pv-mask";
  const s = r => E(`prov.${r.id}`) ? c(`prov.${r.id}`) : r.display;
  n.innerHTML = `
    <div class="pv-panel">
      <div class="pv-title">${c("prov.title")}</div>
      <div class="pv-sub">${c("prov.sub")}</div>
      <div class="pv-list">
        ${(e.providers||[]).map(r=>`
          <button class="pv-item" data-id="${p(r.id)}">
            <span class="pv-name">${p(s(r))}</span>
            <span class="pv-arrow">→</span>
          </button>`).join("")}
        <button class="pv-item pv-manual">
          <span class="pv-name">${c("prov.manual")}</span>
          <span class="pv-arrow">→</span>
        </button>
      </div>
      <div class="pv-actions"><button class="btn-ghost pv-cancel">${c("common.cancel")}</button></div>
    </div>`, document.body.appendChild(n);
  const a = r => {
      r.key === "Escape" && o()
    },
    o = () => {
      n.remove(), document.removeEventListener("keydown", a)
    };
  n.querySelectorAll(".pv-item[data-id]").forEach(r => {
    r.addEventListener("click", () => {
      o(), e.onPick?.(r.dataset.id)
    })
  }), n.querySelector(".pv-manual").addEventListener("click", () => {
    o(), e.onManual?.()
  }), n.querySelector(".pv-cancel").addEventListener("click", o), document.addEventListener("keydown", a);
  const u = n.querySelector(".pv-item");
  u && u.focus()
}
export {
  P as a, h as b, D as c, A as d, p as e, C as i, T as o, g as t
};