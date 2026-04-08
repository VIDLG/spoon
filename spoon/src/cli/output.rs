use std::io::IsTerminal;
use std::sync::{Mutex, OnceLock};

use crossterm::style::{Color, Stylize};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use serde::Serialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::{LinesWithEndings, as_24_bit_terminal_escaped};

use crate::cli::ColorModeArg;
use crate::cli::response::{CliEntry, CliKind, CliResponse};
use crate::bridge::StreamChunk;

const CLI_SECTION_HEADERS: &[&str] = &[
    "config:",
    "MSVC runtimes:",
    "Managed:",
    "Official:",
    "C++ validation:",
    "Rust validation:",
    "policy:",
    "Package:",
    "Install:",
    "Integration:",
];

const CLI_SUBSECTION_HEADERS: &[&str] = &[
    "Root:",
    "Runtime:",
    "Packages:",
    "Python:",
    "Git:",
    "MSVC:",
    "Derived:",
    "Desired:",
    "Detected native config:",
    "Config files:",
    "Conflicts:",
    "python:",
    "git:",
    "Integration:",
    "Policy:",
    "Commands:",
    "Environment:",
    "System:",
];

const CLI_INFO_PREFIXES: &[&str] = &[
    "Starting Scoop bucket sync ",
    "Fetching bucket contents from ",
    "Resolved Scoop package ",
    "Resolved bucket branch:",
    "Using ",
];

const CLI_SUCCESS_PREFIXES: &[&str] = &[
    "Registered bucket ",
    "Updated bucket ",
    "Removed bucket ",
    "Installed Scoop package ",
    "Removed Scoop package ",
    "Created shortcuts:",
    "Ran managed validation sample successfully",
    "Compiled managed C++/Win32 validation sample successfully",
    "Completed Scoop bucket sync ",
    "Fetched bucket contents from ",
    "Synced local bucket contents from ",
    "Compiled ",
    "Ran ",
];

const CLI_WARNING_PREFIXES: &[&str] = &["Skipped ", "Warning:", "Path mismatch:"];
const CLI_ERROR_TOKENS: &[&str] = &[
    "Error:",
    "Cancelled by user.",
    "failed",
    "requires a configured root",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticToken {
    Section,
    Subsection,
    Info,
    Success,
    Warning,
    Error,
    Link,
    SearchPackage,
    SearchVersion,
    SearchBucket,
    SearchDescription,
    InstalledYes,
    InstalledNo,
    Bullet,
    Progress,
    Prompt,
}

fn color_mode_slot() -> &'static Mutex<ColorMode> {
    static MODE: OnceLock<Mutex<ColorMode>> = OnceLock::new();
    MODE.get_or_init(|| Mutex::new(ColorMode::Auto))
}

pub fn set_color_mode(mode: ColorModeArg) {
    let mapped = match mode {
        ColorModeArg::Auto => ColorMode::Auto,
        ColorModeArg::Always => ColorMode::Always,
        ColorModeArg::Never => ColorMode::Never,
    };
    *color_mode_slot().lock().expect("color mode poisoned") = mapped;
}

fn use_color() -> bool {
    match *color_mode_slot().lock().expect("color mode poisoned") {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none()
        }
    }
}

fn render_token(text: &str, token: SemanticToken) -> String {
    match token {
        SemanticToken::Section => text.with(Color::Blue).bold().to_string(),
        SemanticToken::Subsection => text.with(Color::DarkCyan).bold().to_string(),
        SemanticToken::Info => text.with(Color::Cyan).bold().to_string(),
        SemanticToken::Success => text.with(Color::Green).bold().to_string(),
        SemanticToken::Warning => text.with(Color::Yellow).bold().to_string(),
        SemanticToken::Error => text.with(Color::Red).bold().to_string(),
        SemanticToken::Link => text.with(Color::Cyan).underlined().to_string(),
        SemanticToken::SearchPackage => text.with(Color::Cyan).bold().to_string(),
        SemanticToken::SearchVersion => text.with(Color::Green).to_string(),
        SemanticToken::SearchBucket => text.with(Color::Magenta).to_string(),
        SemanticToken::SearchDescription => text.with(Color::DarkGrey).to_string(),
        SemanticToken::InstalledYes => text.with(Color::Green).bold().to_string(),
        SemanticToken::InstalledNo => text.with(Color::Yellow).bold().to_string(),
        SemanticToken::Bullet => text.with(Color::DarkGrey).to_string(),
        SemanticToken::Progress => text.with(Color::Blue).bold().to_string(),
        SemanticToken::Prompt => text.with(Color::Cyan).to_string(),
    }
}

