import {
  b as et,
  i as g,
  a as Ht,
  t as a,
  s as M,
  l as Lt,
  c as H,
  h as Xt
} from "./i18n.js";
import {
  i as te,
  d as Et,
  e as c,
  o as rt,
  a as ee,
  b as m,
  t as ae,
  c as gt
} from "./ui.js";
const U = document.getElementById("app");
let p = null,
  Pt = "",
  Z = null,
  bt = !1,
  yt = "",
  A = localStorage.getItem("zaccounts.page") === "accounts" ? "accounts" : "dashboard",
  P = "all",
  pt = "",
  G = localStorage.getItem("zaccounts.view") === "list" ? "list" : "grid",
  y = localStorage.getItem("zaccounts.hideEmails") === "true",
  D = {},
  B = null,
  b = {},
  O = !1;
const wt = 6e4;
let w = {
    running: !1,
    done: 0,
    total: 0,
    cooldownUntil: 0
  },
  it = null;
const ot = 10 * 60 * 1e3,
  ne = 2 * 60 * 1e3,
  se = 45e3,
  ie = 5,
  oe = 5e3,
  le = 9e4;
let C = !1,
  X = {},
  mt = !1,
  E = !1,
  z = null,
  vt = !1,
  $t = !1,
  kt = !1,
  _t = !1,
  St = !1,
  qt = null,
  dt = null;
const It = ["var(--notch-1)", "var(--notch-2)", "var(--notch-3)", "var(--notch-4)", "var(--notch-5)", "var(--notch-6)"];

function Bt(t) {
  let e = 0;
  for (const n of t) e = e * 31 + n.charCodeAt(0) >>> 0;
  return It[e % It.length]
}

function zt(t) {
  if (t == null) return a("q.unknown");
  const e = Number(t);
  return isFinite(e) ? Lt() === "zh" ? Math.abs(e) >= 1e8 ? (e / 1e8).toFixed(2) + " 亿" : Math.abs(e) >= 1e4 ? (e / 1e4).toFixed(2) + " 万" : e.toLocaleString(H(), {
    maximumFractionDigits: 2
  }) : Math.abs(e) >= 1e9 ? (e / 1e9).toFixed(2) + "B" : Math.abs(e) >= 1e6 ? (e / 1e6).toFixed(2) + "M" : e.toLocaleString(H(), {
    maximumFractionDigits: 2
  }) : a("q.unknown")
}
const ce = /[\p{L}\p{N}._%+-]+@[\p{L}\p{N}.-]+\.[\p{L}]{2,}/giu;

function R(t) {
  const e = String(t ?? "");
  return y ? e.replace(ce, a("m.emailHidden")) : e
}

function Tt(t) {
  return /[\p{L}\p{N}._%+-]+@[\p{L}\p{N}.-]+\.[\p{L}]{2,}/iu.test(String(t ?? ""))
}

function d(t, e = "ok", n = "") {
  ae(R(t), e, R(n))
}
async function T() {
  p = await g("get_state"), p?.language && Ht(p.language), document.documentElement.dataset.theme = p?.theme || "dark", localStorage.setItem("zaccounts.theme", document.documentElement.dataset.theme)
}

function Ut() {
  return Z !== null || document.activeElement?.id === "account-search" || kt || _t
}

function st() {
  kt = !0, clearTimeout(qt), qt = setTimeout(() => {
    kt = !1, St && !Ut() && (St = !1, $())
  }, 180)
}

function _() {
  if (Ut()) {
    St = !0;
    return
  }
  $()
}
async function x(t) {
  if (!bt) {
    bt = !0;
    try {
      await t()
    } catch (e) {
      d(M(e), "err")
    } finally {
      bt = !1
    }
  }
}
async function xt(t) {
  const e = D[t] || {};
  if (!e.busy) {
    D[t] = {
      ...e,
      busy: !0,
      err: null
    }, _();
    try {
      const n = await g("get_account_quota", {
        id: t
      });
      D[t] = {
        data: n,
        err: null,
        busy: !1
      }, ge(t)
    } catch (n) {
      D[t] = {
        data: e.data || null,
        err: M(n),
        busy: !1,
        stale: !!e.data
      }
    }
    _()
  }
}
async function Rt() {
  try {
    const t = await g("get_recent_model_status");
    JSON.stringify(t) !== JSON.stringify(B) && (B = t, _())
  } catch {}
}
async function re(t) {
  const e = b[t] || {};
  if (!e.busy) {
    b[t] = {
      plans: e.plans || [],
      busy: !0
    };
    try {
      const n = await g("claim_preview", {
        id: t
      });
      b[t] = {
        plans: n || [],
        err: null,
        busy: !1
      }
    } catch (n) {
      b[t] = {
        plans: e.plans || [],
        err: String(n),
        busy: !1
      }
    }
  }
}
let K = null;

function At(t, e = 9e4) {
  return new Promise(n => {
    let s = !1;
    const i = o => {
        s || (s = !0, K = null, clearTimeout(l), n(o))
      },
      l = setTimeout(() => i(null), e);
    K = {
      accountId: t,
      finish: i
    }
  })
}
async function de(t) {
  b[t] = {
    ...b[t] || {},
    busy: !0
  };
  try {
    const e = await g("claim_preview", {
      id: t
    });
    b[t] = {
      plans: e || [],
      err: null,
      busy: !1
    }
  } catch (e) {
    b[t] = {
      plans: b[t]?.plans || [],
      err: String(e),
      busy: !1
    }
  }
}
const ue = t => R((p?.accounts || []).find(e => e.id === t)?.name || t);

