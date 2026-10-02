use std::io::IsTerminal;

const TERMINAL_IMAGE_BYTES: &[u8] = include_bytes!("../assets/aiflow_terminal.jpg");

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0F) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Detect if the current terminal supports inline raster images (iTerm2, Ghostty, Kitty, VS Code, WezTerm)
pub fn supports_inline_images() -> bool {
    if !std::io::stdout().is_terminal() {
        return false;
    }

    if let Ok(override_mode) = std::env::var("AIFLOW_BANNER") {
        let val = override_mode.to_lowercase();
        if val == "ascii" || val == "none" || val == "text" || val == "false" || val == "0" {
            return false;
        }
        if val == "image" || val == "true" || val == "1" {
            return true;
        }
    }

    if let Ok(term_prog) = std::env::var("TERM_PROGRAM") {
        let tp = term_prog.to_lowercase();
        if tp.contains("iterm")
            || tp.contains("ghostty")
            || tp.contains("vscode")
            || tp.contains("wezterm")
        {
            return true;
        }
    }

    if let Ok(lc) = std::env::var("LC_TERMINAL") {
        let lc_lower = lc.to_lowercase();
        if lc_lower.contains("iterm") || lc_lower.contains("ghostty") || lc_lower.contains("wezterm") {
            return true;
        }
    }

    if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return true;
    }

    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return true;
    }

    if let Ok(term) = std::env::var("TERM") {
        if term.contains("kitty") || term.contains("ghostty") || term.contains("wezterm") {
            return true;
        }
    }

    false
}

/// Emit inline image using Kitty or iTerm2 protocol
pub fn print_terminal_image() {
    let is_kitty = std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM")
            .map(|t| t.contains("kitty"))
            .unwrap_or(false);

    let b64 = base64_encode(TERMINAL_IMAGE_BYTES);

    if is_kitty {
        // Kitty graphics protocol
        let chunk_size = 4096;
        let bytes = b64.as_bytes();
        for (i, chunk) in bytes.chunks(chunk_size).enumerate() {
            let is_last = (i + 1) * chunk_size >= bytes.len();
            let m = if is_last { 0 } else { 1 };
            if let Ok(chunk_str) = std::str::from_utf8(chunk) {
                if i == 0 {
                    print!("\x1b_Ga=T,f=100,m={};{}\x1b\\", m, chunk_str);
                } else {
                    print!("\x1b_Gm={};{}\x1b\\", m, chunk_str);
                }
            }
        }
        println!();
    } else {
        // iTerm2 / Ghostty / VS Code / WezTerm protocol
        print!("\x1b]1337;File=inline=1;width=16;preserveAspectRatio=1:{}\x07\n", b64);
    }
}

pub struct FirstTimeInfo<'a> {
    pub workspace_name: &'a str,
    pub language: &'a str,
    pub is_initialized: bool,
    pub ai_stack: Option<&'a str>,
}

pub struct WorkingStageInfo<'a> {
    pub project_name: &'a str,
    pub language: &'a str,
    pub phase_name: &'a str,
    pub phase_role: &'a str,
    pub ai_stack: &'a str,
    pub tasks_completed: usize,
    pub tasks_total: usize,
    pub branch: &'a str,
    pub modified_count: usize,
    pub untracked_count: usize,
}