pub fn print_line(line: &str) {
    finish_progress_bar();
    if use_color() {
        println!("{}", colorize_cli_line(line));
    } else {
        println!("{line}");
    }
}

pub fn print_lines(lines: &[String]) {
    for line in lines {
        print_line(line);
    }
}

pub fn print_response(response: &CliResponse) {
    for entry in &response.entries {
        print_entry(entry);
    }
}

pub fn print_search_result_lines(lines: &[String], query: Option<&str>) {
    let color = use_color();
    for line in lines {
        finish_progress_bar();
        if color {
            println!("{}", colorize_search_line(line, query));
        } else {
            println!("{line}");
        }
    }
}

pub fn print_json_lines(lines: &[String]) {
    print_syntax_highlighted_lines(lines, "json");
}

pub fn print_toml_lines(lines: &[String]) {
    print_syntax_highlighted_lines(lines, "toml");
}

pub fn print_json_value<T: Serialize>(value: &T) {
    finish_progress_bar();
    let rendered = serde_json::to_string_pretty(value).expect("serialize json output");
    println!("{rendered}");
}

fn print_syntax_highlighted_lines(lines: &[String], extension: &str) {
    finish_progress_bar();
    let content = lines.join("\n");
    if use_color() {
        match syntax_highlight(content.as_str(), extension) {
            Some(rendered) => print!("{rendered}"),
            None => print_lines(lines),
        }
    } else {
        println!("{content}");
    }
}

pub fn print_stream_chunk(chunk: StreamChunk) {
    match chunk {
        StreamChunk::Append(line) | StreamChunk::ReplaceLast(line) => {
            if std::io::stdout().is_terminal() && update_progress_bar(&line) {
                return;
            }
            if !should_print_progress_line(&line) {
                return;
            }
            print_line(&line);
        }
    }
}

fn print_entry(entry: &CliEntry) {
    match entry {
        CliEntry::Header { level, title } => print_header(*level, title),
        CliEntry::Line { kind, text } => print_typed_line(*kind, text),
        CliEntry::KeyValue { key, value } => {
            print_typed_line(CliKind::Plain, &format!("{key}: {value}"))
        }
    }
}

fn print_header(level: u8, title: &str) {
    finish_progress_bar();
    let text = format!("{}{}:", "  ".repeat(level as usize), title);
    if use_color() {
        let token = if level == 0 {
            SemanticToken::Section
        } else {
            SemanticToken::Subsection
        };
        println!("{}", render_token(&text, token));
    } else {
        println!("{text}");
    }
}

fn print_typed_line(kind: CliKind, text: &str) {
    finish_progress_bar();
    if use_color() {
        println!("{}", colorize_typed_line(kind, text));
    } else {
        println!("{text}");
    }
}

fn colorize_search_line(line: &str, query: Option<&str>) -> String {
    let parts: Vec<_> = line.splitn(4, " | ").collect();
    if parts.len() != 4 {
        return line.to_string();
    }

    let package = {
        let highlighted = highlight_query(parts[0], query);
        render_token(&highlighted, SemanticToken::SearchPackage)
    };
    let version = render_token(parts[1], SemanticToken::SearchVersion);
    let bucket = render_token(parts[2], SemanticToken::SearchBucket);
    let description = {
        let highlighted = highlight_query(parts[3], query);
        render_token(&highlighted, SemanticToken::SearchDescription)
    };

    format!("{package} | {version} | {bucket} | {description}")
}