function me(t) {
  const e = [a("btn.autoClaimTitle")];
  if (C && e.push(a(t.auto_claim ? "m.autoClaimRound" : "m.autoClaimStopping")), z) {
    const n = new Date(z.at),
      s = n.toDateString() === new Date().toDateString() ? n.toLocaleTimeString(H(), {
        hour12: !1
      }) : `${String(n.getMonth()+1).padStart(2,"0")}-${String(n.getDate()).padStart(2,"0")} ${n.toLocaleTimeString(H(),{hour12:!1})}`;
    e.push(a("m.autoClaimLast", {
      time: s,
      claimed: z.claimed,
      skipped: z.skipped
    })), z.cooldownAll && e.push(a("m.autoClaimCooldownAll"))
  }
  return c(e.join(" · "))
}
const S = {
  setPage(t) {
    A = t === "accounts" ? "accounts" : "dashboard", localStorage.setItem("zaccounts.page", A), $()
  },
  setFilter(t) {
    P = t === "active" ? "active" : "all", A = "accounts", localStorage.setItem("zaccounts.page", A), $(), U.querySelector(".list")?.scrollTo({
      top: 0
    })
  },
  setView(t) {
    G = t === "list" ? "list" : "grid", localStorage.setItem("zaccounts.view", G), $()
  },
  togglePrivacy() {
    y = !y, localStorage.setItem("zaccounts.hideEmails", String(y)), y && (Z = null, pt = "", document.querySelector(".toast-zone")?.replaceChildren()), $()
  },
  search(t) {
    pt = t.target.value, !t.isComposing && $()
  },
  async refresh() {
    await T(), $();
    for (const t of p.accounts) ct(t.id);
    W()
  },
  async setLang(t) {
    t !== Lt() && await x(async () => {
      await g("set_language", {
        lang: t
      }), await T(), $()
    })
  },
  async setTheme(t) {
    t !== p?.theme && await x(async () => {
      await g("set_theme", {
        theme: t
      }), await T(), $()
    })
  },
  async exportAll() {
    await x(async () => {
      const t = await g("export_all_pick_path");
      if (!t.picked) {
        d(a("m.exportCanceled"));
        return
      }
      gt({
        mode: "exportAll",
        path: t.path,
        count: t.count,
        onDone: () => S.refresh()
      })
    })
  },
  async importFiles() {
    await x(async () => {
      const t = await g("import_pick_files");
      if (!t.picked) return;
      const e = t.sealed || [],
        n = t.errors || [];
      e.length ? gt({
        mode: "import",
        files: e,
        preErrors: n,
        onDone: s => S.finishImport(s)
      }) : S.finishImport({
        added: [],
        skipped: [],
        errors: n
      })
    })
  },
  finishImport(t) {
    if (!t.added.length && !t.skipped.length) d(a("s.importNone"), "err", t.errors.join(a("common.listSep")) || void 0);
    else {
      const e = [];
      t.added.length && e.push(a("s.importAdded", {
        count: t.added.length,
        names: t.added.join(a("common.listSep"))
      })), t.skipped.length && e.push(a("s.importSkipped", {
        count: t.skipped.length
      })), t.errors.length && e.push(a("s.importFailed", {
        count: t.errors.length
      })), d(e[0], t.errors.length ? "err" : "ok", e.slice(1).join(a("common.listSep")))
    }
    T().then(() => {
      $(), V(), W()
    })
  },
  async capture() {
    await x(async () => {
      const t = await g("capture_current", {
        name: null
      });
      d(a("m.toastSaved", {
        name: t.name
      }), "ok", a("m.toastSavedDetail")), await T(), $(), V()
    })
  },
  async rename(t) {
    if (y && Tt(p.accounts.find(n => n.id === t)?.name)) return;
    Z = t, $();
    const e = document.querySelector(`.row[data-id="${t}"] .rename-input`);
    e && (e.focus(), e.select())
  },
  async doRename(t) {
    const n = (document.querySelector(`.row[data-id="${t}"] .rename-input`)?.value || "").trim();
    n && (window.__renameSaving = !0, clearTimeout(window.__renameBlurTimer), await x(async () => {
      const s = await g("rename_account", {
        id: t,
        name: n
      });
      d(a("m.toastRenamed", {
        name: s.name
      })), Z = null, await T(), $()
    }).finally(() => {
      window.__renameSaving = !1
    }))
  },
  cancelRename() {
    Z = null, $()
  },
  deferCancelRename(t) {
    clearTimeout(window.__renameBlurTimer), window.__renameBlurTimer = setTimeout(() => {
      Z === t && !window.__renameSaving && S.cancelRename()
    }, 180)
  },
  async delete(t) {
    const e = p?.accounts.find(n => n.id === t);
    e && rt({
      kind: "danger",
      icon: "x",
      title: a("m.deleteTitle", {
        name: R(e.name)
      }),
      desc: a("m.deleteDesc"),
      yesLabel: a("common.delete"),
      onYes: () => S.doDelete(t)
    })
  },
  async doDelete(t) {
    await x(async () => {
      await g("delete_account", {
        id: t
      }), d(a("m.toastDeleted")), await T(), $()
    })
  },
  askSwitch(t) {
    if (p.zcode_running && !p.hot_switch) {
      const e = p.accounts.find(n => n.id === t);
      if (!e) return;
      rt({
        kind: "warn",
        icon: "swap",
        title: a("m.switchTitle", {
          name: R(e.name)
        }),
        desc: `<span class="warn-line">${a("m.switchDesc")}</span>`,
        yesLabel: a("m.switchYes"),
        onYes: () => S.doSwitch(t, !0)
      })
    } else S.doSwitch(t, !1)
  },
  async doSwitch(t, e) {
    await x(async () => {
      const n = p.launch_after_switch,
        s = await g("switch_to", {
          id: t,
          force: e,
          restart: n
        });
      const i = [];
      s.killed && i.push(a("m.bitKilled")), s.preserved_as && i.push(a("m.bitPreserved", {
        name: s.preserved_as
      })), s.launched && i.push(a("m.bitLaunched")), s.hot && i.unshift(a("m.bitHot")), s.config_stale && i.push(a("m.bitConfigStale"));
      if (s.launch_error) {
        i.unshift(String(s.launch_error)), d(a(s.already_active ? "m.toastAlready" : "m.toastSwitchLaunchFail", {
          name: s.name
        }), "err", i.join(a("common.listSep")))
      } else d(a(s.already_active ? "m.toastAlready" : "m.toastSwitched", {
        name: s.name
      }), !s.already_active && (s.config_stale || s.hot) ? "warn" : "ok", i.join(a("common.listSep")));
      await T(), $(), ct(t)
    })
  },
  async updateFromLive(t) {
    await x(async () => {
      const e = await g("update_account_from_live", {
        id: t
      });
      d(a("m.toastSynced", {
        name: e.name
      }), "ok", a("m.toastSyncedDetail")), await T(), $()
    })
  },
  async exportOne(t) {
    await x(async () => {
      const e = await g("export_pick_path", {
        id: t
      });
      if (!e.picked) {
        d(a("m.exportCanceled"));
        return
      }
      gt({
        mode: "export",
        id: t,
        path: e.path,
        name: R(e.name)
      })
    })
  },
  async launch() {
    await x(async () => {
      await g("launch_zcode"), d(a("m.launching")), setTimeout(() => S.refresh(), 2500)
    })
  },
  askKill() {
    rt({
      kind: "danger",
      icon: "power",
      title: a("m.killTitle"),
      yesLabel: a("m.killYes"),
      onYes: () => S.doKill()
    })
  },
  async doKill() {
    await x(async () => {
      await g("kill_zcode"), d(a("m.toastKilled")), await T(), $()
    })
  },
  async openSettings() {
    try {
      await g("open_settings")
    } catch (t) {
      d(M(t), "err")
    }
  },
  acctQuota(t) {
    const e = I[t];
    xt(t).then(() => {
      I[t] === e && ht(t)
    })
  },
  async addAccount() {
    let t;
    try {
      t = await g("oauth_providers")
    } catch (e) {
      d(M(e), "err");
      return
    }
    ee({
      providers: t,
      onPick: async e => {
        try {
          await g("oauth_begin", {
            provider: e
          }), d(a("m.loginWindowOpened"), "ok", a("m.loginWindowDetail"))
        } catch (n) {
          S.showManualAdd(M(n))
        }
      },
      onManual: () => S.showManualAdd()
    })
  },
  showManualAdd(t = "") {
    const e = p?.live_logged_in && !p.accounts.some(s => s.is_active),
      n = p?.zcode_path_ok;
    rt({
      kind: "warn",
      icon: "userPlus",
      title: a("m.manualAddTitle"),
      desc: `${t?`<span class="warn-line">${c(t)}</span><br>`:""}${a("m.manualAddDesc")}`,
      yesLabel: e ? a("btn.saveLogin") : n ? a("m.openZcode") : a("common.settings"),
      onYes: () => e ? S.capture() : n ? S.launch() : S.openSettings()
    })
  },
  async claim(t) {
    if (E || O || w.running) {
      d(a("m.claimBusy"), "warn");
      return
    }
    const n = (b[t]?.plans || [])[0];
    if (!n) {
      d(a("m.noClaimable"), "warn");
      return
    }
    E = !0;
    try {
      await g("claim_start", {
        id: t,
        planId: n.plan_id
      }), d(a("m.claimVerify", {
        name: n.name || n.plan_id
      }), "ok", a("m.claimVerifyDetail")), await At(t) || d(a("m.claimTimeout"), "warn")
    } catch (s) {
      d(M(s), "err")
    } finally {
      E = !1
    }
  },
  async claimAll() {
    const t = (p?.accounts || []).map(e => e.id).filter(e => (b[e]?.plans || []).length > 0);
    if (!t.length) {
      d(a("m.noClaimableAccounts"), "warn");
      return
    }
    if (!(E || O || w.running)) {
      O = !0, E = !0;
      try {
        for (let e = 0; e < t.length; e++) {
          const n = t[e],
            s = b[n].plans[0],
            i = p.accounts.find(o => o.id === n)?.name || n;
          try {
            await g("claim_start", {
              id: n,
              planId: s.plan_id
            })
          } catch (o) {
            d(a("m.claimAccountErr", {
              name: i,
              err: M(o)
            }), "err");
            continue
          }
          await At(n, 12e4) || (d(a("m.claimAcctTimeout", {
            name: i
          }), "warn"), await g("claim_cancel").catch(() => {})), e < t.length - 1 && await new Promise(o => setTimeout(o, 1200))
        }
      } finally {
        O = !1, E = !1
      }
    }
  },
  async refreshClaim() {
    const t = (p?.accounts || []).map(s => s.id);
    if (!t.length) return;
    const e = Date.now();
    if (w.running || O || E && !C) return;
    if (e < w.cooldownUntil) {
      d(a("btn.refreshClaimCooldownTitle", {
        n: Math.ceil((w.cooldownUntil - e) / 1e3)
      }), "warn");
      return
    }
    if (w = {
        running: !0,
        done: 0,
        total: t.length,
        cooldownUntil: 0
      }, pe(), C) {
      mt = !0;
      const s = Date.now() + le;
      for (; C && Date.now() < s;) await new Promise(i => setTimeout(i, 300));
      if (C) {
        w.running = !1, w.cooldownUntil = Date.now() + wt, d(a("m.claimBusy"), "warn"), setTimeout(Ct, 1100);
        return
      }
    }
    let n = 0;
    try {
      for (let s = 0; s < t.length; s++) {
        const i = t[s],
          l = p.accounts.find(o => o.id === i)?.name || i;
        w.done = s + 1, b[i] = {
          plans: b[i]?.plans || [],
          busy: !0
        }, _();
        try {
          const o = await g("claim_refresh", {
            id: i
          });
          b[i] = {
            plans: o.plans || [],
            err: null,
            busy: !1
          }, (o.plans || []).length && n++, o.activationError && d(a("m.refreshClaimAcctErr", {
            name: l,
            err: M(o.activationError)
          }), "warn")
        } catch (o) {
          b[i] = {
            plans: b[i]?.plans || [],
            err: String(o),
            busy: !1
          }, d(a("m.refreshClaimAcctErr", {
            name: l,
            err: M(o)
          }), "err")
        }
        ht(i), _(), s < t.length - 1 && await new Promise(o => setTimeout(o, 5e3))
      }
    } finally {
      w.running = !1, w.cooldownUntil = Date.now() + wt, _(), setTimeout(Ct, 1100)
    }
    d(a("m.refreshClaimDone", {
      n: t.length,
      k: n
    }), "ok")
  },
  async toggleAutoClaim() {
    if (vt) return;
    const t = !p.auto_claim;
    if (t && (C || E || O || w.running)) {
      d(a("m.claimBusy"), "warn");
      return
    }
    vt = !0;
    try {
      await g("set_behavior", {
        autoClaim: t
      }), await T(), $(), p.auto_claim ? (d(a("m.autoClaimOn"), "ok", a("m.autoClaimOnDetail")), Date.now() - (z?.at ?? 0) > wt && Dt()) : z = null
    } catch (e) {
      d(M(e), "err")
    } finally {
      vt = !1
    }
  },
  async toggleAutoSwitch() {
    if (!$t) {
      $t = !0;
      try {
        await g("set_behavior", {
          autoSwitch: !p.auto_switch
        }), await T(), $(), d(a(p.auto_switch ? "m.autoSwitchOn" : "m.autoSwitchOff"), "ok", p.auto_switch ? a("m.autoSwitchOnDetail") : "")
      } catch (t) {
        d(M(t), "err")
      } finally {
        $t = !1
      }
    }
  }
};

