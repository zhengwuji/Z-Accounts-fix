//! Backend-side i18n. Error keys match the original `err.*` / `claim.fail.*` /
//! `cli.*` catalogue so the frontend can translate or prefix-code them.

use serde_json::json;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub fn from_str(s: &str) -> Lang {
        match s {
            "en" => Lang::En,
            _ => Lang::Zh,
        }
    }
}

/// Translate an error key with `{{placeholder}}` substitutions.
pub fn t(lang: Lang, key: &str, args: &[(&str, &str)]) -> String {
    let zh = lang == Lang::Zh;
    let raw = match key {
        // store
        "err.write" => if zh { "写入失败：{e}" } else { "Write failed: {e}" },
        "err.write_file" => if zh { "写入失败 {path}: {e}" } else { "Write failed {path}: {e}" },
        "err.rename_fail" => if zh { "落盘失败 {path}: {e}" } else { "Persist failed {path}: {e}" },
        "err.read" => if zh { "无法读取 {path}: {e}" } else { "Cannot read {path}: {e}" },
        "err.bad_json" => if zh { "{path} 不是有效的 JSON：{e}" } else { "{path} is not valid JSON: {e}" },
        "err.mkdir" => if zh { "无法创建目录：{e}" } else { "Cannot create directory: {e}" },
        "err.serialize" => if zh { "序列化失败：{e}" } else { "Serialization failed: {e}" },
        "err.store.not_object" => if zh { "credentials.json 内容不是对象" } else { "credentials.json is not an object" },
        "err.store.list_fail" => if zh { "读取账号库失败：{e}" } else { "Failed to read the account store: {e}" },
        "err.store.bad_id" => "invalid account id",
        "err.store.no_account_id" => if zh { "账号不存在：{id}" } else { "Account not found: {id}" },
        "err.store.corrupt" => if zh { "账号存档损坏：{e}" } else { "Account archive corrupted: {e}" },
        "err.store.no_account" => if zh { "账号不存在" } else { "Account not found" },
        "err.store.delete_fail" => if zh { "删除失败：{e}" } else { "Delete failed: {e}" },
        "err.name.empty" => if zh { "名称不能为空" } else { "Name cannot be empty" },
        "err.name.too_long" => if zh { "名称过长（最多 40 字符）" } else { "Name too long (max 40 characters)" },
        "err.name.taken" => if zh { "名称「{name}」已被账号「{other}」占用" } else { "Name \"{name}\" is already used by account \"{other}\"" },
        // live / zcode
        "err.live.no_creds_file" => if zh { "当前没有 credentials.json，请先在 ZCode 里登录" } else { "No credentials.json present — log in inside ZCode first" },
        "err.live.no_credentials" => if zh { "当前文件里没有登录凭据（未登录）" } else { "Current file has no login credentials (not logged in)" },
        "err.live.dup_saved" => if zh { "当前登录已保存为「{name}」，无需重复保存" } else { "Current login is already saved as \"{name}\"" },
        "err.live.no_file" => if zh { "当前没有登录文件" } else { "No login file present" },
        "err.live.logged_out" => if zh { "当前未登录" } else { "Not logged in" },
        "err.live.same" => if zh { "当前登录与「{name}」一致，请直接切换" } else { "Current login already matches \"{name}\" — just switch to it" },
        "err.live.quota" => if zh { "当前未登录，无法查询额度" } else { "Not logged in — cannot query quota" },
        "err.switch.running" => if zh { "ZCode 正在运行，请先完全退出（含托盘），或使用强制切换（自动关闭并重启）" } else { "ZCode is running. Quit it fully (including the tray icon), or use Force switch (auto close & restart)" },
        "err.switch.kill_timeout" => if zh { "关闭 ZCode 超时，已取消切换（避免登录态损坏）" } else { "Timed out closing ZCode — switch cancelled to protect the login state" },
        "err.write_config" => if zh { "写入 config 失败：{e}" } else { "Failed to write config: {e}" },
        "err.hot.verify" => if zh { "热切换校验失败：ZCode 并发回写冲突，已重试 3 次。建议改用重启式切换" } else { "Hot-switch verification failed: ZCode wrote back concurrently (3 retries). Prefer restart-style switching" },
        "err.zcode.missing" => if zh { "ZCode 不存在：{path}（在设置里修改路径）" } else { "ZCode not found: {path} (change the path in Settings)" },
        "err.zcode.launch" => if zh { "启动失败：{e}" } else { "Launch failed: {e}" },
        "err.zcode.path_invalid" => if zh { "ZCode 路径无效：{p}" } else { "Invalid ZCode path: {p}" },
        "err.zcode.path_invalid_hint" => if zh { "ZCode 路径无效：{p}（在设置里修改）" } else { "Invalid ZCode path: {p} (change it in Settings)" },
        "err.zcode.kill_timeout" => if zh { "关闭 ZCode 超时" } else { "Timed out closing ZCode" },
        // crypto
        "err.cipher.pw_empty" => if zh { "密码不能为空（导出文件包含登录凭据，必须加密）" } else { "Password cannot be empty (exports contain login credentials and must be encrypted)" },
        "err.cipher.key" => if zh { "密钥错误：{e}" } else { "Key error: {e}" },
        "err.cipher.encrypt" => if zh { "加密失败：{e}" } else { "Encryption failed: {e}" },
        "err.cipher.no_kdf" => if zh { "文件缺少 kdf 段（不是有效的加密导出）" } else { "File is missing the kdf section (not a valid encrypted export)" },
        "err.cipher.no_cipher" => if zh { "文件缺少 cipher 段" } else { "File is missing the cipher section" },
        "err.cipher.bad_salt" => if zh { "salt 解码失败" } else { "salt decode failed" },
        "err.cipher.bad_nonce" => if zh { "nonce 解码失败" } else { "nonce decode failed" },
        "err.cipher.bad_tag" => if zh { "tag 解码失败" } else { "tag decode failed" },
        "err.cipher.bad_data" => if zh { "数据解码失败" } else { "data decode failed" },
        "err.cipher.nonce_len" => if zh { "nonce 长度异常" } else { "abnormal nonce length" },
        "err.cipher.wrong_pw" => if zh { "密码错误或文件已损坏" } else { "Wrong password or corrupted file" },
        "err.cipher.bad_plain" => if zh { "解密后内容异常：{e}" } else { "Decrypted payload is invalid: {e}" },
        // bundle import/export
        "err.bundle.no_accounts" => if zh { "捆绑包缺少 accounts 数组" } else { "Bundle is missing the accounts array" },
        "err.bundle.no_creds" => if zh { "捆绑包条目缺少 credentials" } else { "Bundle entry is missing credentials" },
        "err.bundle.unrecognized" => if zh { "无法识别的文件格式（仅支持本工具导出的加密捆绑包 .zsb）" } else { "Unrecognized file format (only encrypted .zsb bundles exported by this tool are supported)" },
        "err.import.no_creds" => if zh { "{fname}：无登录凭据" } else { "{fname}: no login credentials" },
        "err.import.wrap" => if zh { "{fname}：{e}" } else { "{fname}: {e}" },
        "err.import.dup" => if zh { "{fname}：已存在于账号库" } else { "{fname}: already in the account store" },
        "err.import.read" => if zh { "{fname}：读取失败 {e}" } else { "{fname}: read failed {e}" },
        "err.import.not_sealed" => if zh { "{fname}：不是加密捆绑包（仅支持本工具导出的 .zsb）" } else { "{fname}: not an encrypted bundle (only .zsb exported by this tool)" },
        "err.import.not_bundle" => if zh { "{fname}：不是捆绑包格式（仅支持导出全部生成的 .zsb）" } else { "{fname}: not a bundle (only .zsb generated by Export All)" },
        "err.import.not_sealed_plain" => if zh { "不是加密捆绑包（仅支持本工具导出的 .zsb）" } else { "Not an encrypted bundle (only .zsb exported by this tool)" },
        "err.import.not_bundle_plain" => if zh { "不是捆绑包格式（仅支持导出全部生成的 .zsb）" } else { "Not a bundle (only .zsb generated by Export All)" },
        "err.export.empty" => if zh { "账号库为空，没有可导出的内容" } else { "The account store is empty — nothing to export" },
        "err.export.empty_short" => if zh { "账号库为空" } else { "The account store is empty" },
        // misc
        "err.main.missing" => if zh { "主窗口不存在" } else { "Main window not found" },
        "err.path.invalid" => if zh { "路径无效：{e}" } else { "Invalid path: {e}" },
        "err.path.conv" => if zh { "路径错误：{e}" } else { "Path error: {e}" },
        "err.oauth.unknown_provider" => if zh { "未知登录提供方：{provider}" } else { "Unknown login provider: {provider}" },
        "err.proxy.invalid" => if zh { "代理地址无效：{e}" } else { "Invalid proxy address: {e}" },
        "err.oauth.appdata" => if zh { "无法定位应用数据目录：{e}" } else { "Cannot locate the app data directory: {e}" },
        "err.oauth.bad_authorize_url" => if zh { "authorize URL 非法：{e}" } else { "Invalid authorize URL: {e}" },
        "err.oauth.window" => if zh { "登录窗口创建失败：{e}" } else { "Failed to create the login window: {e}" },
        "err.proxy.need_url" => if zh { "开启代理前请先填写代理地址（http:// 或 socks5://）" } else { "Enter a proxy address (http:// or socks5://) before enabling the proxy" },
        "err.oauth.state" => if zh { "OAuth state 校验失败，请重新发起登录" } else { "OAuth state check failed — start the login again" },
        "err.oauth.flow" => if zh { "登录流程异常：{e}" } else { "Login flow error: {e}" },
        "err.oauth.not_callback" => if zh { "不是 OAuth 回调地址" } else { "Not an OAuth callback URL" },
        "err.oauth.bad_cb" => if zh { "回调参数格式错误" } else { "Malformed callback parameters" },
        "err.oauth.no_code_state" => if zh { "回调缺少 code 或 state" } else { "Callback is missing code or state" },
        "err.oauth.init" => if zh { "OAuth flow 初始化失败：{e}" } else { "OAuth flow init failed: {e}" },
        "err.oauth.init_invalid" => if zh { "官方 OAuth 初始化响应缺少必要字段，请稍后重试；也可在 ZCode 登录后使用「保存登录」" } else { "The official OAuth init response is missing required fields. Retry later, or sign in through ZCode and use Save login" },
        "err.oauth.init_http" => if zh { "官方 OAuth 初始化失败（HTTP {status}）：{msg}" } else { "Official OAuth init failed (HTTP {status}): {msg}" },
        "err.oauth.init_server_unknown" => if zh { "服务端未提供原因" } else { "no reason provided by the server" },
        "err.oauth.init_invalid_msg" => if zh { "OAuth flow 初始化失败：{msg}" } else { "OAuth flow init failed: {msg}" },
        "err.oauth.flow_failed" => if zh { "OAuth flow 授权失败" } else { "OAuth flow authorization failed" },
        "err.oauth.poll_invalid" => if zh { "OAuth flow 查询响应无效" } else { "Invalid OAuth flow poll response" },
        "err.oauth.poll_invalid_msg" => if zh { "OAuth flow 查询失败：{msg}" } else { "OAuth flow poll failed: {msg}" },
        "err.oauth.poll_terminal" => if zh { "OAuth flow 已失效（HTTP {code}）" } else { "OAuth flow invalidated (HTTP {code})" },
        "err.oauth.zai_business" => if zh { "z.ai 业务令牌换取失败，请重新登录" } else { "Failed to resolve the z.ai business token — log in again" },
        "err.oauth.expired" => if zh { "OAuth 登录流程已过期，请重新发起" } else { "The OAuth login flow expired — start it again" },
        "err.oauth.exchange_req" => if zh { "token 交换请求失败：{e}" } else { "Token exchange request failed: {e}" },
        "err.oauth.exchange" => if zh { "token 交换失败（{code}）：{msg}" } else { "Token exchange failed ({code}): {msg}" },
        "err.oauth.no_token" => if zh { "token 交换成功但响应缺少 token 字段" } else { "Token exchange succeeded but the response has no token field" },
        "err.proxy.empty" => if zh { "代理地址不能为空" } else { "Proxy address cannot be empty" },
        "err.proxy.scheme" => if zh { "地址需以 http:// 或 socks5:// 开头（如 http://127.0.0.1:7890）" } else { "Address must start with http:// or socks5:// (e.g. http://127.0.0.1:7890)" },
        "err.proxy.no_auth" => if zh { "代理不支持账号密码认证，请使用免认证的本地代理" } else { "Proxies with username/password auth are not supported — use a local proxy without auth" },
        "err.proxy.no_path" => if zh { "代理地址不包含路径，只需 scheme://主机:端口" } else { "Proxy address takes no path, only scheme://host:port" },
        "err.proxy.need_port" => if zh { "代理地址必须带端口（如 :7890）" } else { "Proxy address must include a port (e.g. :7890)" },
        "err.proxy.empty_host" => if zh { "代理主机不能为空" } else { "Proxy host cannot be empty" },
        "err.proxy.bad_host" => if zh { "代理主机格式不正确" } else { "Malformed proxy host" },
        "err.proxy.port_nan" => if zh { "端口“{port}”不是数字" } else { "Port \"{port}\" is not a number" },
        "err.proxy.port_range" => if zh { "端口 {port} 超出范围（1-65535）" } else { "Port {port} out of range (1-65535)" },
        "err.http.read" => if zh { "读取响应失败：{e}" } else { "Failed to read response: {e}" },
        "err.token.biz401" => if zh { "Token 已过期或无效（业务码 401）" } else { "Token expired or invalid (business code 401)" },
        "err.quota.rate_limited" => if zh { "服务端限流，正在重试..." } else { "Server rate limit, retrying..." },
        "err.quota.http429" => if zh { "额度接口 HTTP 429" } else { "Quota API HTTP 429" },
        "err.token.http401" => if zh { "Token 已过期或无效（HTTP {code}）" } else { "Token expired or invalid (HTTP {code})" },
        "err.quota.http" => if zh { "额度接口 HTTP {code}: {msg}" } else { "Quota API HTTP {code}: {msg}" },
        "err.network" => if zh { "网络请求失败：{e}" } else { "Network request failed: {e}" },
        "err.quota.biz" => if zh { "业务码 {code}: {msg}" } else { "Business code {code}: {msg}" },
        "err.quota.bad_resp" => if zh { "额度接口返回异常" } else { "Quota API returned an unexpected response" },
        "err.quota.fail" => if zh { "额度查询失败" } else { "Quota query failed" },
        "err.quota.no_token" => if zh { "未找到可用于查询额度的 ZCode token，请先登录或切换账号" } else { "No usable ZCode token found — log in or switch accounts first" },
        "err.token.expired" => if zh { "该账号 Token 已过期，请删除后重新登录" } else { "This account's token has expired — delete it and log in again" },
        // claim
        "err.claim.no_jwt" => if zh { "该账号缺少 zcodejwttoken 凭证，请先在 ZCode 客户端登录一次刷新" } else { "This account has no zcodejwttoken credential — log in once in the ZCode client to refresh it" },
        "err.claim.preview_req" => if zh { "preview 请求失败" } else { "Preview request failed" },
        "err.claim.claim_req" => if zh { "领取请求失败" } else { "Claim request failed" },
        "err.claim.no_captcha" => if zh { "验证码参数为空，请重试" } else { "Captcha parameter is empty — retry" },
        "err.claim.config_req" => if zh { "配置请求失败：{e}" } else { "Config request failed: {e}" },
        "err.claim.config_unavailable" => if zh { "验证码配置不可用" } else { "Captcha config unavailable" },
        "err.claim.gone" => if zh { "该套餐已不可领取，请刷新" } else { "This plan is no longer claimable — refresh the list" },
        "err.claim.none_pending" => if zh { "没有待领取的套餐" } else { "No pending plan to claim" },
        "err.claim.activate_req" => if zh { "激活上报失败：{e}" } else { "Activation report failed: {e}" },
        "claim.fail.1001" => if zh { "套餐不存在" } else { "Plan does not exist" },
        "claim.fail.1002" => if zh { "活动已结束或套餐暂不可领取" } else { "The event has ended or the plan is not claimable yet" },
        "claim.fail.1003" => if zh { "该套餐已经领取过" } else { "This plan has already been claimed" },
        "claim.fail.1004" => if zh { "不符合领取条件" } else { "Not eligible for this plan" },
        "claim.fail.1005" => if zh { "今日领取名额已用完" } else { "Today's claim quota is used up" },
        "claim.fail.3001" => if zh { "领取参数错误，请刷新后重试" } else { "Claim parameter error — refresh and retry" },
        "claim.fail.3007" => if zh { "验证码校验失败，请重试" } else { "Captcha verification failed — retry" },
        "claim.fail.401" => if zh { "请先登录后再领取" } else { "Log in before claiming" },
        "claim.fail.generic" => if zh { "领取失败" } else { "Claim failed" },
        "claim.fail.with_server" => if zh { "{base}（{server_msg}）" } else { "{base} ({server_msg})" },
        // cli
        "cli.pw_hint" => if zh { "缺少密码：用环境变量 ZSW_PASSWORD（推荐，不会出现在进程列表/命令历史）或 --password <密码>" } else { "Missing password: set the ZSW_PASSWORD env var (recommended — never appears in process lists or shell history) or pass --password <password>" },
        "cli.missing_cmd" => if zh { "缺少子命令：state|list|capture|rename|delete|update|switch|quota|claim-preview|kill|export|export-all|import|behavior|setpath|launch" } else { "Missing subcommand: state|list|capture|rename|delete|update|switch|quota|claim-preview|kill|export|export-all|import|behavior|setpath|launch" },
        "cli.usage.rename" => "用法：rename --id <id> --name <名称>",
        "cli.usage.delete" => "用法：delete --id <id>",
        "cli.usage.update" => "用法：update --id <id>",
        "cli.usage.switch" => "用法：switch --id <id> [--force] [--restart|--no-restart]",
        "cli.usage.export" => "用法：export --id <id> --out <file.zsb>（密码：ZSW_PASSWORD 或 --password）",
        "cli.usage.export_all" => "用法：export-all --out <file.zsb>（密码：ZSW_PASSWORD 或 --password）",
        "cli.usage.import" => "用法：import --file <file.zsb>（密码：ZSW_PASSWORD 或 --password）",
        "cli.usage.setpath" => "用法：setpath --path <ZCode.exe>",
        "cli.unknown_cmd" => if zh { "未知子命令：{cmd}" } else { "Unknown subcommand: {cmd}" },
        "cli.read_fail" => if zh { "读取失败：{e}" } else { "Read failed: {e}" },
        "cli.json_fail" => if zh { "JSON 解析失败：{e}" } else { "JSON parse failed: {e}" },
        "err.lang.unknown" => if zh { "未知语言：{lang}（支持 zh / en）" } else { "Unknown language: {lang} (supported: zh / en)" },
        "err.quota.http_gen" => if zh { "额度接口 HTTP {code}" } else { "Quota API HTTP {code}" },
        // tray / windows / dialogs
        "tray.show" => if zh { "显示主窗口" } else { "Show main window" },
        "tray.capture" => if zh { "保存当前登录" } else { "Save current login" },
        "tray.launch" => if zh { "启动 ZCode" } else { "Launch ZCode" },
        "tray.kill" => if zh { "关闭 ZCode" } else { "Quit ZCode" },
        "tray.quit" => if zh { "退出" } else { "Exit" },
        "tray.unsaved" => if zh { "未保存的登录" } else { "Unsaved login" },
        "tray.logged_out" => if zh { "未登录" } else { "Not logged in" },
        "title.login" => if zh { "登录 ZCode 账号" } else { "Sign in to ZCode" },
        "title.captcha" => if zh { "安全验证" } else { "Security verification" },
        "title.settings" => if zh { "Z-Accounts 设置" } else { "Z-Accounts Settings" },
        "dialog.zsb" => if zh { "ZSwitch 加密捆绑包（.zsb）" } else { "ZSwitch encrypted bundle (.zsb)" },
        "dialog.exe" => if zh { "ZCode 可执行文件" } else { "ZCode executable" },
        "m.status.running" => if zh { "ZCode 运行中" } else { "ZCode running" },
        "m.status.unsaved" => if zh { "未保存的登录" } else { "Unsaved login" },
        "m.status.safe" => if zh { "可安全切换" } else { "Safe to switch" },
        "m.status.logged_out" => if zh { "未登录" } else { "Not logged in" },
        "prov.bigmodel" => if zh { "BigModel（智谱开放平台）" } else { "BigModel (Zhipu Open Platform)" },
        "prov.zai" => if zh { "z.ai（国际站）" } else { "z.ai (Global)" },
        "prov.manual" => if zh { "在 ZCode 登录后保存当前账号（备用方式）" } else { "Sign in with ZCode, then save the login (fallback)" },
        _ => key,
    };
    let mut out = raw.to_string();
    for (k, v) in args {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

/// Convenience: localized error message. The backend messages are shown as-is
/// by the frontend; `wrong_password` keeps a code prefix because the password
/// modal detects it via `stripCode(err) === "wrong_password"`.
pub fn err(lang: Lang, key: &str, args: &[(&str, &str)]) -> serde_json::Value {
    if key == "err.cipher.wrong_pw" {
        return json!(format!("wrong_password: {}", t(lang, key, args)));
    }
    json!(t(lang, key, args))
}
