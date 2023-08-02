#![allow(non_snake_case)]

use std::io;
use std::process::Command;
use std::process::exit;

use clap::{Parser, Subcommand};
use mdbook::MDBook;
use serde::{Deserialize, Serialize};

use error_stack::{report, Result};

use env_logger;
use log::{debug, warn, error};

use semver::{Version, VersionReq};

use mdbook::preprocess::CmdPreprocessor;
use mdbook::preprocess::Preprocessor;

use mdbook_rfc::preprocessor::RFCPreprocessor;
use mdbook_rfc::config::{RFCBookConfig, BookConfig};
use mdbook_rfc::error::MdBookRFCError;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// Option
	#[command(subcommand)]
	command: Option<CmdQuery>,
}

#[derive(Debug, Subcommand)]
enum CmdQuery {
	/// Create a zip from a local folder
	Init {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a local folder
	Install {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a local folder
	Build {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a local folder
	Populate {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Check whether a renderer is supported by this preprocessor
	Supports {
		/// Renderer
		#[arg(short, long)]
		renderer: String,
	},
	/// Create a zip from a remote instance content
	Clean {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a remote instance content
	Watch {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a remote instance content
	New {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
	/// Create a zip from a remote instance content
	Serve {
		/// Folder with the custom emojis to generate the pack from.
		#[arg(short, long)]
		folder: String,
	},
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    match process(args.command) {
        Ok(()) => {},
        Err(err) => {
            error!("The process could not be completed. Quiting.");
            error!("\n{err:?}");
        }
    };
}

fn process(
	command: Option<CmdQuery>
) -> Result<(), MdBookRFCError> {
    
    // Users will want to construct their own preprocessor here
    let preprocessor = RFCPreprocessor::new();

	match command {
        Some(cmd) => match cmd {
            CmdQuery::Init       { folder }      => {
                MDBook::init(folder)
                    .create_gitignore(true)
                    //.with_config(cfg)
                    .build()
                    .expect("Book generation failed");
            },
            CmdQuery::Install    { folder }      => {
                Command::new("mdbook install")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::Build      { folder }      => {
                Command::new("mdbook build")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::Populate   { folder }      => {
                Command::new("mdbook init")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::Supports   { renderer }    => {
                handle_supports(&preprocessor, renderer);
            },
            CmdQuery::Clean      { folder }      => {
                Command::new("mdbook clean")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::Watch      { folder }      => {
                Command::new("mdbook watch")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::New        { folder }      => {
                Command::new("cp")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
            CmdQuery::Serve      { folder }      => {
                Command::new("mdbook serve")
                    .arg(folder)
                    .output()
                    .expect("Failed to execute command");

            },
        }
        None => handle_preprocessing(&preprocessor)?
	}

	Ok(())
}

fn handle_preprocessing(pre: &dyn Preprocessor) -> Result<(), MdBookRFCError> {
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

fn handle_supports(pre: &dyn Preprocessor, renderer: String) -> ! {
    let supported: bool = pre.supports_renderer(&renderer);

    // Signal whether the renderer is supported by exiting with 1 or 0.
    if supported {
        exit(0);
    } else {
        exit(1);
    }
}