pub enum BannerMode<'a> {
    FirstTime(FirstTimeInfo<'a>),
    WorkingStage(WorkingStageInfo<'a>),
}

/// Render the brand banner (Image on modern terminals, clean Fastfetch on other terminals)
pub fn render_banner(mode: &BannerMode) {
    if let Ok(override_mode) = std::env::var("AIFLOW_BANNER") {
        if override_mode.to_lowercase() == "none" {
            return;
        }
    }

    if supports_inline_images() {
        print_terminal_image();
        match mode {
            BannerMode::FirstTime(info) => render_image_info_first_time(info),
            BannerMode::WorkingStage(info) => render_image_info_working_stage(info),
        }
    } else {
        match mode {
            BannerMode::FirstTime(info) => render_fastfetch_first_time(info),
            BannerMode::WorkingStage(info) => render_fastfetch_working_stage(info),
        }
    }
}

fn fastfetch_logo_lines() -> [String; 7] {
    let slate = "\x1b[38;2;100;116;139m";
    let cyan = "\x1b[38;2;56;189;248m";
    let blue = "\x1b[38;2;14;165;233m";
    let orange = "\x1b[38;2;249;115;22m";
    let reset = "\x1b[0m";

    [
        format!("  {slate}>_ ─┬─{reset}        {cyan}▄█{reset}       "),
        format!("      {slate}│{reset}   {cyan}▄█▀{reset}    {cyan}▄██▀{reset}    "),
        format!("      {slate}│{reset}  {cyan}██▀{reset}    {cyan}▄██▀{reset}     "),
        format!("      {slate}├─{reset}{blue}██{reset}     {cyan}▄██▀{reset}      "),
        format!("      {slate}│{reset}{blue}██{reset}    {cyan}▄██▀{reset}  {orange}██{reset}    "),
        format!("      {slate}│{reset}{blue}▀██▄▄██▀{reset}   {orange}██{reset}     "),
        format!("       {slate}▀▀▀▀{reset}      {orange}▀▀{reset}      "),
    ]
}

fn render_fastfetch_first_time(info: &FirstTimeInfo) {
    let white = "\x1b[1;37m";
    let dim = "\x1b[38;2;148;163;184m";
    let slate = "\x1b[38;2;100;116;139m";
    let cyan = "\x1b[38;2;56;189;248m";
    let yellow = "\x1b[38;2;234;179;8m";
    let reset = "\x1b[0m";

    let logo = fastfetch_logo_lines();
    let version = env!("CARGO_PKG_VERSION");

    let status_str = if info.is_initialized {
        format!("{yellow}Initialized (Welcome to AIFlow){reset}")
    } else {
        format!("{yellow}First-Time Setup (Not Initialized){reset}")
    };

    let quick_start = if info.is_initialized {
        format!("Review spec & run {cyan}aiflow status{reset}")
    } else {
        format!("Run {cyan}aiflow init{reset} to start workflow tracking")
    };

    let stack_display = info
        .ai_stack
        .unwrap_or("Claude Architect • Codex Coder • All-Claude");

    let info_lines = [
        format!("{white}A I F L O W{reset}  {slate}•{reset}  {cyan}v{version}{reset}"),
        format!("{dim}Local AI-Assisted Developer Control Plane{reset}"),
        format!("{slate}─────────────────────────────────────────────────{reset}"),
        format!("{dim}Status:     {reset} {status_str}"),
        format!(
            "{dim}Workspace:  {reset} {white}{}{reset} ({cyan}detected: {}{reset})",
            info.workspace_name, info.language
        ),
        format!("{dim}AI Stacks:  {reset} {slate}{stack_display}{reset}"),
        format!("{dim}Quick Start:{reset} {quick_start}"),
    ];

    println!();
    for i in 0..7 {
        println!("{} │  {}", logo[i], info_lines[i]);
    }
    println!();
}

fn render_fastfetch_working_stage(info: &WorkingStageInfo) {
    let white = "\x1b[1;37m";
    let dim = "\x1b[38;2;148;163;184m";
    let slate = "\x1b[38;2;100;116;139m";
    let cyan = "\x1b[38;2;56;189;248m";
    let green = "\x1b[38;2;34;197;94m";
    let yellow = "\x1b[38;2;234;179;8m";
    let reset = "\x1b[0m";

    let logo = fastfetch_logo_lines();
    let version = env!("CARGO_PKG_VERSION");

    let tree_clean = info.modified_count == 0 && info.untracked_count == 0;
    let tree_status = if tree_clean {
        format!("{slate}(clean){reset}")
    } else {
        format!("{yellow}({} mod, {} untracked){reset}", info.modified_count, info.untracked_count)
    };

    let info_lines = [
        format!("{white}A I F L O W{reset}  {slate}•{reset}  {cyan}v{version}{reset}"),
        format!("{dim}Local AI-Assisted Developer Control Plane{reset}"),
        format!("{slate}─────────────────────────────────────────────────{reset}"),
        format!(
            "{dim}Project:    {reset} {white}{}{reset} ({cyan}{}{reset})",
            info.project_name, info.language
        ),
        format!(
            "{dim}Stage:      {reset} {green}{}{reset} {yellow}[ACTIVE - {}]{reset}",
            info.phase_name, info.phase_role
        ),
        format!("{dim}AI Stack:   {reset} {yellow}{}{reset}", info.ai_stack),
        format!(
            "{dim}Tasks:      {reset} {white}{}/{} completed{reset}  {slate}•{reset}  {dim}Branch:{reset} {cyan}{}{reset} {tree_status}",
            info.tasks_completed, info.tasks_total, info.branch
        ),
    ];

    println!();
    for i in 0..7 {
        println!("{} │  {}", logo[i], info_lines[i]);
    }
    println!();
}

fn render_image_info_first_time(info: &FirstTimeInfo) {
    let white = "\x1b[1;37m";
    let dim = "\x1b[38;2;148;163;184m";
    let slate = "\x1b[38;2;100;116;139m";
    let cyan = "\x1b[38;2;56;189;248m";
    let yellow = "\x1b[38;2;234;179;8m";
    let reset = "\x1b[0m";
    let version = env!("CARGO_PKG_VERSION");

    let status_str = if info.is_initialized {
        format!("{yellow}Initialized (Welcome to AIFlow){reset}")
    } else {
        format!("{yellow}First-Time Setup (Not Initialized){reset}")
    };

    let quick_start = if info.is_initialized {
        format!("Review spec & run {cyan}aiflow status{reset}")
    } else {
        format!("Run {cyan}aiflow init{reset} to start workflow tracking")
    };

    let stack_display = info
        .ai_stack
        .unwrap_or("Claude Architect • Codex Coder • All-Claude");

    println!("{white}A I F L O W{reset}  {slate}•{reset}  {cyan}v{version}{reset}");
    println!("{dim}Local AI-Assisted Developer Control Plane{reset}");
    println!("{slate}─────────────────────────────────────────────────{reset}");
    println!("{dim}Status:     {reset} {status_str}");
    println!(
        "{dim}Workspace:  {reset} {white}{}{reset} ({cyan}detected: {}{reset})",
        info.workspace_name, info.language
    );
    println!("{dim}AI Stacks:  {reset} {slate}{stack_display}{reset}");
    println!("{dim}Quick Start:{reset} {quick_start}\n");
}

fn render_image_info_working_stage(info: &WorkingStageInfo) {
    let white = "\x1b[1;37m";
    let dim = "\x1b[38;2;148;163;184m";
    let slate = "\x1b[38;2;100;116;139m";
    let cyan = "\x1b[38;2;56;189;248m";
    let green = "\x1b[38;2;34;197;94m";
    let yellow = "\x1b[38;2;234;179;8m";
    let reset = "\x1b[0m";
    let version = env!("CARGO_PKG_VERSION");

    let tree_clean = info.modified_count == 0 && info.untracked_count == 0;
    let tree_status = if tree_clean {
        format!("{slate}(clean){reset}")
    } else {
        format!("{yellow}({} mod, {} untracked){reset}", info.modified_count, info.untracked_count)
    };

    println!("{white}A I F L O W{reset}  {slate}•{reset}  {cyan}v{version}{reset}");
    println!("{dim}Local AI-Assisted Developer Control Plane{reset}");
    println!("{slate}─────────────────────────────────────────────────{reset}");
    println!(
        "{dim}Project:    {reset} {white}{}{reset} ({cyan}{}{reset})",
        info.project_name, info.language
    );
    println!(
        "{dim}Stage:      {reset} {green}{}{reset} {yellow}[ACTIVE - {}]{reset}",
        info.phase_name, info.phase_role
    );
    println!("{dim}AI Stack:   {reset} {yellow}{}{reset}", info.ai_stack);
    println!(
        "{dim}Tasks:      {reset} {white}{}/{} completed{reset}  {slate}•{reset}  {dim}Branch:{reset} {cyan}{}{reset} {tree_status}\n",
        info.tasks_completed, info.tasks_total, info.branch
    );
}
