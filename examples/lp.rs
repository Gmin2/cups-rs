use cups_rs::config::{set_encryption, set_server, set_user, EncryptionMode};
use cups_rs::{
    create_job_with_options, get_default_destination, get_destination, Error, PrintOptions, Result,
};
use std::{
    env,
    io::{self, Read},
    iter::Peekable,
    process::ExitCode,
};

#[derive(Debug)]
pub enum Step {
    SetEncryption(bool),
    SetServer(String),
    SetUser(String),
    SetDestination(String),
    SetTitle(String),
    SetCopies(u32),
    SetPriority(u32),
    SetSilent(bool),
    AddOption(String, String),
    SetHold(String),
    SetPageRanges(String),
    PrintFile(String),
    PrintStdin,
}

pub struct Context {
    destination: Option<String>,
    title: Option<String>,
    copies: Option<u32>,
    priority: Option<u32>,
    silent: bool,
    options: PrintOptions,
    files: Vec<String>,
    use_stdin: bool,
}

impl Context {
    fn new() -> Self {
        Self {
            destination: None,
            title: None,
            copies: None,
            priority: None,
            silent: false,
            options: PrintOptions::default(),
            files: Vec::new(),
            use_stdin: false,
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("lp: error - {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let steps = parse_args(args)?;

    if steps.is_empty() {
        print_help();
        return Ok(());
    }

    let mut ctx = Context::new();

    for step in steps {
        match step {
            Step::SetEncryption(e) => {
                if e {
                    set_encryption(EncryptionMode::Required);
                }
            }
            Step::SetServer(s) => set_server(Some(s.as_str()))?,
            Step::SetUser(u) => set_user(Some(u.as_str()))?,
            Step::SetDestination(d) => ctx.destination = Some(d),
            Step::SetTitle(t) => ctx.title = Some(t),
            Step::SetCopies(c) => ctx.copies = Some(c),
            Step::SetPriority(p) => ctx.priority = Some(p),
            Step::SetSilent(s) => ctx.silent = s,
            Step::AddOption(k, v) => {
                ctx.options = ctx.options.clone().custom_option(k, v);
            }
            Step::SetHold(h) => {
                let hold_val = match h.as_str() {
                    "hold" => "indefinite",
                    // Intentional simplification: in CUPS, `resume` and `release` are
                    // semantically distinct operations, but this implementation maps both
                    // to `no-hold` to clear the hold state.
                    "resume" | "release" | "immediate" => "no-hold",
                    "restart" => {
                        return Err(Error::ConfigurationError(
                            "'-H restart' not supported in this simplified version.".to_string(),
                        ));
                    }
                    val => val,
                };
                ctx.options = ctx
                    .options
                    .clone()
                    .custom_option("job-hold-until", hold_val);
                if h == "immediate" {
                    ctx.options = ctx.options.clone().custom_option("job-priority", "100");
                }
            }
            Step::SetPageRanges(p) => {
                ctx.options = ctx.options.clone().custom_option("page-ranges", p);
            }
            Step::PrintFile(f) => ctx.files.push(f),
            Step::PrintStdin => ctx.use_stdin = true,
        }
    }

    // destination
    let dest = match ctx.destination.as_deref() {
        Some(name) => get_destination(name)?,
        None => get_default_destination()?,
    };

    // Apply options
    if let Some(c) = ctx.copies {
        ctx.options = ctx.options.clone().copies(c);
    }
    if let Some(p) = ctx.priority {
        ctx.options = ctx
            .options
            .clone()
            .custom_option("job-priority", p.to_string());
    }

    let job_title = ctx.title.as_deref().unwrap_or(if ctx.files.is_empty() {
        "(stdin)"
    } else {
        &ctx.files[0]
    });

    let job = create_job_with_options(&dest, job_title, &ctx.options)?;

    let mut files_printed = 0;

    // Printing
    for file_path in &ctx.files {
        job.submit_file(file_path, "application/octet-stream")?;
        files_printed += 1;
    }

    if ctx.use_stdin || ctx.files.is_empty() {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer).map_err(|e| {
            Error::DocumentSubmissionFailed(format!("Failed to read from stdin: {}", e))
        })?;

        if !buffer.is_empty() {
            job.submit_data(&buffer, "application/octet-stream", job_title)?;
            files_printed += 1;
        }
    }

    if files_printed > 0 && !ctx.silent {
        println!(
            "request id is {}-{} ({} file(s))",
            dest.name,
            job.id,
            files_printed
        );
    }

    Ok(())
}

pub fn parse_args<I>(args: I) -> Result<Vec<Step>>
where
    I: IntoIterator<Item = String>,
{
    let mut it = args.into_iter().peekable();
    let mut steps: Vec<Step> = Vec::new();
    let mut end_options = false;

    while let Some(arg) = it.next() {
        if arg == "--help" {
            print_help();
            std::process::exit(0);
        }

        if end_options || !arg.starts_with("-") || arg == "-" {
            if arg == "-" {
                steps.push(Step::PrintStdin);
            } else {
                steps.push(Step::PrintFile(arg));
            }
            continue;
        }

        if arg == "--" {
            end_options = true;
            continue;
        }

        let rest = arg.strip_prefix('-').unwrap();
        let mut optchars = rest.chars();

        while let Some(ch) = optchars.next() {
            match ch {
                'E' => steps.push(Step::SetEncryption(true)),
                's' => steps.push(Step::SetSilent(true)),
                'd' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-d")?;
                    steps.push(Step::SetDestination(v));
                    break;
                }
                'h' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-h")?;
                    steps.push(Step::SetServer(v));
                    break;
                }
                'U' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-U")?;
                    steps.push(Step::SetUser(v));
                    break;
                }
                't' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-t")?;
                    steps.push(Step::SetTitle(v));
                    break;
                }
                'n' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-n")?;
                    let count = v.parse::<u32>().map_err(|_| {
                        Error::ConfigurationError("Invalid copies count".to_string())
                    })?;
                    steps.push(Step::SetCopies(count));
                    break;
                }
                'q' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-q")?;
                    let priority = v
                        .parse::<u32>()
                        .map_err(|_| Error::ConfigurationError("Invalid priority".to_string()))?;
                    steps.push(Step::SetPriority(priority));
                    break;
                }
                'o' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-o")?;
                    if let Some((k, val)) = v.split_once('=') {
                        steps.push(Step::AddOption(k.to_string(), val.to_string()));
                    } else {
                        steps.push(Step::AddOption(v, "true".to_string()));
                    }
                    break;
                }
                'H' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-H")?;
                    steps.push(Step::SetHold(v));
                    break;
                }
                'P' => {
                    let v = take_require_value(optchars.as_str(), &mut it, "-P")?;
                    steps.push(Step::SetPageRanges(v));
                    break;
                }
                'i' => {
                    let _v = take_require_value(optchars.as_str(), &mut it, "-i")?;
                    return Err(Error::ConfigurationError(
                        "-i (modify job) is not currently supported".to_string(),
                    ));
                }
                other => {
                    return Err(Error::ConfigurationError(format!(
                        "unknown option '-{}'",
                        other
                    )));
                }
            }
        }
    }

    Ok(steps)
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
            "expected value after {}",
            flag
        ))),
    }
}

fn print_help() {
    println!("Usage: lp [options] [--] [file(s)]");
    println!("       lp [options] -i id");
    println!("Options:");
    println!("-d destination          Specify the destination");
    println!("-E                      Encrypt the connection to the server");
    println!("-h server[:port]        Connect to the named server and port");
    println!("-H HH:MM                Hold the job until the specified UTC time");
    println!("-H hold                 Hold the job until released/resumed");
    println!("-H immediate            Print the job as soon as possible");
    println!("-H resume               Resume a held job");
    println!("-n num-copies           Specify the number of copies to print");
    println!("-o option[=value]       Specify a printer-specific option");
    println!("-P page-list            Specify a list of pages to print");
    println!("-q priority             Specify the priority from low (1) to high (100)");
    println!("-s                      Be silent");
    println!("-t title                Specify the job title");
    println!("-U username             Specify the username to use for authentication");
}