function pe() {
  it || (it = setInterval(() => {
    _(), Ct()
  }, 1e3))
}

function Ct() {
  const t = Date.now() < w.cooldownUntil;
  !w.running && !t && it && (clearInterval(it), it = null, _())
}

function he(t) {
  const e = Date.now();
  return t.code === 1005 && t.nextAt ? t.nextAt : Number.isFinite(t.code) && t.code >= 1e3 || t.code === "interactive" ? e + 60 * 60 * 1e3 : e + ot
}
async function Dt() {
  if (!p?.auto_claim || C || E || O || w.running) return;
  const t = (p.accounts || []).map(s => s.id).filter(s => (X[s] ?? 0) <= Date.now());
  if (!t.length) {
    (p.accounts || []).some(s => (b[s.id]?.plans || []).length > 0) && (z = {
      at: Date.now(),
      claimed: 0,
      skipped: 0,
      cooldownAll: !0
    });
    return
  }
  C = !0, E = !0, mt = !1;
  let e = 0,
    n = 0;
  _();
  try {
    for (const s of t) {
      if (!p?.auto_claim || mt) break;
      if (!(p.accounts || []).some(r => r.id === s)) continue;
      let i = !1;
      b[s] = {
        ...b[s] || {},
        busy: !0
      };
      try {
        const r = await g("claim_refresh", {
          id: s
        });
        b[s] = {
          plans: r.plans || [],
          err: null,
          busy: !1
        }
      } catch (r) {
        b[s] = {
          plans: b[s]?.plans || [],
          err: String(r),
          busy: !1
        }, X[s] = Date.now() + ot, n++;
        continue
      }
      _();
      let l = 0,
        o = !0;
      for (; o && l < ie && !mt;) {
        l++, o = !1;
        const r = b[s]?.plans?.[0];
        if (!r) break;
        try {
          await g("claim_start", {
            id: s,
            planId: r.plan_id,
            auto: !0
          })
        } catch {
          await g("claim_cancel").catch(() => {}), X[s] = Date.now() + ot;
          break
        }
        const h = await At(s, se);
        if (!h) {
          await g("claim_cancel").catch(() => {}), X[s] = Date.now() + ot;
          break
        }
        if (h.ok === !1) {
          X[s] = he(h);
          break
        }
        i = !0, e++, await de(s), _(), o = !0, await new Promise(v => setTimeout(v, 1200))
      }!i && (b[s]?.plans || []).length && n++, await new Promise(r => setTimeout(r, oe))
    }
  } finally {
    C = !1, E = !1, p?.auto_claim && (z = {
      at: Date.now(),
      claimed: e,
      skipped: n
    }), _()
  }
}

function jt(t) {
  const e = t == null ? null : Math.min(100, Math.max(0, t)),
    n = e == null ? null : 100 - e,
    s = e != null && e >= 90 ? " danger" : e != null && e >= 70 ? " warn" : "",
    i = n == null ? "--" : n.toFixed(0) + "%";
  const bar = n == null ? "" : `<div class="qbar${s}" role="meter" aria-label="${a("m.remaining")}" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${n}"><div class="qbar-fill" style="width:${n}%"></div></div>`;
  return `<div class="quota-meter">${bar}<span class="meter-value">${i}</span></div>`
}

function tt(t) {
  return t.kind ? t.kind : t.name.includes("提示次数") ? "prompt_count" : t.name.includes("使用时长") ? "duration" : "raw"
}

function fe(t) {
  if (t.window) return t.window.startsWith("hours:") ? a("q.win.hours", {
    n: t.window.slice(6)
  }) : Xt(`q.win.${t.window}`) ? a(`q.win.${t.window}`) : t.window;
  const e = t.name.match(/[（(]每\s*([^）)]+)[）)]/);
  return e ? "每" + e[1].replace(/^每/, "") : tt(t) === "duration" ? a("q.monthlyShort") : tt(t) === "prompt_count" ? a("q.countShort") : t.name
}

function Nt(t) {
  return t.reset ? a("q.resets", {
    time: t.reset
  }) : t.period_end || ""
}

function Yt(t, e = "") {
  return `
  <div class="q-win${e}">
    <span class="q-win-label">${c(fe(t))}</span>
    ${jt(t.percent_used)}
    <span class="q-win-reset" title="${c(Nt(t))}">${t.reset||t.period_end?c(Nt(t)):""}</span>
  </div>`
}

function F(t) {
  return t == null ? "" : Lt() === "zh" ? t >= 1e8 ? (t / 1e8).toFixed(t % 1e8 === 0 ? 0 : 1) + "亿" : t >= 1e6 ? (t / 1e6).toFixed(t % 1e6 === 0 ? 0 : 1) + "M" : t >= 1e3 ? Math.round(t / 1e3) + "K" : String(Math.round(t)) : t >= 1e9 ? (t / 1e9).toFixed(t % 1e9 === 0 ? 0 : 1) + "B" : t >= 1e6 ? (t / 1e6).toFixed(t % 1e6 === 0 ? 0 : 1) + "M" : t >= 1e3 ? Math.round(t / 1e3) + "K" : String(Math.round(t))
}

