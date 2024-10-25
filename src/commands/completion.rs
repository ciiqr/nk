use crate::args::{CompletionArgs, COMPLETION_FILES};
use clap::Command;
use clap_complete::generate;
use faccess::{AccessMode, PathExt};
use std::{fs::OpenOptions, io};

pub fn completion(
    args: &CompletionArgs,
    cmd: &mut Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let shell = args.command.as_shell();

    if let Some(shell) = shell {
        // print completions for shell
        generate(shell, cmd, cmd.get_name().to_string(), &mut io::stdout());
    } else {
        // install all applicable completions
        for completion in &*COMPLETION_FILES {
            let parent = completion
                .path
                .parent()
                .expect("all completion files have a parent path");

            if !parent.exists() {
                println!(
                    "- skipping {} (parent directory does not exist)",
                    completion.path.display()
                );
                continue;
            }

            // TODO: should support writing to root paths... (maybe just encourage user to run this one command with sudo?)
            if parent.access(AccessMode::WRITE).is_err() {
                println!(
                    "- skipping {} (parent directory is readonly)",
                    completion.path.display()
                );
                continue;
            }

            let mut completion_file = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(completion.path.as_path())
                .expect("completion path should be writable...");

            generate(
                completion.shell,
                cmd,
                cmd.get_name().to_string(),
                &mut completion_file,
            );

            println!("- installed {}", completion.path.display());
        }
    }

    Ok(())
}
