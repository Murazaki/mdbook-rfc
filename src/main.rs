#![allow(non_snake_case)]

use std::env::current_dir;
use std::fs::create_dir;
use std::fs::remove_dir_all;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::process::exit;

use clap::{Parser, Subcommand};

use error_stack::{report, Result};

use env_logger;
use log::error;

use semver::{Version, VersionReq};

use walkdir::WalkDir;

use mdbook::Config as MDBookConfig;
use mdbook::preprocess::CmdPreprocessor;
use mdbook::preprocess::Preprocessor;

use mdbook_rfc::preprocessor::RFCPreprocessor;
use mdbook_rfc::config::RFCBookConfig;
use mdbook_rfc::error::MdBookRFCError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 
    #[arg(short, long, global = true)]
    folder: Option<String>,
	/// Option
	#[command(subcommand)]
	command: Option<CmdQuery>,
}

#[derive(Debug, Subcommand)]
enum CmdQuery {
	/// Initialize RFC book project
	Init {
	},
	/// Add preprocessor
	Add {
	},
	/// Install prerequisites
	Install {
	},
	/// Build book
	Build {
	},
	/// Populate source folder and update summary
	Populate {
	},
	/// Check whether a renderer is supported by this preprocessor
	Supports {
        /// 
        #[arg(short, long)]
        renderer: String,
	},
	/// Clean project
	Clean {
	},
	/// Watch for file modification and build
	Watch {
	},
	/// Create a new page from template
	New {
	},
	/// Serve book
	Serve {
	},
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    match process(args.folder, args.command) {
        Ok(()) => {},
        Err(err) => {
            error!("The process could not be completed. Quiting.");
            error!("\n{err:?}");
        }
    };
}

fn process(
    folder: Option<String>,
	command: Option<CmdQuery>
) -> Result<(), MdBookRFCError> {
    
    // Users will want to construct their own preprocessor here
    let preprocessor = RFCPreprocessor::new();
    let currentFolder = &folder.map(|s| PathBuf::from(s)).unwrap_or(current_dir().unwrap());

	match command {
        Some(cmd) => match cmd {
            CmdQuery::Init       { }                        => {
                handle_init(currentFolder)?;
            },
            CmdQuery::Install    { }                        => {
                handle_install(currentFolder)?;
            },
            CmdQuery::Add    { }                            => {
                handle_add(currentFolder)?;
            },
            CmdQuery::Build      { }                        => {
                handle_build(currentFolder)?;
            },
            CmdQuery::Populate   { }                        => {
                handle_populate(currentFolder)?;
            },
            CmdQuery::Supports   { renderer }       => {
                handle_supports(&preprocessor, renderer);
            },
            CmdQuery::Clean      { }                        => {
                handle_clean(currentFolder)?;
            },
            CmdQuery::Watch      { }                        => {
                handle_watch(currentFolder)?;
            },
            CmdQuery::New        { }                        => {
                handle_new(currentFolder)?;
            },
            CmdQuery::Serve      { }                        => {
                handle_serve(currentFolder)?;
            },
        }
        None => handle_preprocessing(&preprocessor)?
	}

	Ok(())
}

fn fetch_mdbook_config(
    currentFolder: &PathBuf,
) -> Result<MDBookConfig, MdBookRFCError> {
    Ok(MDBookConfig::from_disk(PathBuf::from(currentFolder).join("book.toml")).map_err(|_|
        report!(MdBookRFCError::Other)
                        .attach_printable("Could not fetch config.")
    )?)
}

fn fetch_rfcbook_config(
    mdBookCfg: MDBookConfig,
) -> Result<RFCBookConfig, MdBookRFCError> {
    let mut rfcBookCfg: RFCBookConfig = RFCBookConfig::default();

    match mdBookCfg.get("preprocessor") {
        Some(preprocessors) => {
            for (prepName, prepParams ) in preprocessors.as_table().ok_or_else(||
                report!(MdBookRFCError::Other)
                                .attach_printable(format!("Could not parse array of preprocessors."))
            )? {
                let package = match prepParams.get("command") {
                    Some(command) => command.as_str().unwrap(),
                    None => prepName
                };
        
                rfcBookCfg.preprocessors.push(prepName.into());
                rfcBookCfg.packages.push(package.into());
            };
        },
        None => println!("Could not find preprocessors to install.")
    }

    Ok(rfcBookCfg)
}