function lt(t) {
  const e = `${t?.unit||""} ${t?.unit_code||""} ${t?.name||""}`.toLowerCase();
  if (!e.includes("token") && !e.includes("令牌") || t?.kind === "grant") return null;
  const n = o => o != null && o !== "" && Number.isFinite(Number(o)) ? Number(o) : null,
    s = n(t?.total),
    i = n(t?.used),
    l = n(t?.remaining);
  return l != null ? Math.max(0, l) : s != null && i != null ? Math.max(0, s - i) : null
}

function Kt(t) {
  let e = 0,
    n = 0,
    s = 0;
  for (const i of t || []) {
    const l = D[i.id];
    if (!l || l.busy && !l.data || l.err) {
      s++;
      continue
    }
    if (!l.data) continue;
    const o = new Set;
    let r = 0;
    for (const h of l.data.items || []) {
      const v = lt(h);
      if (v == null) continue;
      const N = [h.name, h.unit, h.period_end, h.window, h.total, h.remaining, h.used].join("|");
      o.has(N) || (o.add(N), r += v)
    }
    o.size && (e += r, n++)
  }
  return {
    total: e,
    knownAccounts: n,
    pendingAccounts: s,
    accountCount: (t || []).length
  }
}
const Vt = "zaccounts.tokenHistory.v1",
  Ot = "zaccounts.tokenBalances.v1";

function Mt(t = new Date) {
  const e = t.getFullYear(),
    n = String(t.getMonth() + 1).padStart(2, "0"),
    s = String(t.getDate()).padStart(2, "0");
  return `${e}-${n}-${s}`
}

function Wt() {
  try {
    const t = JSON.parse(localStorage.getItem(Vt) || "[]");
    return Array.isArray(t) ? t.filter(e => e && typeof e.day == "string") : []
  } catch {
    return []
  }
}

function ge(t) {
  const e = D[t];
  if (!e?.data) return;
  const n = {};
  for (const k of e.data.items || []) {
    const q = lt(k);
    if (q == null) continue;
    const L = [k.name, k.unit_code || k.unit, k.window, k.period_end, k.total].join("|");
    n[L] = (n[L] || 0) + q
  }
  if (!Object.keys(n).length) return;
  let s;
  try {
    s = JSON.parse(localStorage.getItem(Ot) || "{}")
  } catch {
    s = {}
  }(!s || typeof s != "object" || Array.isArray(s)) && (s = {});
  const i = s[t],
    l = Date.now(),
    o = Mt();
  let r = 0;
  for (const [k, q] of Object.entries(n)) {
    const L = i?.items?.[k];
    Number.isFinite(L) && (r += Math.max(0, L - q))
  }
  s[t] = {
    items: n,
    updatedAt: l
  }, localStorage.setItem(Ot, JSON.stringify(s));
  const h = Wt().sort((k, q) => k.day.localeCompare(q.day)),
    v = h.find(k => k.day === o);
  v ? (v.consumed = Number(v.consumed || 0) + r, v.updatedAt = l) : h.push({
    day: o,
    consumed: r,
    updatedAt: l
  });
  const N = h.slice(-90);
  localStorage.setItem(Vt, JSON.stringify(N))
}

function be() {
  const t = new Map(Wt().map(n => [n.day, n]));
  if (!t.size) return [];
  const e = new Date;
  return Array.from({
    length: 7
  }, (n, s) => {
    const i = new Date(e.getFullYear(), e.getMonth(), e.getDate() - 6 + s),
      l = Mt(i);
    return t.get(l) || {
      day: l,
      consumed: 0
    }
  })
}

function Ft(t) {
  const e = new Date(`${t}T00:00:00`);
  return Number.isNaN(e.getTime()) ? t : e.toLocaleDateString(H(), {
    month: "numeric",
    day: "numeric"
  })
}

function we(t) {
  const e = D[t.id];
  if (!e || e.busy && !e.data) return {
    label: a("m.dashboardChecking"),
    cls: "checking"
  };
  if (e.err) return {
    label: a("m.dashboardUnavailable"),
    cls: "error"
  };
  if (e.busy) return {
    label: a("m.dashboardRefreshing"),
    cls: "checking"
  };
  const s = (e.data?.items || []).map(lt).filter(i => i != null);
  return e.data ? s.length ? {
    label: s.some(i => i > 0) ? a("m.dashboardAvailable") : a("m.dashboardExhausted"),
    cls: s.some(i => i > 0) ? "ok" : "warn"
  } : e.data.plans?.length || e.data.plan_tier ? {
    label: a("m.dashboardPlanDetected"),
    cls: "ok"
  } : {
    label: a("m.dashboardNoData"),
    cls: "muted"
  } : {
    label: a("m.dashboardNoData"),
    cls: "muted"
  }
}

function ve(t) {
  const e = Kt(t.accounts),
    n = be(),
    s = n.find(f => f.day === Mt()),
    i = Math.max(1, ...n.map(f => Number(f.consumed || 0))),
    l = t.accounts.find(f => f.is_active),
    o = l ? D[l.id] : null,
    r = !o?.err && o?.data?.items?.map(lt).filter(f => f != null) || [],
    h = Math.max(0, ...t.accounts.map(f => Number(D[f.id]?.data?.refreshed_at || 0))),
    v = h ? new Date(h).toLocaleTimeString(H(), {
      hour: "2-digit",
      minute: "2-digit"
    }) : "--",
    N = [{
      icon: "grid",
      label: a("m.dashboardAccounts"),
      value: zt(t.accounts.length),
      detail: a("m.dashboardAccountsDetail", {
        active: l ? 1 : 0
      })
    }, {
      icon: "token",
      label: a("m.dailyTokens"),
      value: e.knownAccounts ? F(e.total) : "--",
      detail: a("m.dailyTokensDetail", {
        known: e.knownAccounts,
        total: e.accountCount
      })
    }, {
      icon: "activity",
      label: a("m.dashboardConsumed"),
      value: s ? F(s.consumed || 0) : "--",
      detail: a("m.dashboardToday")
    }, {
      icon: "check",
      label: a("m.dashboardActiveQuota"),
      value: r.length ? F(r.reduce((f, j) => f + j, 0)) : "--",
      detail: l ? R(l.name) : a("m.dashboardNoActive")
    }],
    k = n.length ? n.map(f => `<div class="dash-bar-col" title="${c(`${Ft(f.day)} · ${F(f.consumed||0)}`)}"><div class="dash-bar" style="height:${Math.max(4,Math.round(Number(f.consumed||0)/i*100))}%"></div><span>${c(Ft(f.day))}</span></div>`).join("") : `<div class="dashboard-empty">${a("m.dashboardHistoryEmpty")}</div>`,
    q = [...t.accounts].sort((f, j) => Number(j.is_active) - Number(f.is_active)).map(f => {
      const j = D[f.id],
        Y = !j?.err && j?.data?.items?.map(lt).filter(ft => ft != null) || [],
        Q = we(f);
      return `<div class="dash-account-row"><span class="dash-avatar" style="--avatar-color:${Bt(f.id)}">${c((f.name||"Z").slice(0,1).toUpperCase())}</span><div class="dash-account-name"><strong>${c(R(f.name))}</strong><span>${f.is_active?a("m.dashboardActive"):a("m.dashboardSaved")}</span></div><span class="dash-account-token">${Y.length?c(F(Y.reduce((ft,Qt)=>ft+Qt,0))+" Token"):"--"}</span><span class="dash-status ${Q.cls}"><i></i>${c(Q.label)}</span></div>`
    }).join(""),
    L = B?.kind,
    at = [B?.status_code, B?.provider_code].filter(f => f != null).join(" / "),
    u = B?.at ? new Date(B.at).toLocaleTimeString(H(), {
      hour: "2-digit",
      minute: "2-digit"
    }) : "--",
    J = B?.request_id ? `<small>${c(a("m.modelRequestId",{id:B.request_id}))}</small>` : "",
    nt = L === "gateway_blocked" || L === "rate_limited" ? `<section class="dashboard-model-alert ${L}"><span class="dashboard-model-alert-icon">${m("activity",18)}</span><div><strong>${c(a(L==="gateway_blocked"?"m.modelGatewayBlocked":"m.modelRateLimited",{code:at}))}</strong><p>${c(a(L==="gateway_blocked"?"m.modelGatewayBlockedDetail":"m.modelRateLimitedDetail"))}</p>${J}</div><span class="dashboard-model-alert-time">${c(a("m.modelLastRequest",{time:u}))}</span></section>` : "";
  return `<div class="dashboard-page">
    <div class="dashboard-title"><div><h1>${a("m.dashboard")}</h1><p>${a("m.dashboardSubtitle")}</p></div><div class="dashboard-title-actions"><span class="dashboard-last-sync">${m("refresh",13)} ${c(a("m.dashboardLastSync",{time:v}))}</span><button class="tog-inline${t.auto_switch?" on":""}" role="switch" aria-checked="${t.auto_switch}" aria-label="${a("btn.autoSwitch")}" title="${a("btn.autoSwitchTitle")}" click="actions.toggleAutoSwitch()"><span class="toggle${t.auto_switch?" on":""}" aria-hidden="true"><span class="knob"></span></span>${a("btn.autoSwitch")}</button><button class="btn-ghost has-ic privacy-toggle${y?" is-private":""}" aria-pressed="${y}" aria-label="${a(y?"m.showEmails":"m.hideEmails")}" title="${a(y?"m.showEmails":"m.hideEmails")}" click="actions.togglePrivacy()">${m(y?"eyeOff":"eye",15)}</button><button class="btn-ghost has-ic" click="actions.refresh()">${m("refresh",15)} ${a("m.refreshAccounts")}</button></div></div>
    ${nt}
    <div class="dashboard-stat-grid">${N.map(f=>`<div class="dashboard-stat"><span class="dashboard-stat-icon">${m(f.icon,17)}</span><div><span>${c(f.label)}</span><strong>${c(f.value)}</strong><small>${c(f.detail)}</small></div></div>`).join("")}</div>
    <div class="dashboard-grid"><section class="dashboard-panel usage-panel"><div class="dashboard-panel-head"><div><h2>${a("m.dashboardUsageTrend")}</h2><p>${a("m.dashboardUsageHint")}</p></div><span class="panel-period">${a("m.dashboardSevenDays")}</span></div><div class="dash-chart">${k}</div></section><section class="dashboard-panel status-panel"><div class="dashboard-panel-head"><div><h2>${a("m.dashboardAccountStatus")}</h2><p>${a("m.dashboardStatusHint")}</p></div></div><div class="dash-account-list">${q||`<div class="dashboard-empty">${a("m.dashboardNoAccounts")}</div>`}</div></section></div>
    <section class="dashboard-panel dashboard-note"><span class="dashboard-note-icon">${m("lock",16)}</span><div><strong>${a("m.dashboardLocalTitle")}</strong><p>${a("m.dashboardLocalBody")}</p></div></section>
  </div>`
}

