import {
  b as f,
  i as e,
  t as a,
  l as _,
  a as x,
  s as k
} from "./i18n.js";
import {
  i as A,
  d as w,
  e as p,
  t as c,
  b as g,
  c as $
} from "./ui.js";
const b = document.getElementById("app");
let o = null,
  y = !1,
  v = !1,
  u = "";
async function i() {
  o = await e("get_state"), y = await e("autostart_status").catch(() => !1), o?.language && x(o.language), document.documentElement.dataset.theme = o?.theme || "dark", localStorage.setItem("zaccounts.theme", document.documentElement.dataset.theme)
}
async function l(t) {
  if (!v) {
    v = !0;
    try {
      await t()
    } catch (s) {
      c(k(s), "err")
    } finally {
      v = !1
    }
  }
}
const r = {
    async refresh() {
      await i(), n()
    },
    async openGitHub() {
      await e("open_external", {
        url: "https://github.com/Kang-code-sudo/Z-Accounts"
      })
    },
    async setLang(t) {
      t !== _() && await l(async () => {
        await e("set_language", {
          lang: t
        }), await i(), n()
      })
    },
    async setTheme(t) {
      t !== o?.theme && await l(async () => {
        await e("set_theme", {
          theme: t
        }), await i(), n()
      })
    },
    async toggleAutostart() {
      await l(async () => {
        const t = await e("autostart_set", {
          enable: !y
        });
        y = t, c(t ? a("s.autostartOnToast") : a("s.autostartOffToast")), n()
      })
    },
    async toggleBehavior(t) {
      await l(async () => {
        await e("set_behavior", {
          launchAfterSwitch: t === "launch" ? !o.launch_after_switch : null,
          closeToTray: t === "tray" ? !o.close_to_tray : null,
          hotSwitch: t === "hot" ? !o.hot_switch : null
        }), await i(), n(), c(a("s.savedToast"))
      })
    },
    async exportAll() {
      await l(async () => {
        const t = await e("export_all_pick_path");
        if (!t.picked) {
          c(a("m.exportCanceled"));
          return
        }
        $({
          mode: "exportAll",
          path: t.path,
          count: t.count,
          onDone: () => r.refresh()
        })
      })
    },
    async importFiles() {
      await l(async () => {
        const t = await e("import_pick_files");
        if (!t.picked) return;
        const s = t.sealed || [],
          d = t.errors || [];
        if (s.length) {
          $({
            mode: "import",
            files: s,
            preErrors: d,
            onDone: m => r.finishImport(m)
          });
          return
        }
        r.finishImport({
          added: [],
          skipped: [],
          errors: d
        })
      })
    },
    finishImport(t) {
      if (t.added.length === 0 && t.skipped.length === 0) c(a("s.importNone"), "err", t.errors.join(a("common.listSep")) || void 0);
      else {
        const s = [];
        t.added.length && s.push(a("s.importAdded", {
          count: t.added.length,
          names: t.added.join(a("common.listSep"))
        })), t.skipped.length && s.push(a("s.importSkipped", {
          count: t.skipped.length
        })), t.errors.length && s.push(a("s.importFailed", {
          count: t.errors.length
        })), c(s[0], t.errors.length ? "err" : "ok", s.slice(1).join(a("common.listSep")))
      }
      i().then(n)
    },
    async browsePath() {
      await l(async () => {
        const t = await e("pick_zcode_path");
        t.picked && (await e("set_zcode_path", {
          path: t.path
        }), c(a("s.pathUpdated")), await i(), n())
      })
    },
    async savePath() {
      const t = document.querySelector(".settings input.zcode-path:not(.auth-proxy)");
      if (!t) return;
      const s = t.value.trim();
      await l(async () => {
        await e("set_zcode_path", {
          path: s
        }), c(s ? a("s.pathUpdated") : a("s.pathAuto")), await i(), n()
      })
    },
    async toggleAuthProxy() {
      const s = (document.querySelector(".settings input.auth-proxy")?.value || "").trim() || o.auth_proxy_url || null;
      await l(async () => {
        await e("set_auth_proxy", {
          on: !o.auth_proxy_on,
          url: s
        }), await i(), n(), c(o.auth_proxy_on ? a("s.proxyOnToast") : a("s.proxyOffToast"), "ok", a("s.proxyOnDetail"))
      })
    },
    async saveProxy() {
      const t = document.querySelector(".settings input.auth-proxy");
      t && await l(async () => {
        await e("set_auth_proxy", {
          on: o.auth_proxy_on,
          url: t.value.trim()
        }), await i(), n(), c(a("s.proxySaved"), "ok", o.auth_proxy_on ? a("s.proxySavedOn") : a("s.proxySavedOff"))
      })
    }
  },
  h = (t, s, d, m) => `
  <div class="tog-row">
    <div class="tog-info"><div class="tog-label">${d}</div><div class="tog-desc">${m}</div></div>
    <button class="toggle${t?" on":""}" role="switch" aria-checked="${t}" aria-label="${d}" click="${s}">
      <span class="knob"></span>
    </button>
  </div>`,
  S = t => `
  <div class="tog-row">
    <div class="tog-info"><div class="tog-label">${a("s.langLabel")}</div></div>
    <div class="lang-seg" role="radiogroup" aria-label="${a("s.langLabel")}">
      <button class="lang-opt${t==="zh"?" on":""}" role="radio" aria-checked="${t==="zh"}" click="actions.setLang('zh')">${a("s.langZh")}</button>
      <button class="lang-opt${t==="en"?" on":""}" role="radio" aria-checked="${t==="en"}" click="actions.setLang('en')">${a("s.langEn")}</button>
    </div>
  </div>`;

