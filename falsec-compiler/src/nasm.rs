use crate::Target;
use std::fmt::Write;
use std::fs::File;
use std::path::Path;
use std::process::{Command, Output};

impl Target {
    pub fn into_nasm_target(self) -> &'static str {
        match self {
            Target::LinuxX86_64Elf => "elf64",
            _ => unreachable!("Unsupported target"),
        }
    }
}

pub fn assemble(assembly_path: &Path, output_path: &Path, target: Target) {
    let object_path = output_path.with_extension("o");
    File::create(&object_path).unwrap();
    let assembly_path = canonicalize_path(assembly_path);
    let object_path = canonicalize_path(&object_path);
    let output_path = canonicalize_path(output_path);
    match shell()
        .arg(format!(
            r#"nasm -f {} -o "{}" "{}""#,
            target.into_nasm_target(),
            object_path,
            assembly_path
        ))
        .output()
        .unwrap()
    {
        Output { status, stderr, .. } if !status.success() => {
            panic!(
                "Failed to assemble: {}\n{}",
                String::from_utf8(stderr).unwrap(),
                std::fs::read_to_string(assembly_path)
                    .unwrap()
                    .lines()
                    .enumerate()
                    .fold(String::new(), |mut output, (i, l)| {
                        writeln!(output, "{:4} {}", i + 1, l).unwrap();
                        output
                    })
            )
        }
        _ => (),
    };
    match shell()
        .arg(format!(r#"ld -o "{}" "{}""#, output_path, object_path))
        .output()
        .unwrap()
    {
        Output { status, stderr, .. } if !status.success() => {
            panic!("Failed to link: {}", String::from_utf8(stderr).unwrap())
        }
        _ => (),
    };
}

fn shell() -> Command {
    if cfg!(target_os = "windows") {
        let mut command = Command::new("bash.exe");
        command.arg("-c");
        command
    } else {
        let mut command = Command::new("sh");
        command.arg("-c");
        command
    }
}

fn canonicalize_path(path: &Path) -> String {
    if cfg!(target_os = "windows") {
        windows_path_into_wsl_path(path)
    } else {
        path.canonicalize()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    }
}

fn windows_path_into_wsl_path(path: &Path) -> String {
    String::from_utf8(
        match Command::new("wsl")
            .arg("wslpath")
            .arg("-a")
            .arg(
                path.canonicalize()
                    .unwrap()
                    .into_os_string()
                    .into_string()
                    .unwrap()
                    .replace('\\', "\\\\"),
            )
            .output()
        {
            Ok(Output { stdout, status, .. }) if status.success() => stdout,
            Err(e) => panic!("Failed to run wslpath: {}", e),
            Ok(Output { stderr, .. }) => {
                panic!("wslpath failed: {}", String::from_utf8(stderr).unwrap())
            }
        },
    )
    .unwrap()
}

#[cfg(test)]
mod tests {}