function $e(t) {
  const e = Kt(t),
    n = e.knownAccounts ? F(e.total) : "--",
    s = e.pendingAccounts ? a("m.dailyTokensPending", {
      known: e.knownAccounts,
      total: e.accountCount
    }) : e.knownAccounts ? a("m.dailyTokensDetail", {
      known: e.knownAccounts,
      total: e.accountCount
    }) : a("m.dailyTokensNone");
  return `<div class="daily-token" title="${c(s)}">
    <span class="daily-token-icon">${m("token",15)}</span>
    <span class="daily-token-copy"><span class="daily-token-label">${a("m.dailyTokens")}</span><strong>${c(n)}</strong></span>
    <span class="daily-token-detail">${c(s)}</span>
  </div>`
}

function ye(t) {
  const e = t.total != null && t.remaining != null ? `${F(t.remaining)}/${F(t.total)}` : "",
    n = t.name.replace(/^GLM-?/i, "");
  return `
  <div class="q-win mini">
    <span class="q-win-label" title="${c(t.name)}">${c(n)}</span>
    ${jt(t.percent_used)}
    <span class="q-win-reset">${c(e)}</span>
  </div>`
}

function ke(t) {
  const e = t.name.replace(/^GLM-?/i, ""),
    n = t.unit?.toLowerCase() === "token" ? " Token" : ` ${t.unit||""}`;
  return `<div class="grant-row" title="${c(a("q.grantPending"))}">
    <span class="grant-name">${c(e)}</span>
    <span class="grant-amount">${c(F(t.total)+n)}</span>
    <span class="grant-note">${c(a("q.grantPending"))}</span>
  </div>`
}

function Jt(t, e) {
  const n = String(e || "").toLowerCase(),
    s = String(t || "").toLowerCase();
  let i, l;
  return n === "max" || !n && s.includes("max") ? (i = "Max", l = "max") : n === "pro" || !n && s.includes("pro") ? (i = "Pro", l = "pro") : n === "lite" || !n && s.includes("lite") ? (i = "Lite", l = "lite") : n === "start" ? (i = "Start", l = "trial") : n === "trial" || !n && (s.includes("trial") || String(t || "").includes("体验")) ? (i = a("q.trial"), l = "trial") : (i = t, l = "other"), `<span class="tier-b ${l}">${c(i)}</span>`
}

function _e(t) {
  const e = D[t];
  if (!e?.data) return "";
  const n = e.data.plans || [],
    s = [...new Map(n.filter(l => l.tier).map(l => [`${l.tier}:${l.tier_code||""}`, [l.tier, l.tier_code]])).values()],
    i = (s.length ? s : e.data.plan_tier ? [
      [e.data.plan_tier, null]
    ] : []).slice(0, 2);
  return i.length ? i.map(([l, o]) => Jt(l, o)).join("") : '<span class="tier-b free">Free</span>'
}

function Se(t) {
  const e = t.grant_items || [];
  if (e.length) {
    const n = e[0];
    return a("q.grant", {
      name: n.name,
      amount: F(n.units),
      period: a(`q.period.${n.period}`, {}) === `q.period.${n.period}` ? n.period : a(`q.period.${n.period}`, {})
    })
  }
  return (t.grants || [])[0] || ""
}

function Te(t) {
  const n = b[t]?.plans?.[0];
  if (!n) return "";
  const s = Se(n),
    i = n.name || n.plan_id;
  return `
  <div class="claim-strip" title="${c(n.description||i)}">
    ${m("gift",15)}
    <span class="claim-name">${c(i)}</span>
    ${s?`<span class="claim-grants">${c(s)}</span>`:""}
    <button class="btn-claim has-ic" click="actions.claim('${t}')" ${O||w.running||E||C?"disabled":""}>${m("gift",13)} ${a("btn.claim")}</button>
  </div>`
}

function Zt(t) {
  const e = t || [],
    n = r => tt(r) !== "raw",
    s = e.filter(r => n(r) && tt(r) !== "grant"),
    i = e.filter(r => tt(r) === "grant"),
    l = new Map;
  for (const r of e) {
    if (n(r)) continue;
    const h = l.get(r.name);
    (!h || (r.total || 0) > (h.total || 0)) && l.set(r.name, r)
  }
  const o = [...l.values()].sort((r, h) => (h.total || 0) - (r.total || 0));
  return [...s.map(r => Yt(r, " mini")), ...o.map(ye), ...i.map(ke)].join("")
}

function Gt(t) {
  if (!t) return null;
  const e = t.length >= 16,
    n = new Date(e ? t.replace(" ", "T") : t + "T23:59:59") - Date.now();
  if (isNaN(n)) return {
    text: t,
    soon: !1,
    warn: !1
  };
  const s = n <= 5 * 864e5,
    i = n <= 7 * 864e5;
  return {
    text: s && e ? t : t.slice(0, 10),
    soon: s,
    warn: i
  }
}

function Ae(t) {
  const e = t.tier_code === "other" && !t.pid ? a("q.other") : t.name || t.tier || "",
    n = Gt(t.expire);
  return `
  <div class="plan-grp">
    <div class="pg-head">
      ${t.tier?Jt(t.tier,t.tier_code):""}
      <span class="pg-name" title="${c(e)}">${c(e)}</span>
      ${n?`<span class="pg-exp${n.warn?" warn-line":""}" title="${c(a("q.validUntil",{date:n.text}))}">${c(a("q.validUntilShort",{date:n.text}))}</span>`:""}
    </div>
    ${Zt(t.items)}
  </div>`
}