fn call_command(
    program: &str,
    args: Vec<&str>,
    currentFolder: &PathBuf,
    errorType: MdBookRFCError,
    error_msg: &str,
) -> Result<(), MdBookRFCError> {
    let err_msg_string = error_msg.to_string();

    let status = Command::new(program)
    .current_dir(currentFolder)
    .args(args)
    .status()
    .map_err(|_|
        report!(errorType.clone())
                        .attach_printable(err_msg_string.clone())
    )?;

    if !status.success() {
        return Err(report!(errorType)
                        .attach_printable(err_msg_string.clone()));
    }

    Ok(())
}

fn handle_init(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError>  {
    let args = vec!["init"];

    println!("Initializing book... ");
    call_command("mdbook", args, currentFolder, MdBookRFCError::Other, "Could not initialize book.")?;

    handle_install(currentFolder)?;

    Ok(())
}

fn handle_build(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let args = vec!["build"];

    println!("Building book... ");
    call_command("mdbook", args, currentFolder, MdBookRFCError::Other, "Could not build book.")?;

    Ok(())
}

fn handle_populate(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    Ok(())
}

fn handle_install(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {

    let mdBookCfg = fetch_mdbook_config(currentFolder)?;
    let rfcBookCfg = fetch_rfcbook_config(mdBookCfg)?;

    let preprocessorPackages: Vec<&str> = rfcBookCfg.packages.iter().map(|s|s.as_str()).collect();
    let packages = [&["mdbook"], &preprocessorPackages[..]].concat();
    let args = [&["install"], &packages[..]].concat();

    println!("Installing preprocessor packages: {:?}... ", packages);
    call_command("cargo", args, currentFolder, MdBookRFCError::Other, "Installing preprocessor packages failed.")?;

    for folder in [rfcBookCfg.templateFolder, rfcBookCfg.textFolder, rfcBookCfg.vendorFolder] {
        println!("Creating folder {:?}... ", folder);

        let path = currentFolder.join(folder.clone());
        if path.exists() {
            println!("Folder '{:?}' exists, no action needed. ", folder);
            continue;
        }

        create_dir(path).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Couldn't create folder {}.", folder))
        )?;
    }

    Ok(())

    // /TODO add project dir to extra-watch-dirs
}

fn handle_add(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let args = vec!["build"];

    println!("Building book... ");
    let status = Command::new("cargo")
        .current_dir(currentFolder)
        .args(args)
        .status()
        .map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not initialize book."))
        )?;

    if !status.success() {
        return Err(report!(MdBookRFCError::Other)
                        .attach_printable(format!("Could not initialize book.")));
    }

    Ok(())
}

fn handle_preprocessing(
    pre: &dyn Preprocessor
) -> Result<(), MdBookRFCError> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin()).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not parse command input."))
        )?;

    let book_version = Version::parse(&ctx.mdbook_version).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not parse book version."))
        )?;
    let version_req = VersionReq::parse(mdbook::MDBOOK_VERSION).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not parse mdbook version."))
        )?;

    if !version_req.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, \
             but we're being called from version {}",
            pre.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }

    let processed_book = pre.run(&ctx, book).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not preprocess book."))
        )?;
    serde_json::to_writer(io::stdout(), &processed_book).map_err(|_|
            report!(MdBookRFCError::Other)
                            .attach_printable(format!("Could not preprocess book."))
        )?;

    Ok(())
}

fn handle_supports(
    pre: &dyn Preprocessor,
    renderer: String,
) -> ! {
    let supported: bool = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        exit(0);
    } else {
        exit(1);
    }
}

fn handle_clean(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let args = vec!["clean"];

    println!("cleaning book... ");
    call_command("mdbook", args, currentFolder, MdBookRFCError::Other, "Could not initialize book.")?;

    let mdBookCfg = fetch_mdbook_config(currentFolder)?;
    let rfcBookCfg = fetch_rfcbook_config(mdBookCfg)?;

    remove_dir_all(currentFolder.join(rfcBookCfg.vendorFolder)).unwrap();

    Ok(())
}

fn handle_watch(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let args = vec!["watch"];

    println!("Initializing book... ");
    call_command("mdbook", args, currentFolder, MdBookRFCError::Other, "Could not initialize book.")?;

    Ok(())
}

fn handle_new(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let mdBookCfg = fetch_mdbook_config(currentFolder)?;
    let rfcBookCfg = fetch_rfcbook_config(mdBookCfg)?;

    for template in WalkDir::new(rfcBookCfg.templateFolder) {

    }

    Ok(())
}

fn handle_serve(
    currentFolder: &PathBuf,
) -> Result<(), MdBookRFCError> {
    let args = vec!["serve"];

    println!("Serving book...");
    call_command("mdbook", args, currentFolder, MdBookRFCError::Other, "Could not initialize book.")?;

    Ok(())
}
