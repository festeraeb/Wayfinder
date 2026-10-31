//! Self-update: a signed-update check (Tauri's updater plugin) on start and on a timer, plus a small
//! authenticated "push" endpoint so the operator can tell a running Wayfinder to update now.
//!
//! Privacy: the manifest is a static file (`plugins.updater.endpoints` in tauri.conf.json has no
//! `{{...}}` variables), so the check sends no version, platform, id or text. The app compares the
//! version itself. Nothing is sent from here to anywhere else.
//!
//! Push API (loopback and the tailnet only, never 0.0.0.0), `Authorization: Bearer <update_token>`:
//!   POST /api/update/check-now[?install=0]   check; if newer, start the install (the app then restarts)
//!   GET  /api/update/status                  current version, last check, update waiting
//! The token and the settings live in `<config dir>/updater_config.json` (created on first run).

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const DEFAULT_PUSH_PORT: u16 = 14322;
pub const DEFAULT_CHECK_MINUTES: u64 = 360;
const MIN_CHECK_MINUTES: u64 = 5;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdaterConfig {
    /// Bearer token the push endpoint requires. Generated on first run.
    pub update_token: String,
    /// Minutes between background checks.
    #[serde(default = "default_minutes")]
    pub check_interval_minutes: u64,
    /// Install a found update from the timer without being pushed. Off by default: a person in the middle
    /// of work is never restarted by a background check; the push endpoint installs.
    #[serde(default)]
    pub auto_install: bool,
    #[serde(default = "default_port")]
    pub push_port: u16,
    /// Also listen on this machine's tailnet address (100.64.0.0/10). Loopback is always on.
    #[serde(default = "default_true")]
    pub bind_tailnet: bool,
    /// Master switch for the background check and the push endpoint.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_minutes() -> u64 { DEFAULT_CHECK_MINUTES }
fn default_port() -> u16 { DEFAULT_PUSH_PORT }
fn default_true() -> bool { true }

// The token must never reach logs or panics.
impl std::fmt::Debug for UpdaterConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdaterConfig")
            .field("update_token", &"***")
            .field("check_interval_minutes", &self.check_interval_minutes)
            .field("auto_install", &self.auto_install)
            .field("push_port", &self.push_port)
            .field("bind_tailnet", &self.bind_tailnet)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl UpdaterConfig {
    pub fn with_token(token: String) -> Self {
        Self {
            update_token: token,
            check_interval_minutes: DEFAULT_CHECK_MINUTES,
            auto_install: false,
            push_port: DEFAULT_PUSH_PORT,
            bind_tailnet: true,
            enabled: true,
        }
    }

    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.check_interval_minutes.max(MIN_CHECK_MINUTES) * 60)
    }
}

/// 32 random bytes as hex.
pub fn new_token() -> String {
    use rand::RngCore;
    let mut b = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Load the config, or create it (with a fresh token) when missing. An unreadable file is replaced
/// only if it is empty; a malformed one is left alone and an error returned, so a person's edit is never lost.
pub fn load_or_create(dir: &Path) -> Result<UpdaterConfig, String> {
    let path = dir.join("updater_config.json");
    match std::fs::read_to_string(&path) {
        Ok(text) if !text.trim().is_empty() => {
            let mut cfg: UpdaterConfig =
                serde_json::from_str(&text).map_err(|e| format!("updater_config.json is not valid: {e}"))?;
            if cfg.update_token.trim().len() < 16 {
                cfg.update_token = new_token();
                write_config(&path, &cfg)?;
            }
            Ok(cfg)
        }
        _ => {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            let cfg = UpdaterConfig::with_token(new_token());
            write_config(&path, &cfg)?;
            Ok(cfg)
        }
    }
}

fn write_config(path: &Path, cfg: &UpdaterConfig) -> Result<(), String> {
    let text = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// The Wayfinder config directory (same place as lemonade_config.json).
pub fn config_dir() -> Option<PathBuf> {
    if let Ok(a) = std::env::var("APPDATA") {
        if !a.trim().is_empty() {
            return Some(Path::new(a.trim()).join("wayfinder"));
        }
    }
    if let Ok(h) = std::env::var("HOME") {
        if !h.trim().is_empty() {
            return Some(Path::new(h.trim()).join(".config").join("wayfinder"));
        }
    }
    None
}

/// Tailnet (CGNAT) range 100.64.0.0/10, which Tailscale assigns.
pub fn is_tailnet_ip(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    o[0] == 100 && (64..=127).contains(&o[1])
}

/// Peers allowed to talk to the endpoint: this machine and the tailnet.
pub fn peer_allowed(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || is_tailnet_ip(v4),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

/// Addresses to listen on: loopback, plus each tailnet address of this machine when enabled.
/// Never a wildcard address.
pub fn bind_addrs(local_ips: &[IpAddr], cfg: &UpdaterConfig) -> Vec<SocketAddr> {
    let mut v = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), cfg.push_port)];
    if cfg.bind_tailnet {
        for ip in local_ips {
            if let IpAddr::V4(v4) = ip {
                if is_tailnet_ip(*v4) {
                    let a = SocketAddr::new(*ip, cfg.push_port);
                    if !v.contains(&a) {
                        v.push(a);
                    }
                }
            }
        }
    }
    v
}