fn highlight_query(text: &str, query: Option<&str>) -> String {
    let Some(query) = query.map(str::trim).filter(|value| !value.is_empty()) else {
        return text.to_string();
    };
    let lower_text = text.to_ascii_lowercase();
    let lower_query = query.to_ascii_lowercase();
    let mut cursor = 0usize;
    let mut out = String::new();

    while let Some(found) = lower_text[cursor..].find(&lower_query) {
        let start = cursor + found;
        let end = start + lower_query.len();
        out.push_str(&text[cursor..start]);
        out.push_str(&text[start..end].with(Color::Yellow).bold().to_string());
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    out
}

fn colorize_cli_line(line: &str) -> String {
    if CLI_SECTION_HEADERS.contains(&line) {
        return render_token(line, SemanticToken::Section);
    }
    let indent = line.chars().take_while(|ch| *ch == ' ').count();
    if indent > 0 {
        let rest = &line[indent..];
        return format!("{}{}", " ".repeat(indent), colorize_indented_cli_line(rest));
    }
    if line.starts_with("> ") {
        return highlight_urls(&format!(
            "{}{}",
            render_token("> ", SemanticToken::Prompt),
            &line[2..]
        ));
    }
    if line.starts_with("Resolved Scoop package ") {
        return highlight_urls(&line.replacen(
            "Resolved Scoop package",
            &render_token("Resolved Scoop package", SemanticToken::Info),
            1,
        ));
    }
    if line.starts_with("Bootstrapping default 'main' bucket.") {
        return highlight_urls(&line.replacen(
            "Bootstrapping",
            &render_token("Bootstrapping", SemanticToken::Info),
            1,
        ));
    }
    if starts_with_any(line, CLI_INFO_PREFIXES) {
        return colorize_cli_info_line(line);
    }
    if starts_with_any(line, CLI_SUCCESS_PREFIXES) {
        return colorize_cli_success_line(line);
    }
    if line.starts_with("Download progress ") {
        return line.replacen(
            "Download progress",
            &render_token("Download progress", SemanticToken::Progress),
            1,
        );
    }
    if starts_with_any(line, CLI_WARNING_PREFIXES) {
        return colorize_cli_warning_line(line);
    }
    if contains_any(line, CLI_ERROR_TOKENS) {
        return colorize_cli_error_line(line);
    }
    highlight_urls(line)
}

fn colorize_indented_cli_line(line: &str) -> String {
    if CLI_SUBSECTION_HEADERS.contains(&line) {
        return render_token(line, SemanticToken::Subsection);
    }
    if let Some(installed_line) = colorize_installed_field(line) {
        return installed_line;
    }
    if let Some(url_line) = colorize_url_line(line) {
        return url_line;
    }
    if starts_with_any(line, CLI_SUCCESS_PREFIXES) {
        return colorize_cli_success_line(line);
    }
    if starts_with_any(line, CLI_INFO_PREFIXES) {
        return colorize_cli_info_line(line);
    }
    if starts_with_any(line, CLI_WARNING_PREFIXES) {
        return colorize_cli_warning_line(line);
    }
    highlight_urls(line)
}

fn colorize_installed_field(line: &str) -> Option<String> {
    let (prefix, value) = line.split_once(": ")?;
    if !prefix.eq_ignore_ascii_case("installed") {
        return None;
    }
    let rendered = if value.eq_ignore_ascii_case("yes") {
        render_token(value, SemanticToken::InstalledYes)
    } else if value.eq_ignore_ascii_case("no") {
        render_token(value, SemanticToken::InstalledNo)
    } else {
        value.to_string()
    };
    Some(format!("{prefix}: {rendered}"))
}

fn colorize_url_line(line: &str) -> Option<String> {
    if let Some(url) = line.strip_prefix("- ").map(str::trim) {
        if looks_like_url(url) {
            return Some(format!(
                "{} {}",
                render_token("-", SemanticToken::Bullet),
                render_token(url, SemanticToken::Link)
            ));
        }
    }
    if let Some((prefix, url)) = line.split_once(": ") {
        if looks_like_url(url) {
            return Some(format!(
                "{}: {}",
                prefix,
                render_token(url, SemanticToken::Link)
            ));
        }
    }
    None
}

fn looks_like_url(text: &str) -> bool {
    text.starts_with("https://") || text.starts_with("http://") || text.starts_with("file://")
}

fn highlight_urls(text: &str) -> String {
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let url_re = URL_RE.get_or_init(|| Regex::new(r"(https?://\S+|file://\S+)").unwrap());
    let mut out = String::new();
    let mut cursor = 0usize;
    for matched in url_re.find_iter(text) {
        out.push_str(&text[cursor..matched.start()]);
        out.push_str(&render_token(
            &text[matched.start()..matched.end()],
            SemanticToken::Link,
        ));
        cursor = matched.end();
    }
    out.push_str(&text[cursor..]);
    out
}

fn syntax_highlight(content: &str, extension: &str) -> Option<String> {
    let syntax_set = syntax_set();
    let syntax = syntax_set
        .find_syntax_by_extension(extension)
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, syntax_theme());
    let mut rendered = String::new();
    for line in LinesWithEndings::from(content) {
        let ranges = highlighter.highlight_line(line, syntax_set).ok()?;
        rendered.push_str(&as_24_bit_terminal_escaped(&ranges[..], false));
    }
    Some(rendered)
}