function n() {
  if (!o) {
    b.innerHTML = '<div class="loading">LOADING</div>';
    return
  }
  const t = o;
  document.title = `Z-Accounts · ${a("s.title")}`, b.innerHTML = `
    <header class="topbar">
      <div class="brand"><img class="brand-mark" src="/brand-icon.png" alt=""><div class="brand-copy"><span class="wordmark">Z-Accounts</span><span class="brand-caption">${a("s.title")}${u?` · v${p(u)}`:""}</span></div></div>
    </header>
    <section class="settings open">
      ${S(t.language||"zh")}
      <div class="tog-row"><div class="tog-info"><div class="tog-label">${a("m.appearance")}</div></div>
        <div class="lang-seg compact" role="radiogroup" aria-label="${a("m.appearance")}">
          <button class="lang-opt${t.theme!=="light"?" on":""}" role="radio" aria-checked="${t.theme!=="light"}" click="actions.setTheme('dark')">${g("moon",15)} ${a("m.darkMode")}</button>
          <button class="lang-opt${t.theme==="light"?" on":""}" role="radio" aria-checked="${t.theme==="light"}" click="actions.setTheme('light')">${g("sun",15)} ${a("m.lightMode")}</button>
        </div>
      </div>
      <label>BEHAVIOR · ${a("s.behaviorLabel")}</label>
      ${h(y,"actions.toggleAutostart()",a("s.autostart"),a("s.autostartDesc"))}
      ${h(t.launch_after_switch,"actions.toggleBehavior('launch')",a("s.launchAfter"),a("s.launchAfterDesc"))}
      ${h(t.close_to_tray,"actions.toggleBehavior('tray')",a("s.closeTray"),a("s.closeTrayDesc"))}
      ${h(t.hot_switch,"actions.toggleBehavior('hot')",a("s.hotSwitch"),a("s.hotSwitchDesc"))}
      <label style="margin-top:14px">${a("s.authLabel")}</label>
      ${h(t.auth_proxy_on,"actions.toggleAuthProxy()",a("s.proxyToggle"),a("s.proxyToggleDesc"))}
      <div class="path-line" style="margin-top:6px">
        <input class="zcode-path auth-proxy" type="text" value="${p(t.auth_proxy_url||"")}"
          placeholder="${a("s.proxyPh")}" keydown="onProxyKey(event)">
        <button class="btn-ghost" click="actions.saveProxy()">${a("common.save")}</button>
      </div>
      <label style="margin-top:14px">${a("s.libLabel")}</label>
      <div class="lib-row">
        <button class="btn-ghost has-ic" click="actions.importFiles()">${g("import",14)} ${a("s.importBtn")}</button>
        <button class="btn-ghost has-ic" click="actions.exportAll()" ${t.accounts.length?"":"disabled"}>${g("exportAll",14)} ${a("s.exportAllBtn")}</button>
      </div>
      <label style="margin-top:14px">${a("s.pathLabel")}</label>
      <div class="path-line">
        <input class="zcode-path" type="text" value="${p(t.zcode_path)}" placeholder="C:\\Program Files\\ZCode\\ZCode.exe" keydown="onPathKey(event)">
        <button class="btn-ghost" click="actions.browsePath()">${a("s.browse")}</button>
        <button class="btn-ghost" click="actions.savePath()">${a("common.save")}</button>
      </div>
      <div class="hint">${a("s.hint")}</div>
      <div class="gh-row"><a class="gh-link" href="https://github.com/Kang-code-sudo/Z-Accounts" target="_blank" rel="noopener" click="actions.openGitHub()">${a("s.githubLink")}</a>${u?`<span class="ver">v${p(u)}</span>`:""}</div>
    </section>`
}
window.actions = r;
window.onPathKey = t => {
  t.key === "Enter" && r.savePath()
};
window.onProxyKey = t => {
  t.key === "Enter" && r.saveProxy()
};
A();
f("state-changed", () => {
  i().then(n).catch(() => {})
});
(async () => {
  try {
    u = await e("app_version").catch(() => ""), await i(), n(), w()
  } catch (t) {
    b.innerHTML = `<div class="loading" style="color:var(--red)">${a("common.loadFail",{e:p(String(t))})}</div>`, w()
  }
})();