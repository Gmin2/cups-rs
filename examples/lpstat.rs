use chrono::{Local, TimeZone};
use cups_rs::config::{get_server, get_user, set_encryption, set_server, set_user, EncryptionMode};
use cups_rs::{
    bindings, destination::AttrKind, destination::AttrSpec, get_active_jobs, get_all_destinations,
    get_jobs, job::get_completed_jobs, ConnectionFlags, Destination, Destinations, Error,
    IppOperation, IppRequest, IppTag, IppValueTag, JobInfo, JobStatus, Result,
};
use std::{collections::HashMap, env, iter::Peekable, process::ExitCode};

#[derive(Debug, Clone, Copy)]
pub enum WhichJobs {
    Completed,
    NotCompleted,
    All,
}

impl WhichJobs {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "completed" => Some(Self::Completed),
            "not-completed" => Some(Self::NotCompleted),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Step {
    SetLongStatus(i32),
    SetRanking(bool),
    SetEncryption(bool),
    SetWhich(WhichJobs),
    SetServer(String),
    SetUser(String),
    Action(Action),
}

#[derive(Debug)]
pub enum Action {
    ShowServerAndPort, // -H
    ShowDefault,       // -d
    ListDestinations,  // -e
    ShowScheduler,     // -r
    ShowSummary,       // -s
    ShowAll,           // -t

    ShowAccepting(Option<String>),  // -a
    ShowClasses(Option<String>),    // -c
    ShowPrinters(Option<String>),   // -p
    ShowDevices(Option<String>),    // -v
    ShowJobsByDest(Option<String>), // -o
    ShowJobsByUser(Option<String>), // -u
    ShowJobsBareArg(String),        // special case
}

pub struct Context {
    dests: Vec<Destination>,
    long_status: i32,
    ranking: bool,
    which: WhichJobs,
}

impl Context {
    fn new() -> Result<Self> {
        Ok(Self {
            dests: get_all_destinations()?,
            long_status: 0,
            ranking: false,
            which: WhichJobs::NotCompleted,
        })
    }

