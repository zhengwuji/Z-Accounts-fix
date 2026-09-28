import {
  t as c,
  i as l,
  a as g,
  s as u,
  l as h,
  e as y
} from "./i18n.js";
const E = "https://o.alicdn.com/captcha-frontend/aliyunCaptcha/AliyunCaptcha.js",
  C = 8e3,
  b = document.getElementById("cap-text"),
  v = document.getElementById("cap-detail"),
  k = document.getElementById("cap-dot"),
  s = document.getElementById("cap-btn");

function n(e, a = "run") {
  b.textContent = e, k.className = "cap-dot" + (a === "ok" ? " ok" : a === "err" ? " err" : "")
}

function r(e) {
  v.textContent = e || ""
}
document.addEventListener("securitypolicyviolation", e => {
  r(c("c.cspBlocked", {
    directive: e.violatedDirective,
    uri: String(e.blockedURI).slice(0, 70)
  }))
});
const o = () => {
  y("captcha://interactive").catch(() => {})
};

function S() {
  return new Promise((e, a) => {
    if (typeof window.initAliyunCaptcha == "function") return e();
    const i = document.createElement("script");
    i.src = E, i.onload = () => e(), i.onerror = () => a(new Error(c("c.sdkFail"))), document.head.appendChild(i)
  })
}
let f = !1,
  p = null,
  d = 0;
async function T() {
  try {
    const t = await l("get_state");
    t?.language && g(t.language)
  } catch {}
  document.title = c("c.title"), s.textContent = c("c.btn"), document.querySelector(".cap-foot").textContent = c("c.foot"), n(c("c.preparing"));
  let e;
  try {
    e = await l("claim_captcha_config")
  } catch (t) {
    o(), n(c("c.cfgFail"), "err"), r(u(t));
    return
  }
  if (!e.enabled || !e.scene_id) {
    o(), n(c("c.cfgUnavailable"), "err"), r(c("c.cfgUnavailableDetail"));
    return
  }
  p = e.region || null;
  try {
    await S()
  } catch (t) {
    o(), n(t.message || c("c.sdkFail"), "err");
    return
  }
  window.AliyunCaptchaConfig = {
    region: e.region,
    prefix: e.prefix
  }, n(c("c.traceless"));
  const a = t => {
      f || !t || !t.trim() || (f = !0, clearTimeout(d), n(c("c.passed")), l("claim_captcha_submit", {
        param: t,
        region: p
      }).catch(m => {
        n(c("c.claimReqFail"), "err"), r(u(m))
      }))
    },
    i = t => {
      o(), clearTimeout(d), n(c("c.interactive")), s.hidden = !1, s.focus(), t && r(typeof t == "string" ? t.slice(0, 120) : JSON.stringify(t).slice(0, 120))
    };
  try {
    window.initAliyunCaptcha({
      SceneId: e.scene_id,
      mode: "popup",
      language: h() === "en" ? "en" : "zh-CN",
      showErrorTip: !1,
      element: "#cap-holder",
      button: "#cap-btn",
      getInstance: t => {
        typeof t.startTracelessVerification == "function" ? (t.startTracelessVerification(), d = setTimeout(i, C)) : i()
      },
      success: t => a(typeof t == "string" ? t : t?.captchaVerifyParam),
      fail: t => i(t),
      onError: t => i(t)
    })
  } catch (t) {
    o(), n(c("c.initFail"), "err"), r(String(t))
  }
}
s.addEventListener("click", () => {
  s.hidden || n(c("c.inPopup"))
});
T();