fn syntax_set() -> &'static SyntaxSet {
    static SET: OnceLock<SyntaxSet> = OnceLock::new();
    SET.get_or_init(SyntaxSet::load_defaults_newlines)
}

fn syntax_theme() -> &'static Theme {
    static THEME: OnceLock<Theme> = OnceLock::new();
    THEME.get_or_init(|| {
        let themes = ThemeSet::load_defaults();
        themes
            .themes
            .get("InspiredGitHub")
            .cloned()
            .or_else(|| themes.themes.values().next().cloned())
            .unwrap_or_default()
    })
}

#[cfg(test)]
fn with_color_mode<T>(mode: ColorMode, f: impl FnOnce() -> T) -> T {
    let slot = color_mode_slot();
    let mut guard = slot.lock().expect("color mode poisoned");
    let previous = *guard;
    *guard = mode;
    drop(guard);
    let result = f();
    *slot.lock().expect("color mode poisoned") = previous;
    result
}

fn colorize_typed_line(kind: CliKind, text: &str) -> String {
    match kind {
        CliKind::Plain => colorize_cli_line(text),
        CliKind::Info => render_token(text, SemanticToken::Info),
        CliKind::Warning => render_token(text, SemanticToken::Warning),
        CliKind::Error => render_token(text, SemanticToken::Error),
    }
}

fn colorize_cli_info_line(line: &str) -> String {
    for token in [
        "Starting",
        "Fetching",
        "Resolved Scoop package",
        "Resolved bucket branch:",
        "Using",
    ] {
        if line.starts_with(token) {
            return highlight_urls(&line.replacen(
                token,
                &render_token(token, SemanticToken::Info),
                1,
            ));
        }
    }
    highlight_urls(line)
}

fn colorize_cli_success_line(line: &str) -> String {
    for token in [
        "Registered",
        "Updated",
        "Removed",
        "Installed",
        "Created",
        "Completed",
        "Fetched",
        "Synced",
        "Ran",
        "Compiled",
    ] {
        if line.starts_with(token) {
            return highlight_urls(&line.replacen(
                token,
                &render_token(token, SemanticToken::Success),
                1,
            ));
        }
    }
    highlight_urls(line)
}

fn colorize_cli_warning_line(line: &str) -> String {
    for token in ["Skipped", "Warning:", "Path mismatch:"] {
        if line.starts_with(token) {
            return highlight_urls(&line.replacen(
                token,
                &render_token(token, SemanticToken::Warning),
                1,
            ));
        }
    }
    highlight_urls(line)
}