fn local_ips() -> Vec<IpAddr> {
    if_addrs::get_if_addrs().map(|v| v.into_iter().map(|i| i.ip()).collect()).unwrap_or_default()
}

/// Constant-time token comparison.
pub fn token_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        diff |= a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0);
    }
    diff == 0
}

/// `Authorization: Bearer <token>` check.
pub fn authorised(auth_header: Option<&str>, token: &str) -> bool {
    if token.trim().len() < 16 {
        return false;
    }
    match auth_header.map(str::trim).and_then(|h| h.split_once(' ')) {
        Some((scheme, rest)) if scheme.eq_ignore_ascii_case("bearer") => token_eq(rest.trim(), token),
        _ => false,
    }
}

// ---------------------------------------------------------------------------------------------
// Request handling, independent of Tauri so it can be tested.

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpdateInfo {
    pub version: String,
}

pub type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// What the endpoint needs from the app.
pub trait UpdateService: Send + Sync {
    fn current_version(&self) -> String;
    /// Ask the update channel whether a newer, correctly signed version exists.
    fn check(&self) -> BoxFut<Result<Option<UpdateInfo>, String>>;
    /// Download and install the update. On Windows the app exits and the installer relaunches it.
    fn install(&self) -> BoxFut<Result<(), String>>;
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Status {
    pub last_check_unix: Option<u64>,
    pub last_error: Option<String>,
    pub update_available: Option<String>,
    pub installing: bool,
}

pub struct Shared {
    pub status: Mutex<Status>,
}

fn now_unix() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn record_check(shared: &Shared, r: &Result<Option<UpdateInfo>, String>) {
    if let Ok(mut s) = shared.status.lock() {
        s.last_check_unix = Some(now_unix());
        match r {
            Ok(u) => {
                s.last_error = None;
                s.update_available = u.as_ref().map(|u| u.version.clone());
            }
            Err(e) => s.last_error = Some(e.clone()),
        }
    }
}

pub struct Req<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub authorization: Option<&'a str>,
}

fn reply(status: u16, v: serde_json::Value) -> (u16, String) {
    (status, v.to_string())
}

/// Route one request. `install` in the response means an install was started, and the app is about to restart.
pub async fn route(req: &Req<'_>, token: &str, svc: Arc<dyn UpdateService>, shared: &Shared) -> (u16, String) {
    let (path, query) = req.target.split_once('?').unwrap_or((req.target, ""));
    if !authorised(req.authorization, token) {
        return reply(401, serde_json::json!({"error": "unauthorized"}));
    }
    match (req.method, path) {
        ("POST", "/api/update/check-now") => {
            let install = !query.split('&').any(|p| p == "install=0" || p == "install=false");
            let r = svc.check().await;
            record_check(shared, &r);
            match r {
                Err(e) => reply(502, serde_json::json!({"ok": false, "current_version": svc.current_version(), "error": e})),
                Ok(None) => reply(200, serde_json::json!({
                    "ok": true, "current_version": svc.current_version(), "update_available": false, "installing": false
                })),
                Ok(Some(u)) => {
                    let started = install && {
                        let mut s = shared.status.lock().unwrap();
                        let first = !s.installing;
                        s.installing = true;
                        first
                    };
                    if started {
                        let svc2 = svc.clone();
                        tokio::spawn(async move {
                            // Let the response reach the caller before the app exits.
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            if let Err(e) = svc2.install().await {
                                eprintln!("[updater] install failed: {e}");
                            }
                        });
                    }
                    reply(200, serde_json::json!({
                        "ok": true, "current_version": svc.current_version(), "update_available": true,
                        "version": u.version, "installing": started
                    }))
                }
            }
        }
        ("GET", "/api/update/status") => {
            let s = shared.status.lock().unwrap().clone();
            reply(200, serde_json::json!({"current_version": svc.current_version(), "status": s}))
        }
        (_, "/api/update/check-now") | (_, "/api/update/status") => reply(405, serde_json::json!({"error": "method not allowed"})),
        _ => reply(404, serde_json::json!({"error": "not found"})),
    }
}

