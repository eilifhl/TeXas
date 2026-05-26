use std::{
    env, fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub struct BuildResult {
    pub output: String,
    pub pdf_path: Option<PathBuf>,
}

pub fn run_tectonic(editor_text: &str) -> BuildResult {
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

    let mut text = String::new();
    text.push_str("$ tectonic::latex_to_pdf(main.tex)\n");
    text.push_str(&format!("Working directory: {}\n\n", build_dir.display()));

    let pdf_path = build_dir.join("main.pdf");
    let pdf_bytes = match tectonic::latex_to_pdf(editor_text) {
        Ok(pdf_bytes) => pdf_bytes,
        Err(error) => {
            text.push_str(&format!("Compilation failed:\n{error}\n"));
            return BuildResult {
                output: text,
                pdf_path: None,
            };
        }
    };

    if let Err(error) = fs::write(&pdf_path, &pdf_bytes) {
        text.push_str(&format!(
            "Compilation succeeded, but failed to write {}:\n{error}\n",
            pdf_path.display()
        ));
        BuildResult {
            output: text,
            pdf_path: None,
        }
    } else {
        text.push_str(&format!(
            "Compilation succeeded.\nOutput PDF size: {} bytes\nPDF: {}\n",
            pdf_bytes.len(),
            pdf_path.display()
        ));
        BuildResult {
            output: text,
            pdf_path: Some(pdf_path),
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