fn colorize_cli_error_line(line: &str) -> String {
    for token in [
        "Error:",
        "Cancelled by user.",
        "failed",
        "requires a configured root",
    ] {
        if line.contains(token) {
            return highlight_urls(&line.replacen(
                token,
                &render_token(token, SemanticToken::Error),
                1,
            ));
        }
    }
    highlight_urls(line)
}

fn starts_with_any(line: &str, prefixes: &[&str]) -> bool {
    prefixes.iter().any(|prefix| line.starts_with(prefix))
}

fn contains_any(line: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|token| line.contains(token))
}

fn progress_gate() -> &'static Mutex<Option<u64>> {
    static GATE: OnceLock<Mutex<Option<u64>>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(None))
}

fn cli_progress_bar() -> &'static Mutex<Option<ProgressBar>> {
    static BAR: OnceLock<Mutex<Option<ProgressBar>>> = OnceLock::new();
    BAR.get_or_init(|| Mutex::new(None))
}

fn finish_progress_bar() {
    if !std::io::stdout().is_terminal() {
        return;
    }
    let mut bar = cli_progress_bar().lock().expect("progress bar poisoned");
    if let Some(progress) = bar.take() {
        progress.finish_and_clear();
    }
}

fn update_progress_bar(line: &str) -> bool {
    let Some(progress) = parse_progress_line(line) else {
        return false;
    };

    let mut slot = cli_progress_bar().lock().expect("progress bar poisoned");
    match progress {
        DownloadProgress::Determinate {
            percent,
            summary,
            target,
        } => {
            let bar = slot.get_or_insert_with(|| {
                let bar = ProgressBar::new(100);
                bar.set_style(
                    ProgressStyle::with_template(if use_color() {
                        "{spinner:.cyan} {bar:30.cyan/blue} {pos:>3}% {msg}"
                    } else {
                        "{spinner} {bar:30} {pos:>3}% {msg}"
                    })
                    .expect("valid progress template")
                    .progress_chars("=>-"),
                );
                bar
            });
            bar.set_length(100);
            bar.set_position(percent.min(100));
            bar.set_message(format!("{summary} {target}"));
            if percent >= 100 {
                bar.finish_and_clear();
                *slot = None;
            }
        }
        DownloadProgress::Activity { summary, target } => {
            let bar = slot.get_or_insert_with(|| {
                let bar = ProgressBar::new_spinner();
                bar.set_style(
                    ProgressStyle::with_template(if use_color() {
                        "{spinner:.cyan} {msg}"
                    } else {
                        "{spinner} {msg}"
                    })
                    .expect("valid spinner template"),
                );
                bar.enable_steady_tick(std::time::Duration::from_millis(100));
                bar
            });
            bar.set_message(format!("{summary} {target}"));
        }
    }
    true
}

enum DownloadProgress<'a> {
    Determinate {
        percent: u64,
        summary: &'a str,
        target: &'a str,
    },
    Activity {
        summary: &'a str,
        target: &'a str,
    },
}

fn parse_progress_line(line: &str) -> Option<DownloadProgress<'_>> {
    parse_download_progress(line)
}

fn parse_download_progress(line: &str) -> Option<DownloadProgress<'_>> {
    let rest = line.strip_prefix("Download progress ")?;
    if let Some((percent_text, tail)) = rest.split_once("% ") {
        let percent = percent_text.parse::<u64>().ok()?;
        let summary_end = tail.rfind(") ")? + 1;
        let (summary, target) = tail.split_at(summary_end);
        return Some(DownloadProgress::Determinate {
            percent,
            summary,
            target: target.trim(),
        });
    }
    let summary_end = rest.rfind(") ")? + 1;
    let (summary, target) = rest.split_at(summary_end);
    Some(DownloadProgress::Activity {
        summary,
        target: target.trim(),
    })
}