    fn refresh_dests(&mut self) -> Result<()> {
        self.dests = get_all_destinations()?;
        Ok(())
    }
}

impl Action {
    fn run(&self, ctx: &mut Context) -> Result<()> {
        match self {
            Action::ShowServerAndPort => {
                let server = get_server();
                if server.starts_with('/') {
                    println!("{}", server);
                } else {
                    unsafe {
                        let port = bindings::ippPort();
                        println!("{}:{}", server, port);
                    }
                }
            }
            Action::ShowDefault => match Destinations::get_default() {
                Ok(dest) => show_default(Some(&dest)),
                Err(Error::DestinationNotFound(_)) => show_default(None),
                Err(e) => eprintln!("lpstat: error - {}", e),
            },
            Action::ListDestinations => {
                for dest in &ctx.dests {
                    if let Some(intance) = dest.instance.as_deref() {
                        print!("{}/{}", dest.name, intance);
                    } else {
                        print!("{}", dest.name);
                    }

                    if ctx.long_status > 0 {
                        let printer_uri_supported = dest
                            .options
                            .get("printer-uri-supported")
                            .map(|s| s.as_str());
                        let printer_is_temporary =
                            dest.options.get("printer-is-temporary").map(|s| s.as_str());

                        let ty = if matches!(printer_is_temporary, Some("true")) {
                            "temporary"
                        } else if printer_uri_supported.is_some() {
                            "permanent"
                        } else {
                            "network"
                        };

                        println!(
                            " {} {} {}",
                            ty,
                            printer_uri_supported.unwrap_or("none"),
                            dest.options
                                .get("device-uri")
                                .map(|s| s.as_str())
                                .unwrap_or("none")
                        );
                    } else {
                        println!();
                    }
                }
            }
            Action::ShowScheduler => {
                if scheduler_is_running() {
                    println!("scheduler is running");
                } else {
                    println!("scheduler is not running");
                }
            }
            Action::ShowSummary => {
                match Destinations::get_default() {
                    Ok(dest) => show_default(Some(&dest)),
                    Err(Error::DestinationNotFound(_)) => show_default(None),
                    Err(e) => eprintln!("lpstat: error - {}", e),
                }
                show_classes(None, ctx)?;
                show_devices(None, ctx)?;
            }
            Action::ShowAll => {
                if !scheduler_is_running() {
                    return Err(Error::ServerUnavailable);
                }
                println!("scheduler is running");
                match Destinations::get_default() {
                    Ok(dest) => show_default(Some(&dest)),
                    Err(Error::DestinationNotFound(_)) => show_default(None),
                    Err(e) => eprintln!("lpstat: error - {}", e),
                }
                show_classes(None, ctx)?;
                show_devices(None, ctx)?;
                show_accepting(None, ctx)?;
                show_printers(None, ctx)?;
                show_jobs(None, None, ctx)?;
            }

            Action::ShowAccepting(v) => {
                show_accepting(v.as_deref(), ctx)?;
            }
            Action::ShowClasses(v) => {
                show_classes(v.as_deref(), ctx)?;
            }
            Action::ShowPrinters(v) => {
                show_printers(v.as_deref(), ctx)?;
            }
            Action::ShowDevices(v) => {
                show_devices(v.as_deref(), ctx)?;
            }
            Action::ShowJobsByDest(v) => {
                show_jobs(None, v.clone(), ctx)?;
            }
            Action::ShowJobsByUser(v) => {
                show_jobs(v.clone(), None, ctx)?;
            }
            Action::ShowJobsBareArg(v) => {
                show_jobs(Some(v.clone()), None, ctx)?;
            }
        }
        Ok(())
    }
}

pub fn parse_args<I>(args: I) -> Result<Vec<Step>>
where
    I: IntoIterator<Item = String>,
{
    let mut it = args.into_iter().peekable();

    let mut commands: Vec<Step> = Vec::new();

    while let Some(arg) = it.next() {
        if arg == "--help" {
            print_help();
            std::process::exit(0);
        }

        if !arg.starts_with('-') || arg == "-" {
            commands.push(Step::Action(Action::ShowJobsByDest(Some(arg))));
            continue;
        }

        let rest = arg.strip_prefix('-').unwrap();
        let mut optchars = rest.chars();

        while let Some(ch) = optchars.next() {
            match ch {
                'D' => commands.push(Step::SetLongStatus(1)),
                'l' => commands.push(Step::SetLongStatus(2)),
                'R' => commands.push(Step::SetRanking(true)),
                'E' => commands.push(Step::SetEncryption(true)),

                'h' => {
                    let rest = optchars.as_str();
                    let v = take_require_value(rest, &mut it, "-h")?;
                    commands.push(Step::SetServer(v));
                    break;
                }
                'U' => {
                    let rest = optchars.as_str();
                    let v = take_require_value(rest, &mut it, "-U")?;
                    commands.push(Step::SetUser(v));
                    break;
                }
                'W' => {
                    let rest = optchars.as_str();
                    let v = take_require_value(rest, &mut it, "-W")?;
                    commands.push(Step::SetWhich(WhichJobs::parse(&v).ok_or_else(|| {
                        Error::ConfigurationError(
                            "need \"completed\", \"not-completed\", or \"all\" after -W"
                                .to_string(),
                        )
                    })?));
                    break;
                }

                'H' => commands.push(Step::Action(Action::ShowServerAndPort)),
                'd' => commands.push(Step::Action(Action::ShowDefault)),
                'e' => commands.push(Step::Action(Action::ListDestinations)),
                'r' => commands.push(Step::Action(Action::ShowScheduler)),
                's' => commands.push(Step::Action(Action::ShowSummary)),
                't' => commands.push(Step::Action(Action::ShowAll)),

                'a' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowAccepting(v)));
                    break;
                }

                'c' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowClasses(v)));
                    break;
                }

                'p' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowPrinters(v)));
                    break;
                }
                'v' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowDevices(v)));
                    break;
                }

                'o' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowJobsByDest(v)));
                    break;
                }

                'u' => {
                    let rest = optchars.as_str();
                    let v = take_optional_value(rest, &mut it);
                    commands.push(Step::Action(Action::ShowJobsByUser(v)));
                    break;
                }

                other => {
                    return Err(Error::ConfigurationError(format!(
                        "unknown option '-{other}'"
                    )));
                }
            }
        }
    }

    if !commands.iter().any(|c| matches!(c, Step::Action(_))) {
        commands.push(Step::Action(Action::ShowJobsByUser(Some(get_user()))));
    }
    Ok(commands)
}