function Ce(t) {
  const e = Te(t),
    n = D[t];
  let s = "";
  if (!n || n.busy && !n.data) s = `<span class="aq-loading">${a("q.loading")}</span>`;
  else if (n?.err && !n.data) {
    const i = n.err.length > 46 ? n.err.slice(0, 46) + "…" : n.err;
    s = `<span class="aq-err">${c(i)}</span>`
  } else if (n?.data) {
    const i = n.data.plans || [];
    if (i.length >= 2) s = i.map(Ae).join("");
    else {
      const l = n.data.items || [],
        o = l.filter(r => tt(r) === "prompt_count");
      o.length ? s = o.map(r => Yt(r, " mini")).join("") : s = Zt(l)
    }!s && n.data.total != null && (s = `<div class="quota-summary"><span>${a("q.totalGrant")}</span><strong>${c(zt(n.data.total))}</strong><small>${a("q.grantPending")}</small></div>`), s || (s = `<span class="aq-empty">${a(n.data.is_empty?"q.noPlan":"q.noDetails")}</span>`), n.busy && (s = `<span class="aq-refreshing">${a("q.refreshingPrevious")}</span>${s}`), n.err && (s = `<span class="aq-err">${a("q.refreshFailedPrevious")}: ${c(n.err)}</span>${s}`)
  }
  return `<div class="row-quota-slot">${e}${s}</div>`
}

function De() {
  const t = U.querySelector(".list");
  if (!t || t.scrollTop === 0) return null;
  const e = t.getBoundingClientRect().top;
  for (const n of t.querySelectorAll(".row[data-id]"))
    if (n.getBoundingClientRect().bottom > e) return {
      id: n.dataset.id,
      offset: n.getBoundingClientRect().top - e,
      scrollTop: t.scrollTop
    };
  return {
    scrollTop: t.scrollTop
  }
}

function Le(t) {
  if (!t) return;
  const e = U.querySelector(".list");
  if (!e) return;
  const n = t.id ? e.querySelector(`.row[data-id="${CSS.escape(t.id)}"]`) : null;
  if (n) {
    const s = n.getBoundingClientRect().top - e.getBoundingClientRect().top;
    e.scrollTop = s - t.offset
  } else e.scrollTop = t.scrollTop
}

function xe() {
  dt?.disconnect(), dt = null;
  const t = U.querySelector(".list"),
    e = U.querySelector(".list-scroll-rail"),
    n = U.querySelector(".list-scroll-thumb");
  if (!t || (t.addEventListener("scroll", () => st(), {
      passive: !0
    }), t.addEventListener("wheel", () => st(), {
      passive: !0
    }), !e || !n)) return;
  const s = () => {
    const o = t.scrollHeight - t.clientHeight;
    if (o <= 1) {
      e.classList.remove("visible");
      return
    }
    e.classList.add("visible");
    const r = Math.max(1, e.clientHeight),
      h = Math.max(38, Math.round(r * t.clientHeight / t.scrollHeight)),
      v = Math.max(0, r - h);
    n.style.height = `${h}px`, n.style.transform = `translateY(${Math.round(v*t.scrollTop/o)}px)`
  };
  t.addEventListener("scroll", s, {
    passive: !0
  });
  let i = null;
  const l = o => {
    i && (o?.pointerId != null && n.hasPointerCapture(o.pointerId) && n.releasePointerCapture(o.pointerId), i = null, _t = !1, st())
  };
  n.addEventListener("pointerdown", o => {
    o.preventDefault();
    const r = Math.max(1, t.scrollHeight - t.clientHeight),
      h = Math.max(1, e.clientHeight - n.offsetHeight);
    i = {
      pointerId: o.pointerId,
      startY: o.clientY,
      startTop: t.scrollTop,
      range: r,
      travel: h
    }, _t = !0, st(), n.setPointerCapture(o.pointerId)
  }), n.addEventListener("pointermove", o => {
    if (!i || o.pointerId !== i.pointerId) return;
    const r = o.clientY - i.startY;
    t.scrollTop = Math.max(0, Math.min(i.range, i.startTop + r * i.range / i.travel)), s()
  }), n.addEventListener("pointerup", l), n.addEventListener("pointercancel", l), e.addEventListener("pointerdown", o => {
    if (o.target !== e) return;
    const r = e.getBoundingClientRect(),
      h = Math.max(1, t.scrollHeight - t.clientHeight),
      v = Math.max(1, e.clientHeight - n.offsetHeight),
      N = Math.max(0, Math.min(v, o.clientY - r.top - n.offsetHeight / 2));
    t.scrollTop = N * h / v, st(), s()
  }), dt = new ResizeObserver(s), dt.observe(t), s()
}