fn should_print_progress_line(line: &str) -> bool {
    if let Some(percent) = parse_progress_percent(line) {
        let checkpoint = download_checkpoint(percent);
        let mut gate = progress_gate().lock().expect("progress gate poisoned");
        if gate.as_ref() == Some(&checkpoint) {
            return false;
        }
        *gate = Some(checkpoint);
        return true;
    }
    let mut gate = progress_gate().lock().expect("progress gate poisoned");
    *gate = None;
    true
}

fn parse_progress_percent(line: &str) -> Option<u64> {
    parse_prefixed_progress_percent(line, "Download progress ")
}

fn parse_prefixed_progress_percent(line: &str, prefix: &str) -> Option<u64> {
    let rest = line.strip_prefix(prefix)?;
    let percent_text = rest.split('%').next()?;
    percent_text.parse::<u64>().ok()
}

fn download_checkpoint(percent: u64) -> u64 {
    if percent >= 100 {
        100
    } else {
        (percent / 10) * 10
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ColorMode, colorize_cli_line, parse_progress_line, syntax_highlight, with_color_mode,
    };

    #[test]
    fn colorize_cli_line_only_highlights_bucket_sync_keyword() {
        let rendered = colorize_cli_line(
            "Starting Scoop bucket sync from https://example.com/repo into D:\\spoon\\scoop\\buckets\\main",
        );
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("Scoop bucket sync from https://example.com/repo"));
        assert!(!rendered.starts_with(
            "\u{1b}[36mStarting Scoop bucket sync from https://example.com/repo into"
        ));
    }

    #[test]
    fn colorize_cli_line_only_highlights_success_keyword() {
        let rendered = colorize_cli_line("Registered bucket 'extras'.");
        assert!(rendered.contains("\u{1b}["));
        assert!(rendered.contains("bucket 'extras'."));
        assert!(!rendered.starts_with("\u{1b}[32mRegistered bucket 'extras'."));
    }

    #[test]
    fn colorize_cli_line_highlights_installed_yes_no_value() {
        let yes = colorize_cli_line("  installed: yes");
        assert!(yes.contains("\u{1b}["));
        assert!(yes.contains("installed: "));

        let no = colorize_cli_line("  installed: no");
        assert!(no.contains("\u{1b}["));
        assert!(no.contains("installed: "));
    }

    #[test]
    fn parse_download_progress_extracts_determinate_summary_and_target() {
        let parsed = parse_progress_line(
            "Download progress 40% (22.0 MB / 55.6 MB) D:\\spoon\\scoop\\cache\\git.7z",
        )
        .expect("progress line");
        match parsed {
            super::DownloadProgress::Determinate {
                percent,
                summary,
                target,
            } => {
                assert_eq!(percent, 40);
                assert_eq!(summary, "(22.0 MB / 55.6 MB)");
                assert_eq!(target, "D:\\spoon\\scoop\\cache\\git.7z");
            }
            _ => panic!("expected determinate progress"),
        }
    }

    #[test]
    fn parse_download_progress_extracts_activity_summary_and_target() {
        let parsed =
            parse_progress_line("Download progress (12.3 MB downloaded) vs_BuildTools.exe")
                .expect("activity line");
        match parsed {
            super::DownloadProgress::Activity { summary, target } => {
                assert_eq!(summary, "(12.3 MB downloaded)");
                assert_eq!(target, "vs_BuildTools.exe");
            }
            _ => panic!("expected activity progress"),
        }
    }

    #[test]
    fn syntax_highlight_toml_emits_ansi_sequences() {
        with_color_mode(ColorMode::Always, || {
            let rendered = syntax_highlight("[policy.python]\npip_mirror = \"tuna\"\n", "toml")
                .expect("toml highlighting");
            assert!(rendered.contains("\u{1b}["));
            assert!(rendered.contains("pip_mirror"));
        });
    }

    #[test]
    fn render_tokenized_section_and_subsection_still_emit_ansi() {
        let section = colorize_cli_line("config:");
        assert!(section.contains("\u{1b}["));

        let subsection = colorize_cli_line("  Runtime:");
        assert!(subsection.contains("\u{1b}["));
    }
}