fn take_require_value(
    rest_after_flag: &str,
    it: &mut Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<String> {
    if !rest_after_flag.is_empty() {
        return Ok(rest_after_flag.to_string());
    }

    match it.next() {
        Some(v) => Ok(v),
        None => Err(Error::ConfigurationError(format!(
            "expected value after {flag}"
        ))),
    }
}

fn take_optional_value(
    rest_after_flag: &str,
    it: &mut Peekable<impl Iterator<Item = String>>,
) -> Option<String> {
    if !rest_after_flag.is_empty() {
        return Some(rest_after_flag.to_string());
    }

    match it.peek() {
        Some(next) if !next.starts_with('-') => it.next(),
        _ => None,
    }
}

fn match_list(list: Option<&str>, name: &str, instance: Option<&str>) -> bool {
    let list = match list {
        None => return true,
        Some(s) if s.trim().is_empty() || s.eq_ignore_ascii_case("all") || s == "*" => return true,
        Some(s) => s.trim(),
    };

    let name = name.trim();
    if name.is_empty() {
        return false;
    }

    let full_instance = instance.map(|i| format!("{}/{}", name, i.trim()));

    list.split(|c: char| c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .any(|token| {
            if token == "*" || token.eq_ignore_ascii_case("all") || token.eq_ignore_ascii_case(name)
            {
                return true;
            }
            if let Some(ref full) = full_instance {
                if token.eq_ignore_ascii_case(full) {
                    return true;
                }
            }
            // Check if token is "name/"
            if let Some((t_name, t_inst)) = token.split_once('/') {
                if t_name.eq_ignore_ascii_case(name) && t_inst.is_empty() {
                    return true;
                }
            }
            false
        })
}

fn format_time(epoch_seconds: i64) -> String {
    Local
        .timestamp_opt(epoch_seconds, 0)
        .single()
        .map(|dt| dt.format("%a %b %e %H:%M:%S %Y").to_string())
        .unwrap_or_else(|| epoch_seconds.to_string())
}

fn print_help() {
    println!("Usage: lpstat [options]");
    println!("Options:");
    println!("-E                      Encrypt the connection to the server");
    println!("-h server[:port]        Connect to the named server and port");
    println!("-l                      Show verbose (long) output");
    println!("-U username             Specify the username to use for authentication");
    println!("-H                      Show the default server and port");
    println!("-W completed            Show completed jobs");
    println!("-W not-completed        Show pending jobs");
    println!("-a [destination(s)]     Show the accepting state of destinations");
    println!("-c [class(es)]          Show classes and their member printers");
    println!("-d                      Show the default destination");
    println!("-e                      Show available destinations on the network");
    println!("-o [destination(s)]     Show jobs");
    println!("-p [printer(s)]         Show the processing state of destinations");
    println!("-r                      Show whether the CUPS server is running");
    println!("-R                      Show the ranking of jobs");
    println!("-s                      Show a status summary");
    println!("-t                      Show all status information");
    println!("-u [user(s)]            Show jobs queued by the current or specified users");
    println!("-v [printer(s)]         Show the devices for each destination");
}

fn show_default(dest: Option<&Destination>) {
    if let Some(dest) = dest {
        if let Some(instance) = dest.instance.as_deref() {
            println!("system default destination: {}/{}", dest.name, instance);
        } else {
            println!("system default destination: {}", dest.name);
        }
        return;
    }

    let mut val: Option<&'static str> = None;
    let mut printer: Option<String> = None;

    if let Ok(p) = env::var("LPDEST") {
        val = Some("LPDEST");
        printer = Some(p);
    } else if let Ok(p) = env::var("PRINTER") {
        if p != "lp" {
            val = Some("PRINTER");
            printer = Some(p);
        }
    }

    if let Some(printer) = printer {
        println!(
            "lpstat: error - {} environment variable names non-existent destination \"{}\".",
            val.unwrap_or("UNKNOWN"),
            printer
        );
    } else {
        println!("no system default destination");
    }
}

fn scheduler_is_running() -> bool {
    unsafe {
        let http = bindings::httpConnectEncrypt(
            bindings::cupsServer(),
            bindings::ippPort(),
            bindings::cupsEncryption(),
        );
        if http.is_null() {
            return false;
        } else {
            bindings::httpClose(http);
            return true;
        }
    }
}

fn show_devices(opt: Option<&str>, ctx: &mut Context) -> Result<()> {
    for dest in &ctx.dests {
        if !match_list(opt, dest.name.as_str(), dest.instance.as_deref()) {
            continue;
        }

        let display_name = match dest.instance.as_deref() {
            Some(instance) => format!("{}/{}", dest.name, instance),
            None => dest.name.clone(),
        };

        let uri = dest
            .options
            .get("printer-uri-supported")
            .map(|s| s.as_str());
        let device = dest.options.get("device-uri").map(|s| s.as_str());

        let shown = match device {
            None => uri.unwrap_or("none"),
            Some(d) => d.strip_prefix("file:").unwrap_or(d),
        };

        println!("device for {}: {}", display_name, shown);
    }
    Ok(())
}

fn show_classes(filter: Option<&str>, ctx: &mut Context) -> Result<()> {
    const PRINTER_ATTRS: &[AttrSpec<'_>] = &[AttrSpec {
        name: "member-names",
        kind: AttrKind::StringLike,
    }];
    
    let conn = ctx.dests.first().and_then(|d| d.connect(ConnectionFlags::Scheduler, Some(5000), None).ok());

    for dest in &mut ctx.dests {
        if let Some(c) = &conn {
            let _ = dest.get_attrs(c, PRINTER_ATTRS);
        }

        let members_raw = match dest.options.get("member-names") {
            Some(s) => s.as_str(),
            None => continue,
        };

        if !match_list(filter, dest.name.as_str(), dest.instance.as_deref()) {
            continue;
        }

        println!("members of class {}:", dest.name);

        let mut printed_any = false;
        for member in members_raw
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(|s| s.trim())
            .filter(|t| !t.is_empty())
        {
            println!("\t{}", member);
            printed_any = true;
        }

        if !printed_any {
            println!("\tunknown");
        }
    }

    Ok(())
}

pub fn show_printers(filter: Option<&str>, ctx: &mut Context) -> Result<()> {
    // Attributes that lpstat show_printers() expects from the printer list:
    const PRINTER_ATTRS: &[AttrSpec<'_>] = &[
        AttrSpec {
            name: "printer-state",
            kind: AttrKind::IntegerLike,
        },
        AttrSpec {
            name: "printer-state-message",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-state-reasons",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-state-change-time",
            kind: AttrKind::IntegerLike,
        },
        AttrSpec {
            name: "printer-type",
            kind: AttrKind::IntegerLike,
        },
        AttrSpec {
            name: "printer-info",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-location",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-make-and-model",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-uri-supported",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "requesting-user-name-allowed",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "requesting-user-name-denied",
            kind: AttrKind::StringLike,
        },
    ];

    // CUPS_PRINTER_REMOTE bit (matches CUPS headers)
    const CUPS_PRINTER_REMOTE: i32 = 0x00000002;

    let conn = ctx.dests.first().and_then(|d| d.connect(ConnectionFlags::Scheduler, Some(5000), None).ok());

    for dest in &mut ctx.dests {
        if !match_list(filter, dest.name.as_str(), dest.instance.as_deref()) {
            continue;
        }

        if let Some(c) = &conn {
            let _ = dest.get_attrs(c, PRINTER_ATTRS);
        }

        let pstate = dest
            .options
            .get("printer-state")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(3);

        let ptime = dest
            .options
            .get("printer-state-change-time")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let printer_type = dest
            .options
            .get("printer-type")
            .and_then(|s| s.parse::<i32>().ok())
            .unwrap_or(0);

        let message = dest
            .options
            .get("printer-state-message")
            .map(|s| s.as_str());
        let description = dest
            .options
            .get("printer-info")
            .map(|s| s.as_str())
            .unwrap_or("");
        let location = dest
            .options
            .get("printer-location")
            .map(|s| s.as_str())
            .unwrap_or("");
        let make_model = dest
            .options
            .get("printer-make-and-model")
            .map(|s| s.as_str())
            .unwrap_or("");
        let uri = dest
            .options
            .get("printer-uri-supported")
            .map(|s| s.as_str());

        let reasons_str = dest
            .options
            .get("printer-state-reasons")
            .map(|s| s.as_str())
            .unwrap_or("");
        let holding_new_jobs = reasons_str
            .split(|c: char| c == ',' || c.is_whitespace())
            .map(str::trim)
            .any(|r| r == "hold-new-jobs");

        let printer_state_time = format_time(ptime);

        match pstate {
            3 => {
                if holding_new_jobs {
                    println!(
                        "printer {} is holding new jobs.  enabled since {}",
                        dest.full_name(),
                        printer_state_time
                    );
                } else {
                    println!(
                        "printer {} is idle.  enabled since {}",
                        dest.full_name(),
                        printer_state_time
                    );
                }
            }
            4 => {
                // NOTE: instances are “client-side”; jobs are typically associated with dest.name.
                let jobid = get_active_jobs(Some(dest.name.as_str()))
                    .unwrap_or_default()
                    .into_iter()
                    .find(|j| matches!(j.status, JobStatus::Processing))
                    .map(|j| j.id)
                    .unwrap_or(0);

                println!(
                    "printer {} now printing {}-{}.  enabled since {}",
                    dest.full_name(),
                    dest.name,
                    jobid,
                    printer_state_time
                );
            }
            5 => {
                println!(
                    "printer {} disabled since {} -",
                    dest.full_name(),
                    printer_state_time
                );
            }
            _ => {
                println!(
                    "printer {} is idle.  enabled since {}",
                    dest.full_name(),
                    printer_state_time
                );
            }
        }

        if (message.map(|m| !m.is_empty()).unwrap_or(false)) || pstate == 5 {
            if let Some(m) = message {
                if !m.is_empty() {
                    println!("\t{}", m);
                } else {
                    println!("\treason unknown");
                }
            } else {
                println!("\treason unknown");
            }
        }

        if ctx.long_status > 1 {
            println!("\tForm mounted:");
            println!("\tContent types: any");
            println!("\tPrinter types: unknown");
        }

        if ctx.long_status >= 1 {
            println!("\tDescription: {}", description);
            if !reasons_str.is_empty() {
                println!("\tAlerts: {}", reasons_str);
            }
        }

        if ctx.long_status > 1 {
            println!("\tLocation: {}", location);

            let is_remote = (printer_type & CUPS_PRINTER_REMOTE) != 0;
            if is_remote {
                println!("\tConnection: remote");

                if !make_model.contains("System V Printer") && !make_model.contains("Raw Printer") {
                    if let Some(u) = uri {
                        println!("\tInterface: {}.ppd", u);
                    }
                }
            } else {
                println!("\tConnection: direct");

                if !make_model.contains("Raw Printer") {
                    println!("\tInterface: /etc/cups/ppd/{}.ppd", dest.name);
                }
            }

            println!("\tOn fault: no alert");
            println!("\tAfter fault: continue");

            let allowed = dest
                .options
                .get("requesting-user-name-allowed")
                .map(|s| s.as_str());
            let denied = dest
                .options
                .get("requesting-user-name-denied")
                .map(|s| s.as_str());

            if let Some(a) = allowed.filter(|s| !s.trim().is_empty()) {
                println!("\tUsers allowed:");
                for u in a
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    println!("\t\t{}", u);
                }
            } else if let Some(d) = denied.filter(|s| !s.trim().is_empty()) {
                println!("\tUsers denied:");
                for u in d
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    println!("\t\t{}", u);
                }
            } else {
                println!("\tUsers allowed:");
                println!("\t\t(all)");
            }

            println!("\tForms allowed:");
            println!("\t\t(none)");
            println!("\tBanner required");
            println!("\tCharset sets:");
            println!("\t\t(none)");
            println!("\tDefault pitch:");
            println!("\tDefault page size:");
            println!("\tDefault port settings:");
        }
    }

    Ok(())
}