function $() {
  const t = De(),
    e = document.activeElement?.id === "account-search",
    n = e ? [document.activeElement.selectionStart, document.activeElement.selectionEnd] : null;
  if (!p) {
    U.innerHTML = '<div class="loading">LOADING</div>';
    return
  }
  const s = p;
  Pt = JSON.stringify(s), document.documentElement.dataset.theme = s.theme || "dark";
  const i = s.accounts.find(u => u.is_active) || null,
    l = s.live_logged_in && !i,
    o = pt.trim().toLocaleLowerCase(),
    r = u => !o || [u.name, u.identity?.username, u.identity?.email].filter(Boolean).join(" ").toLocaleLowerCase().includes(o),
    h = i && r(i) ? i : null,
    v = P === "active" ? [] : s.accounts.filter(u => !u.is_active && r(u)),
    N = s.zcode_running ? "run" : s.live_logged_in ? "" : "off",
    k = s.zcode_running ? a("m.status.running") : s.live_logged_in ? l ? a("m.status.unsaved") : a("m.status.safe") : a("m.status.loggedOut"),
    q = u => {
      const J = u.is_active;
      if (Z === u.id) return `
      <div class="row account-card${J?" active":""}" data-id="${u.id}">
        <div class="row-main">
          <input class="rename-input" value="${c(u.name)}" maxlength="40"
            keydown="onRenameKey(event,'${u.id}')" blur="actions.deferCancelRename('${u.id}')">
          <div class="row-meta">${a("btn.renameMeta")}</div>
        </div>
        <div class="row-actions">
          <button class="btn-ghost" style="padding:4px 10px" click="actions.doRename('${u.id}')">${a("common.save")}</button>
          <button class="btn-ghost" style="padding:4px 10px" click="actions.cancelRename()">${a("common.cancel")}</button>
        </div>
      </div>`;
      const nt = R(u.name),
        f = [...new Set([u.identity?.username && R(u.identity.username), u.identity?.email && (y ? a("m.emailHidden") : u.identity.email)].filter(Boolean))].join(" · "),
        j = D[u.id];
      let Y = "";
      u.has_config || (Y += `<span class="no-cfg">${a("q.noCfg")}</span>`);
      const Q = Gt(j?.data?.plan_expire);
      return Q && (Y += `<span class="${Q.warn?"warn-line":""}">${c(a("q.validUntil",{date:Q.text}))}</span>`), f && (Y += `${Y?" · ":""}${c(f)}`), `
    <div class="row account-card${J?" active":""}" data-id="${u.id}">
      <div class="row-top">
        <span class="account-avatar" style="--avatar-color:${Bt(u.id)}">${c((nt||"Z").trim().slice(0,1).toUpperCase())}</span>
        <div class="row-main">
          <div class="row-name"><span class="account-name" title="${c(nt)}">${c(nt)}</span>${_e(u.id)}${u.has_user_info===!1?`<span class="tag-relogin" title="${c(a("btn.reloginTitle"))}">${a("btn.relogin")}</span>`:""}</div>
          <div class="row-meta">${Y}</div>
        </div>
        ${J?`<span class="tag-use"><span class="status-dot"></span>${a("btn.inUse")}</span>`:""}
      </div>
      ${Ce(u.id)}
      <div class="row-actions">
        <div class="card-tools">
          <button class="icon-btn" title="${a("btn.quota")}" aria-label="${a("btn.quota")}" click="actions.acctQuota('${u.id}')">${m("gauge",16)}</button>
          <button class="icon-btn" title="${y&&Tt(u.name)?a("m.showEmailToRename"):a("btn.rename")}" aria-label="${a("btn.rename")}" click="actions.rename('${u.id}')" ${y&&Tt(u.name)?"disabled":""}>${m("pen",16)}</button>
          <button class="icon-btn" title="${a("btn.export")}" aria-label="${a("btn.export")}" click="actions.exportOne('${u.id}')">${m("export",16)}</button>
          <button class="icon-btn danger" title="${a("btn.delete")}" aria-label="${a("btn.delete")}" click="actions.delete('${u.id}')">${m("trash",16)}</button>
        </div>
        <button class="btn-switch has-ic" click="actions.askSwitch('${u.id}')" ${J?"disabled":""}>
          ${J?m("check",16)+" "+a("btn.current"):m("swap",16)+" "+a("btn.switch")}
        </button>
      </div>
    </div>`
    },
    L = s.accounts.length === 0 ? `<div class="empty">
         <div class="glyph">${m("empty",34)}</div>
         ${a("m.emptyTitle")}<br>
         ${a("m.emptyBody")}
       </div>` : `${h?`<div class="section-label"><span class="section-indicator"></span>${a("m.activeSection")}</div>${q(h)}`:o?"":`<div class="active-placeholder">${m("userPlus",22)}<span>${a("m.noActiveAccount")}</span></div>`}
       ${P!=="active"&&v.length?`<div class="section-label other-label">${a("m.otherSection",{count:v.length})}</div><div class="other-grid${G==="list"?" list-view":""}">${v.map(q).join("")}</div>`:""}
       ${o&&!h&&!v.length?`<div class="empty">${m("search",28)}<h2>${a("m.noResults")}</h2><p>${a("m.noResultsHint")}</p></div>`:""}`,
    at = s.accounts.filter(u => (b[u.id]?.plans || []).length > 0).length;
  if (U.innerHTML = `
  <div class="desktop-shell">
    <aside class="sidebar">
      <div class="brand"><img class="brand-mark" src="/brand-icon.png" alt=""><div class="brand-copy"><span class="wordmark">Z-Accounts</span><span class="brand-caption">${a("m.brandCaption")}</span></div></div>
      <div class="sidebar-section">${a("m.sidebarLibrary")}</div>
      <nav class="sidebar-nav" aria-label="${a("m.sidebarLibrary")}">
        <button class="nav-item${A==="dashboard"?" selected":""}" click="actions.setPage('dashboard')" ${A==="dashboard"?'aria-current="page"':""}>${m("activity",17)}<span>${a("m.dashboard")}</span></button>
        <button class="nav-item${A==="accounts"&&P==="all"?" selected":""}" click="actions.setFilter('all')" ${A==="accounts"&&P==="all"?'aria-current="page"':""}>${m("grid",17)}<span>${a("m.allAccounts")}</span><span class="nav-count">${s.accounts.length}</span></button>
        <button class="nav-item${A==="accounts"&&P==="active"?" selected":""}" click="actions.setFilter('active')" ${A==="accounts"&&P==="active"?'aria-current="page"':""}>${m("check",17)}<span>${a("m.activeSection")}</span><span class="nav-count">${i?1:0}</span></button>
      </nav>
      <div class="sidebar-section">${a("m.sidebarTools")}</div>
      <button class="nav-item${l?" attention":""}" click="actions.capture()" ${!s.live_logged_in||i?"disabled":""} title="${i?c(a("m.saveLoginDisabledTitle",{name:R(i.name)})):""}">${m("capture",17)}<span>${a("btn.saveLogin")}</span></button>
      <button class="nav-item" click="actions.openSettings()">${m("sliders",17)}<span>${a("common.settings")}</span></button>
      <div class="sidebar-bottom">
        <div class="client-panel"><div class="client-heading"><span class="client-icon">Z</span><div><strong>ZCode</strong><span class="client-status"><span class="status-dot ${N}"></span>${c(k)}</span></div></div>
          ${s.zcode_running?`<button class="client-action has-ic" click="actions.askKill()">${m("power",14)}${a("btn.killZcode")}</button>`:`<button class="client-action has-ic" click="actions.launch()" ${s.zcode_path_ok?"":"disabled"}>${m("play",13)}${a("btn.launchZcode")}</button>`}
        </div>
        <div class="preference-row"><span>${a("m.languageShort")}</span><div class="lang-seg compact" role="radiogroup" aria-label="${a("s.langLabel")}">
          <button class="lang-opt${s.language==="zh"?" on":""}" role="radio" aria-checked="${s.language==="zh"}" click="actions.setLang('zh')">中文</button>
          <button class="lang-opt${s.language==="en"?" on":""}" role="radio" aria-checked="${s.language==="en"}" click="actions.setLang('en')">EN</button>
        </div></div>
        <div class="preference-row"><span>${a("m.appearance")}</span><div class="lang-seg compact" role="radiogroup" aria-label="${a("m.appearance")}">
          <button class="lang-opt${s.theme==="light"?" on":""}" role="radio" aria-checked="${s.theme==="light"}" click="actions.setTheme('light')" aria-label="${a("m.lightMode")}" title="${a("m.lightMode")}">${m("sun",15)}</button>
          <button class="lang-opt${s.theme!=="light"?" on":""}" role="radio" aria-checked="${s.theme!=="light"}" click="actions.setTheme('dark')" aria-label="${a("m.darkMode")}" title="${a("m.darkMode")}">${m("moon",15)}</button>
        </div></div>
        <div class="sidebar-version">Z-Accounts${yt?` <span>v${c(yt)}</span>`:""}</div>
      </div>
    </aside>
    <section class="workspace">
    <header class="topbar">
      <div class="breadcrumb"><span>${a("m.sidebarLibrary")}</span>${m("chevron",12)}<strong>${a(A==="dashboard"?"m.dashboard":P==="active"?"m.activeSection":"m.allAccounts")}</strong></div>
      <div class="topbar-actions">
        <button class="btn-ghost has-ic" click="actions.importFiles()">${m("import",16)} ${a("m.importShort")}</button>
        <button class="btn-ghost has-ic" click="actions.exportAll()" ${s.accounts.length?"":"disabled"}>${m("export",16)} ${a("m.exportShort")}</button>
        <span class="action-divider"></span>
        <button class="btn-primary has-ic" click="actions.addAccount()" title="${a("btn.addAccountTitle")}">${m("plus",16)} ${a("m.addAccountShort")}</button>
      </div>
    </header>
    <main class="list${A==="dashboard"?" dashboard-list":""}">
    ${A==="dashboard"?ve(s):`
      <div class="dashboard-heading"><div><h1>${a(P==="active"?"m.activeSection":"m.accounts")}</h1><p>${a("m.librarySubtitle",{count:s.accounts.length})}</p></div>
        <div class="dashboard-heading-actions">${$e(s.accounts)}
          <label class="account-search">${m("search",16)}<input id="account-search" type="search" autocomplete="off" value="${c(pt)}" placeholder="${a("m.searchAccounts")}" aria-label="${a("m.searchAccounts")}" input="actions.search(event)"></label>
        </div>
      </div>
    <section class="toolbar">
      <button class="icon-btn refresh-accounts" click="actions.refresh()" aria-label="${a("m.refreshAccounts")}" title="${a("m.refreshAccounts")}">${m("refresh",16)}</button>
      ${at>0?`<button class="btn-ghost has-ic claim-all" click="actions.claimAll()" ${O||w.running||C?"disabled":""}
            title="${a("btn.claimAllTitle")}">${m("gift",16)} ${a("btn.claimAll")}${at>1?` (${at})`:""}</button>`:""}
      ${s.accounts.length>0?`<button class="btn-ghost has-ic" click="actions.refreshClaim()"
            ${w.running||O||Date.now()<w.cooldownUntil?"disabled":""}
            title="${Date.now()<w.cooldownUntil&&!w.running?c(a("btn.refreshClaimCooldownTitle",{n:Math.ceil((w.cooldownUntil-Date.now())/1e3)})):c(a("btn.refreshClaimTitle"))}">
            ${m("refresh",16)} ${w.running?c(a("btn.refreshClaimRunning",{done:w.done,total:w.total})):c(a("btn.refreshClaim"))}
          </button>`:""}
      <button class="tog-inline${s.auto_claim?" on":""}${C?" running":""}"
        role="switch" aria-checked="${s.auto_claim}" aria-label="${a("btn.autoClaim")}"
        title="${me(s)}"
        click="actions.toggleAutoClaim()">
        <span class="toggle${s.auto_claim?" on":""}" aria-hidden="true"><span class="knob"></span></span>
        ${a("btn.autoClaim")}
      </button>
      <button class="tog-inline${s.auto_switch?" on":""}"
        role="switch" aria-checked="${s.auto_switch}" aria-label="${a("btn.autoSwitch")}" title="${a("btn.autoSwitchTitle")}" click="actions.toggleAutoSwitch()">
        <span class="toggle${s.auto_switch?" on":""}" aria-hidden="true"><span class="knob"></span></span>
        ${a("btn.autoSwitch")}
      </button>
      <span class="tb-spacer"></span>
      <button class="btn-ghost has-ic privacy-toggle${y?" is-private":""}" role="button" aria-pressed="${y}" aria-label="${a(y?"m.showEmails":"m.hideEmails")}" title="${a(y?"m.showEmails":"m.hideEmails")}" click="actions.togglePrivacy()">${m(y?"eyeOff":"eye",15)}<span>${a(y?"m.showEmails":"m.hideEmails")}</span></button>
      <div class="lang-seg compact view-seg" role="radiogroup" aria-label="${a("m.viewMode")}">
        <button class="lang-opt${G==="grid"?" on":""}" role="radio" aria-checked="${G==="grid"}" click="actions.setView('grid')" aria-label="${a("m.gridView")}" title="${a("m.gridView")}">${m("grid",15)}</button>
        <button class="lang-opt${G==="list"?" on":""}" role="radio" aria-checked="${G==="list"}" click="actions.setView('list')" aria-label="${a("m.listView")}" title="${a("m.listView")}">${m("list",15)}</button>
      </div>
    </section>
      ${L}
      <div class="workspace-foot">${m("lock",12)}${a("m.storedLocally")}</div>
    `}
    </main>
    ${A==="accounts"?'<div class="list-scroll-rail" aria-hidden="true"><div class="list-scroll-thumb"></div></div>':""}
    </section>
  </div>
  `, Le(t), xe(), e) {
    const u = document.getElementById("account-search");
    u?.focus({
      preventScroll: !0
    }), u && n && u.setSelectionRange(...n)
  }
}
window.actions = S;
window.onRenameKey = (t, e) => {
  t.key === "Enter" && S.doRename(e), t.key === "Escape" && S.cancelRename()
};
te();
et("tray-action", t => {
  const e = t.payload || {};
  e.action === "capture" && e.ok ? d(a("m.toastSaved", {
    name: e.result.name
  })) : !e.ok && e.error && d(e.error, "err"), T().then(() => {
    _(), V(), ct(e.result?.id), W()
  }).catch(() => {})
});
et("claim://result", t => {
  const e = t.payload || {};
  if (K && K.accountId === e.accountId && K.finish(e), e.ok === !1) {
    let n = e.message || a("m.unknownErr");
    e.code === 1005 && e.nextAt && (n += a("m.claimNextAt", {
      time: new Date(e.nextAt).toLocaleString(H(), {
        hour12: !1
      })
    })), d(a("m.claimFailed", {
      name: e.accountName,
      msg: n
    }), "err")
  } else {
    const n = [],
      s = e.serverTime || Date.now();
    e.startsAt && e.startsAt > s && n.push(a("m.claimStartsAt", {
      time: new Date(e.startsAt).toLocaleString(H(), {
        hour12: !1
      })
    })), e.endsAt && n.push(a("m.claimEndsAt", {
      time: new Date(e.endsAt).toLocaleString(H(), {
        hour12: !1
      })
    })), d(a("m.claimOk", {
      name: e.accountName,
      plan: e.planName
    }), "ok", n.join(a("common.listSep")))
  }
  e.accountId && (xt(e.accountId), C || re(e.accountId).then(() => {
    _()
  }), ht(e.accountId))
});
et("captcha://interactive", () => {
  if (!C || !K) return;
  const t = K.accountId;
  g("claim_cancel").catch(() => {}), X[t] = Date.now() + 60 * 60 * 1e3, d(a("m.autoClaimInteractive", {
    name: ue(t)
  }), "warn", a("m.autoClaimInteractiveDetail")), K.finish({
    ok: !1,
    code: "interactive"
  })
});
et("oauth://done", t => {
  const e = t.payload || {};
  if (e.ok === !1) {
    if (e.soft) {
      d(a("m.oauthSoft", {
        err: e.error || a("m.unknownErr")
      }), "warn", a("m.oauthSoftDetail"));
      return
    }
    d(a("m.oauthFail", {
      err: e.error || a("m.unknownErr")
    }), "err");
    return
  }
  if (e.duplicate) {
    d(a("m.oauthDup", {
      name: e.name
    }), "warn", a("m.oauthDupDetail"));
    return
  }
  d(a("m.oauthOk", {
    name: e.name
  }), "ok", a("m.oauthOkDetail")), T().then(() => {
    _(), V(), ct(e.id), W()
  }).catch(() => {})
});
et("state-changed", () => {
  T().then(() => {
    _()
  }).catch(() => {})
});
et("auto-switch-result", t => {
  const e = t.payload || {};
  if (e.status === "switched") {
    const n = [e.launchError && a("m.bitLaunchFailed", {
      err: e.launchError
    }), e.configStale && a("m.bitConfigStale")].filter(Boolean).join(a("common.listSep"));
    d(a("m.autoSwitched", {
      name: e.name
    }), e.launchError || e.configStale ? "warn" : "ok", n), T().then(() => {
      _(), V(), ct(p.active_account_id), W()
    }).catch(() => {})
  } else e.status === "no-candidate" ? d(a("m.autoSwitchNoCandidate"), "warn") : e.status === "error" && d(a("m.autoSwitchError", {
    err: e.error || a("m.unknownErr")
  }), "err")
});
const Me = 5 * 60 * 1e3,
  Ee = .2,
  qe = 8e3,
  Ie = 3;
