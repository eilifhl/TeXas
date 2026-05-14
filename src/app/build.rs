use std::{
    env, fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct BuildResult {
    pub output: String,
    pub pdf_path: Option<PathBuf>,
}

pub fn run_latexmk(editor_text: &str) -> BuildResult {
    let build_dir = match create_build_dir() {
        Ok(path) => path,
        Err(error) => {
            return BuildResult {
                output: format!("Failed to prepare build directory:\n{error}"),
                pdf_path: None,
            };
        }
    };

    let tex_path = build_dir.join("main.tex");
    if let Err(error) = fs::write(&tex_path, editor_text) {
        return BuildResult {
            output: format!("Failed to write {}:\n{error}", tex_path.display()),
            pdf_path: None,
        };
    }

    let output = match Command::new("latexmk")
        .args([
            "-pdf",
            "-interaction=nonstopmode",
            "-halt-on-error",
            "main.tex",
        ])
        .current_dir(&build_dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => {
            return BuildResult {
                output: format!("Failed to run latexmk in {}:\n{error}", build_dir.display()),
                pdf_path: None,
            };
        }
    };

    let mut text = String::new();
    text.push_str("$ latexmk -pdf -interaction=nonstopmode -halt-on-error main.tex\n");
    text.push_str(&format!("Working directory: {}\n\n", build_dir.display()));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        text.push_str(&stdout);
        if !stdout.ends_with('\n') {
            text.push('\n');
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        if !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str("[stderr]\n");
        text.push_str(&stderr);
        if !stderr.ends_with('\n') {
            text.push('\n');
        }
    }

    text.push_str(&format!(
        "\nExit status: {}\n",
        output
            .status
            .code()
            .map(|code| code.to_string())
            .unwrap_or_else(|| "terminated by signal".to_owned())
    ));

    let pdf_path = build_dir.join("main.pdf");
    if pdf_path.exists() {
        text.push_str(&format!("PDF: {}\n", pdf_path.display()));
        BuildResult {
            output: text,
            pdf_path: Some(pdf_path),
        }
    } else {
        BuildResult {
            output: text,
            pdf_path: None,
        }
    }
}

fn create_build_dir() -> std::io::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let build_dir = env::temp_dir().join(format!("texas-{timestamp}"));
    fs::create_dir_all(&build_dir)?;
    Ok(build_dir)
}