fn parse_head(head: &str) -> Option<(String, String, Option<String>)> {
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split_whitespace();
    let method = first.next()?.to_string();
    let target = first.next()?.to_string();
    let auth = lines.find_map(|l| {
        let (k, v) = l.split_once(':')?;
        k.trim().eq_ignore_ascii_case("authorization").then(|| v.trim().to_string())
    });
    Some((method, target, auth))
}

async fn serve_conn(mut sock: TcpStream, token: Arc<String>, svc: Arc<dyn UpdateService>, shared: Arc<Shared>) {
    match sock.peer_addr() {
        Ok(p) if peer_allowed(p.ip()) => {}
        _ => return,
    }
    let mut buf = Vec::new();
    let mut tmp = [0u8; 2048];
    let head_end = loop {
        match tokio::time::timeout(Duration::from_secs(5), sock.read(&mut tmp)).await {
            Ok(Ok(n)) if n > 0 => buf.extend_from_slice(&tmp[..n]),
            _ => return,
        }
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > 16 * 1024 {
            return;
        }
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).to_string();
    let (status, body) = match parse_head(&head) {
        Some((m, t, a)) => route(&Req { method: &m, target: &t, authorization: a.as_deref() }, &token, svc, &shared).await,
        None => (400, serde_json::json!({"error": "bad request"}).to_string()),
    };
    let phrase = match status { 200 => "OK", 400 => "Bad Request", 401 => "Unauthorized", 404 => "Not Found", 405 => "Method Not Allowed", 502 => "Bad Gateway", _ => "OK" };
    let out = format!(
        "HTTP/1.1 {status} {phrase}\r\nContent-Type: application/json\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = sock.write_all(out.as_bytes()).await;
    let _ = sock.shutdown().await;
}

/// Listen on every address in `addrs`. Addresses that fail to bind are skipped (e.g. a tailnet
/// address that went away); returns how many are listening.
pub async fn serve(addrs: Vec<SocketAddr>, token: Arc<String>, svc: Arc<dyn UpdateService>, shared: Arc<Shared>) -> usize {
    let mut n = 0;
    for a in addrs {
        match TcpListener::bind(a).await {
            Ok(l) => {
                n += 1;
                let (token, svc, shared) = (token.clone(), svc.clone(), shared.clone());
                tokio::spawn(async move {
                    loop {
                        let Ok((sock, _)) = l.accept().await else { continue };
                        tokio::spawn(serve_conn(sock, token.clone(), svc.clone(), shared.clone()));
                    }
                });
            }
            Err(e) => eprintln!("[updater] cannot listen on {a}: {e}"),
        }
    }
    n
}

/// Background check: shortly after start, then every `interval`.
pub async fn timer_loop(cfg: UpdaterConfig, svc: Arc<dyn UpdateService>, shared: Arc<Shared>) {
    tokio::time::sleep(Duration::from_secs(20)).await;
    loop {
        let r = svc.check().await;
        record_check(&shared, &r);
        if let Ok(Some(u)) = &r {
            eprintln!("[updater] update {} is available", u.version);
            if cfg.auto_install {
                let first = {
                    let mut s = shared.status.lock().unwrap();
                    let f = !s.installing;
                    s.installing = true;
                    f
                };
                if first {
                    if let Err(e) = svc.install().await {
                        eprintln!("[updater] install failed: {e}");
                        shared.status.lock().unwrap().installing = false;
                    }
                }
            }
        }
        tokio::time::sleep(cfg.interval()).await;
    }
}

// ---------------------------------------------------------------------------------------------
// The real service, on Tauri's updater plugin.

pub struct TauriUpdates<R: tauri::Runtime> {
    pub app: tauri::AppHandle<R>,
}

impl<R: tauri::Runtime> UpdateService for TauriUpdates<R> {
    fn current_version(&self) -> String {
        self.app.package_info().version.to_string()
    }

    fn check(&self) -> BoxFut<Result<Option<UpdateInfo>, String>> {
        use tauri_plugin_updater::UpdaterExt;
        let app = self.app.clone();
        Box::pin(async move {
            let updater = app.updater().map_err(|e| e.to_string())?;
            let u = updater.check().await.map_err(|e| e.to_string())?;
            Ok(u.map(|u| UpdateInfo { version: u.version.clone() }))
        })
    }

    fn install(&self) -> BoxFut<Result<(), String>> {
        use tauri_plugin_updater::UpdaterExt;
        let app = self.app.clone();
        Box::pin(async move {
            let updater = app.updater().map_err(|e| e.to_string())?;
            let Some(u) = updater.check().await.map_err(|e| e.to_string())? else { return Ok(()) };
            u.download_and_install(|_, _| {}, || {}).await.map_err(|e| e.to_string())?;
            // Non-Windows platforms return here; Windows exits inside the call and the installer relaunches the app.
            app.restart();
        })
    }
}

/// Start the timer and the push endpoint. Called once from the app's setup.
pub fn start<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
    let Some(dir) = config_dir() else { return };
    let cfg = match load_or_create(&dir) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[updater] disabled: {e}");
            return;
        }
    };
    if !cfg.enabled {
        return;
    }
    let svc: Arc<dyn UpdateService> = Arc::new(TauriUpdates { app });
    let shared = Arc::new(Shared { status: Mutex::new(Status::default()) });
    let addrs = bind_addrs(&local_ips(), &cfg);
    let token = Arc::new(cfg.update_token.clone());
    let (svc2, shared2, cfg2) = (svc.clone(), shared.clone(), cfg.clone());
    tauri::async_runtime::spawn(async move {
        let n = serve(addrs, token, svc2, shared2).await;
        eprintln!("[updater] push endpoint listening on {n} address(es)");
    });
    tauri::async_runtime::spawn(timer_loop(cfg2, svc, shared));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const TOKEN: &str = "0123456789abcdef0123456789abcdef";

    struct Fake {
        newer: Option<String>,
        installs: AtomicUsize,
        fail: bool,
    }
    impl UpdateService for Fake {
        fn current_version(&self) -> String { "2.1.0".into() }
        fn check(&self) -> BoxFut<Result<Option<UpdateInfo>, String>> {
            let (n, f) = (self.newer.clone(), self.fail);
            Box::pin(async move { if f { Err("manifest unreachable".into()) } else { Ok(n.map(|version| UpdateInfo { version })) } })
        }
        fn install(&self) -> BoxFut<Result<(), String>> {
            self.installs.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(()) })
        }
    }
    fn fake(newer: Option<&str>, fail: bool) -> Arc<Fake> {
        Arc::new(Fake { newer: newer.map(String::from), installs: AtomicUsize::new(0), fail })
    }
    fn shared() -> Shared { Shared { status: Mutex::new(Status::default()) } }
    fn bearer() -> String { format!("Bearer {TOKEN}") }

    #[test]
    fn tailnet_range_is_cgnat_only() {
        assert!(is_tailnet_ip("100.76.125.86".parse().unwrap()));
        assert!(is_tailnet_ip("100.64.0.1".parse().unwrap()));
        assert!(is_tailnet_ip("100.127.255.254".parse().unwrap()));
        assert!(!is_tailnet_ip("100.63.0.1".parse().unwrap()));
        assert!(!is_tailnet_ip("100.128.0.1".parse().unwrap()));
        assert!(!is_tailnet_ip("192.168.4.27".parse().unwrap()));
        assert!(!is_tailnet_ip("10.0.0.5".parse().unwrap()));
    }

    #[test]
    fn peers_are_loopback_or_tailnet() {
        assert!(peer_allowed("127.0.0.1".parse().unwrap()));
        assert!(peer_allowed("::1".parse().unwrap()));
        assert!(peer_allowed("100.103.38.29".parse().unwrap()));
        assert!(!peer_allowed("192.168.4.1".parse().unwrap()));
        assert!(!peer_allowed("8.8.8.8".parse().unwrap()));
        assert!(!peer_allowed("fe80::1".parse().unwrap()));
    }

    #[test]
    fn binds_loopback_and_tailnet_never_wildcard() {
        let cfg = UpdaterConfig::with_token(TOKEN.into());
        let ips: Vec<IpAddr> = ["192.168.4.27", "100.76.125.86", "127.0.0.1", "0.0.0.0"].iter().map(|s| s.parse().unwrap()).collect();
        let a = bind_addrs(&ips, &cfg);
        assert_eq!(a, vec!["127.0.0.1:14322".parse().unwrap(), "100.76.125.86:14322".parse().unwrap()]);
        assert!(a.iter().all(|s| !s.ip().is_unspecified()));
        let off = UpdaterConfig { bind_tailnet: false, ..cfg };
        assert_eq!(bind_addrs(&ips, &off), vec!["127.0.0.1:14322".parse::<SocketAddr>().unwrap()]);
    }

    #[test]
    fn token_checks() {
        assert!(authorised(Some(&bearer()), TOKEN));
        assert!(authorised(Some(&format!("bearer   {TOKEN}")), TOKEN));
        assert!(!authorised(None, TOKEN));
        assert!(!authorised(Some("Bearer nope"), TOKEN));
        assert!(!authorised(Some(TOKEN), TOKEN));
        assert!(!authorised(Some("Bearer "), TOKEN));
        // a short or empty configured token never authorises, even when presented
        assert!(!authorised(Some("Bearer short"), "short"));
        assert!(!authorised(Some("Bearer "), ""));
        assert!(token_eq("a", "a") && !token_eq("a", "ab") && !token_eq("ab", "ac"));
    }

    #[test]
    fn config_is_created_with_a_token_and_survives_reload() {
        let dir = std::env::temp_dir().join(format!("wf-upd-{}", new_token()));
        let a = load_or_create(&dir).unwrap();
        assert_eq!(a.update_token.len(), 64);
        assert!(!a.auto_install && a.bind_tailnet && a.enabled);
        let b = load_or_create(&dir).unwrap();
        assert_eq!(a, b);
        assert!(!format!("{a:?}").contains(&a.update_token));
        // a malformed file is reported, not overwritten
        std::fs::write(dir.join("updater_config.json"), "{ not json").unwrap();
        assert!(load_or_create(&dir).is_err());
        assert_eq!(std::fs::read_to_string(dir.join("updater_config.json")).unwrap(), "{ not json");
        // missing optional fields get defaults
        std::fs::write(dir.join("updater_config.json"), format!("{{\"update_token\":\"{TOKEN}\"}}")).unwrap();
        let c = load_or_create(&dir).unwrap();
        assert_eq!((c.push_port, c.check_interval_minutes, c.auto_install), (DEFAULT_PUSH_PORT, DEFAULT_CHECK_MINUTES, false));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn interval_has_a_floor() {
        let mut c = UpdaterConfig::with_token(TOKEN.into());
        c.check_interval_minutes = 0;
        assert_eq!(c.interval(), Duration::from_secs(300));
    }

    #[tokio::test]
    async fn unauthenticated_requests_do_nothing() {
        let f = fake(Some("2.2.0"), false);
        let sh = shared();
        let r = route(&Req { method: "POST", target: "/api/update/check-now", authorization: None }, TOKEN, f.clone(), &sh).await;
        assert_eq!(r.0, 401);
        let r = route(&Req { method: "POST", target: "/api/update/check-now", authorization: Some("Bearer wrong") }, TOKEN, f.clone(), &sh).await;
        assert_eq!(r.0, 401);
        tokio::time::sleep(Duration::from_millis(700)).await;
        assert_eq!(f.installs.load(Ordering::SeqCst), 0);
        assert!(sh.status.lock().unwrap().last_check_unix.is_none());
    }

    #[tokio::test]
    async fn up_to_date_reports_and_does_not_install() {
        let f = fake(None, false);
        let sh = shared();
        let a = bearer();
        let (s, body) = route(&Req { method: "POST", target: "/api/update/check-now", authorization: Some(&a) }, TOKEN, f.clone(), &sh).await;
        assert_eq!(s, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["update_available"], false);
        assert_eq!(v["current_version"], "2.1.0");
        assert_eq!(f.installs.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn newer_version_starts_one_install() {
        let f = fake(Some("2.2.0"), false);
        let sh = shared();
        let a = bearer();
        let req = Req { method: "POST", target: "/api/update/check-now", authorization: Some(&a) };
        let (s, body) = route(&req, TOKEN, f.clone(), &sh).await;
        assert_eq!(s, 200);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!((v["update_available"].clone(), v["version"].clone(), v["installing"].clone()), (true.into(), "2.2.0".into(), true.into()));
        // a second push while installing does not start another
        let (_, body2) = route(&req, TOKEN, f.clone(), &sh).await;
        assert_eq!(serde_json::from_str::<serde_json::Value>(&body2).unwrap()["installing"], false);
        tokio::time::sleep(Duration::from_millis(900)).await;
        assert_eq!(f.installs.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn install_zero_only_checks() {
        let f = fake(Some("2.2.0"), false);
        let sh = shared();
        let a = bearer();
        let (_, body) = route(&Req { method: "POST", target: "/api/update/check-now?install=0", authorization: Some(&a) }, TOKEN, f.clone(), &sh).await;
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!((v["update_available"].clone(), v["installing"].clone()), (true.into(), false.into()));
        tokio::time::sleep(Duration::from_millis(700)).await;
        assert_eq!(f.installs.load(Ordering::SeqCst), 0);
        assert_eq!(sh.status.lock().unwrap().update_available.as_deref(), Some("2.2.0"));
    }

    #[tokio::test]
    async fn channel_failure_is_a_502_with_the_reason() {
        let f = fake(None, true);
        let sh = shared();
        let a = bearer();
        let (s, body) = route(&Req { method: "POST", target: "/api/update/check-now", authorization: Some(&a) }, TOKEN, f, &sh).await;
        assert_eq!(s, 502);
        assert!(body.contains("manifest unreachable"));
        assert_eq!(sh.status.lock().unwrap().last_error.as_deref(), Some("manifest unreachable"));
    }

    #[tokio::test]
    async fn status_and_routing() {
        let f = fake(None, false);
        let sh = shared();
        let a = bearer();
        let (s, body) = route(&Req { method: "GET", target: "/api/update/status", authorization: Some(&a) }, TOKEN, f.clone(), &sh).await;
        assert_eq!(s, 200);
        assert!(body.contains("\"current_version\":\"2.1.0\""));
        assert_eq!(route(&Req { method: "GET", target: "/api/update/check-now", authorization: Some(&a) }, TOKEN, f.clone(), &sh).await.0, 405);
        assert_eq!(route(&Req { method: "GET", target: "/api/other", authorization: Some(&a) }, TOKEN, f, &sh).await.0, 404);
    }

    #[tokio::test]
    async fn real_socket_round_trip_on_loopback() {
        let f = fake(None, false);
        let sh = Arc::new(shared());
        // find a free port
        let port = { let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap(); l.local_addr().unwrap().port() };
        let n = serve(vec![SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port)], Arc::new(TOKEN.to_string()), f, sh).await;
        assert_eq!(n, 1);
        async fn call(port: u16, auth: Option<&str>) -> String {
            let mut s = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
            let a = auth.map(|a| format!("Authorization: {a}\r\n")).unwrap_or_default();
            s.write_all(format!("POST /api/update/check-now HTTP/1.1\r\nHost: x\r\n{a}Content-Length: 0\r\n\r\n").as_bytes()).await.unwrap();
            let mut out = String::new();
            s.read_to_string(&mut out).await.unwrap();
            out
        }
        assert!(call(port, None).await.starts_with("HTTP/1.1 401"));
        let ok = call(port, Some(&bearer())).await;
        assert!(ok.starts_with("HTTP/1.1 200") && ok.contains("\"update_available\":false"));
    }

    /// The shipped config: static manifest on our site, a public key, no per-request variables.
    #[test]
    fn shipped_updater_config_is_private_and_signed() {
        let text = include_str!("../tauri.conf.json");
        let v: serde_json::Value = serde_json::from_str(text).unwrap();
        let u = &v["plugins"]["updater"];
        let eps = u["endpoints"].as_array().expect("endpoints");
        assert!(!eps.is_empty());
        for e in eps {
            let e = e.as_str().unwrap();
            assert!(e.starts_with("https://"), "{e}");
            assert!(!e.contains("{{") && !e.contains('?'), "update URL must not carry version/platform/ids: {e}");
        }
        assert!(u["pubkey"].as_str().unwrap().len() > 40, "public key missing");
        assert_eq!(u["windows"]["installMode"], "passive");
        assert!(!text.contains("PRIVATE"), "no private key material in config");
    }
}