let I = {};
const ut = new Set;

function ht(t, e = Date.now()) {
  const n = 1 + (Math.random() * 2 - 1) * Ee;
  I[t] = e + Math.round(Me * n)
}

function V() {
  const t = new Set((p?.accounts || []).map(e => e.id));
  for (const e of t) e in I || (I[e] = Date.now());
  for (const e of Object.keys(I)) t.has(e) || delete I[e]
}

function ct(t) {
  t && (I[t] = Date.now())
}

function W() {
  for (V(); ut.size < Ie;) {
    const t = Date.now(),
      e = (p?.accounts || []).find(s => (I[s.id] ?? 1 / 0) <= t && !ut.has(s.id) && !D[s.id]?.busy && !b[s.id]?.busy);
    if (!e) break;
    const n = I[e.id];
    ut.add(e.id), xt(e.id).finally(() => {
      ut.delete(e.id), I[e.id] === n && ht(e.id), W()
    })
  }
}(async () => {
  try {
    yt = await g("app_version").catch(() => ""), await T(), $(), await g("reveal_main"), setTimeout(Et, 350), V(), W(), Rt(), setInterval(() => {
      g("get_state").then(t => {
        const e = Pt !== JSON.stringify(t);
        p = t, t?.language && Ht(t.language), document.documentElement.dataset.theme = t?.theme || "dark", localStorage.setItem("zaccounts.theme", document.documentElement.dataset.theme), V(), e && _()
      }).catch(() => {})
    }, 5e3), setInterval(W, qe), setInterval(Rt, 3e4), setTimeout(Dt, ne), setInterval(Dt, ot)
  } catch (t) {
    U.innerHTML = `<div class="loading" style="color:var(--red)">${a("common.loadFail",{e:c(M(t))})}</div>`, g("reveal_main").catch(() => {}), Et()
  }
})();