pub fn show_accepting(filter: Option<&str>, ctx: &mut Context) -> Result<()> {
    const ACCEPTING_ATTRS: &[AttrSpec<'_>] = &[
        AttrSpec {
            name: "printer-state-change-time",
            kind: AttrKind::IntegerLike,
        },
        AttrSpec {
            name: "printer-state-message",
            kind: AttrKind::StringLike,
        },
        AttrSpec {
            name: "printer-is-accepting-jobs",
            kind: AttrKind::StringLike,
        },
    ];

    fn parse_boolish(s: &str) -> bool {
        matches!(
            s.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    }

    let conn = ctx.dests.first().and_then(|d| d.connect(ConnectionFlags::Scheduler, Some(5000), None).ok());

    for dest in &mut ctx.dests {
        if !match_list(filter, dest.name.as_str(), dest.instance.as_deref()) {
            continue;
        }

        if let Some(c) = &conn {
            let _ = dest.get_attrs(c, ACCEPTING_ATTRS);
        }

        let ptime = dest
            .options
            .get("printer-state-change-time")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let message = dest
            .options
            .get("printer-state-message")
            .map(|s| s.as_str())
            .unwrap_or("");

        let accepting = dest
            .options
            .get("printer-is-accepting-jobs")
            .map(|s| parse_boolish(s))
            .unwrap_or(true);

        let printer_state_time = format_time(ptime);

        if accepting {
            println!(
                "{} accepting requests since {}",
                dest.full_name(),
                printer_state_time
            );
        } else {
            println!(
                "{} not accepting requests since {} -",
                dest.full_name(),
                printer_state_time
            );
            println!(
                "\t{}",
                if !message.is_empty() {
                    message
                } else {
                    "reason unknown"
                }
            );
        }
    }

    Ok(())
}

fn time_for_which(which: WhichJobs, j: &JobInfo) -> i64 {
    match which {
        WhichJobs::Completed => {
            if j.completed_time != 0 {
                j.completed_time
            } else {
                j.creation_time
            }
        }
        _ => j.creation_time,
    }
}

type JobExtraInfo = (Option<String>, Vec<String>);
type JobExtras = HashMap<(String, i32), JobExtraInfo>;

fn get_job_extras(which: WhichJobs) -> Result<JobExtras> {
    let mut extras = HashMap::new();
    let mut dests = get_all_destinations()?;
    let mut conn = None;

    for d in &mut dests {
        if let Ok(c) = d.connect(ConnectionFlags::Scheduler, Some(5000), None) {
            conn = Some(c);
            break;
        }
    }

    let Some(conn) = conn else {
        return Ok(extras);
    };

    let which_kw = match which {
        WhichJobs::All => "all",
        WhichJobs::NotCompleted => "not-completed",
        WhichJobs::Completed => "completed",
    };

    const REQ_ATTRS: &[&str] = &[
        "job-id",
        "job-printer-uri",
        "job-printer-state-message",
        "job-state-reasons",
    ];

    let mut req = IppRequest::new(IppOperation::GetJobs)?;
    let server = get_server();
    let uri = if server.starts_with('/') {
        "ipp://localhost/".to_string()
    } else {
        format!("ipp://{}/", server)
    };
    req.add_string(
        IppTag::Operation,
        IppValueTag::Uri,
        "printer-uri",
        &uri,
    )?;
    req.add_strings(
        IppTag::Operation,
        IppValueTag::Keyword,
        "requested-attributes",
        REQ_ATTRS,
    )?;
    req.add_string(
        IppTag::Operation,
        IppValueTag::Keyword,
        "which-jobs",
        which_kw,
    )?;

    let resp = req.send(&conn, conn.resource_path())?;
    if !resp.is_successful() {
        return Ok(extras);
    }

    let mut id: i32 = 0;
    let mut dest: Option<String> = None;
    let mut msg: Option<String> = None;
    let mut reasons: Vec<String> = Vec::new();

    for a in resp.attributes() {
        let Some(name) = a.name() else {
            continue;
        };

        match name.as_str() {
            "job-id" => {
                if id != 0 && dest.is_some() {
                    extras.insert((dest.unwrap(), id), (msg, reasons));
                    dest = None;
                    msg = None;
                    reasons = Vec::new();
                }
                id = a.get_integer(0);
            }
            "job-printer-uri" => {
                if dest.is_some() && id != 0 {
                    extras.insert((dest.unwrap(), id), (msg, reasons));
                    id = 0;
                    dest = None;
                    msg = None;
                    reasons = Vec::new();
                }
                if let Some(uri) = a.get_string(0) {
                    dest = Some(uri.rsplit('/').next().unwrap_or(uri.as_str()).to_string());
                }
            }
            "job-printer-state-message" => {
                msg = a.get_string(0);
            }
            "job-state-reasons" => {
                reasons = (0..a.count()).filter_map(|i| a.get_string(i)).collect();
            }
            _ => {}
        }
    }

    if id != 0 && dest.is_some() {
        extras.insert((dest.unwrap(), id), (msg, reasons));
    }

    Ok(extras)
}

pub fn show_jobs(user: Option<String>, dest: Option<String>, ctx: &mut Context) -> Result<()> {
    let mut jobs: Vec<JobInfo> = match ctx.which {
        WhichJobs::NotCompleted => get_active_jobs(None)?,
        WhichJobs::Completed => get_completed_jobs(None)?,
        WhichJobs::All => get_jobs(None)?,
    };

    jobs.retain(|j| match_list(dest.as_deref(), j.dest.as_str(), None));
    jobs.retain(|j| match_list(user.as_deref(), j.user.as_str(), None));

    let needs_extras = ctx.long_status != 0 || matches!(ctx.which, WhichJobs::Completed);

    let extras = if needs_extras {
        get_job_extras(ctx.which)?
    } else {
        HashMap::new()
    };

    let mut rank: i32 = -1;

    for j in jobs {
        rank += 1;

        let temp = format!("{}-{}", j.dest, j.id);
        let date = format_time(time_for_which(ctx.which, &j));
        let bytes = 1024.0 * (j.size as f64);
        let user = if j.user.is_empty() {
            "unknown"
        } else {
            j.user.as_str()
        };

        if ctx.ranking {
            println!(
                "{:3} {:<21} {:<13} {:8.0} {}",
                rank, temp, user, bytes, date
            );
        } else {
            println!("{:<23} {:<13} {:8.0}   {}", temp, user, bytes, date);
        }

        if ctx.long_status != 0 {
            if let Some((msg, reasons)) = extras.get(&(j.dest.clone(), j.id)) {
                if let Some(m) = msg.as_deref() {
                    println!("\tStatus: {}", m);
                }
                if !reasons.is_empty() {
                    println!("\tAlerts: {}", reasons.join(" "));
                }
            }

            println!("\tqueued for {}", j.dest);
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lpstat: {}", e);
            if e.to_string().contains("unknown option") || e.to_string().contains("expected value")
            {
                eprintln!();
                eprintln!("Use --help for usage information");
            }
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<()> {
    let commands = parse_args(env::args().skip(1))?;
    let mut ctx = Context::new()?;

    for cmd in commands {
        match cmd {
            Step::SetLongStatus(v) => ctx.long_status = ctx.long_status.max(v),
            Step::SetRanking(v) => ctx.ranking = v,
            Step::SetEncryption(_v) => {
                set_encryption(EncryptionMode::Required);
            }
            Step::SetWhich(v) => ctx.which = v,
            Step::SetServer(v) => {
                set_server(Some(v.as_str()))?;
                ctx.refresh_dests()?;
            }
            Step::SetUser(v) => {
                set_user(Some(v.as_str()))?;
            }
            Step::Action(a) => a.run(&mut ctx)?,
        }
    }

    Ok(())
}
