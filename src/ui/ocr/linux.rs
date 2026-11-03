use std::{
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use color_eyre::eyre::{eyre, Result, WrapErr};
use image::{
    codecs::png::{CompressionType, FilterType, PngEncoder},
    ImageEncoder,
};

use super::{OcrJob, OcrLanguage};

const LANGUAGE_TIMEOUT: Duration = Duration::from_secs(10);
const SCAN_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_STDOUT: usize = 4 * 1024 * 1024;
const MAX_STDERR: usize = 64 * 1024;
const INSTALL_HINT: &str = "Install Tesseract and its language data (Ubuntu/Debian: sudo apt install tesseract-ocr tesseract-ocr-eng tesseract-ocr-kor), then restart Edolview.";

pub(super) struct Backend {
    executable: PathBuf,
    languages: Vec<String>,
}

impl Backend {
    pub(super) fn new() -> Result<Self> {
        Self::with_executable(PathBuf::from("tesseract"))
    }

    fn with_executable(executable: PathBuf) -> Result<Self> {
        // This backend is constructed on the first image context-menu opening,
        // never at app startup. The language-list process exits immediately.
        let output = run_command(&executable, &["--list-langs"], None, LANGUAGE_TIMEOUT)?;
        let output = String::from_utf8(output).wrap_err("Tesseract language list is not UTF-8")?;
        Ok(Self {
            executable,
            languages: parse_languages(&output)?,
        })
    }

    pub(super) fn available_languages(&self) -> Result<(Vec<OcrLanguage>, String, u32)> {
        let locale = ["LC_ALL", "LC_MESSAGES", "LANG"]
            .into_iter()
            .find_map(|key| std::env::var(key).ok().filter(|value| !value.is_empty()))
            .unwrap_or_default();
        let language = default_language(&self.languages, &locale).to_string();
        let languages = self
            .languages
            .iter()
            .map(|tag| OcrLanguage {
                tag: tag.clone(),
                label: language_label(tag),
            })
            .collect();
        // Application memory/readback limit, not a Tesseract engine restriction.
        Ok((languages, language, 4096))
    }

    pub(super) fn recognize(&mut self, job: OcrJob) -> Result<String> {
        job.validate()?;
        if !self.languages.contains(&job.language) {
            return Err(eyre!("OCR language is not installed: {}", job.language));
        }
        let mut png = Vec::new();
        PngEncoder::new_with_quality(&mut png, CompressionType::Fast, FilterType::NoFilter)
            .write_image(&job.rgba, job.width, job.height, image::ExtendedColorType::Rgba8)
            .wrap_err("Could not encode the OCR region")?;
        // Only the selected region is encoded, on the OCR worker. Free the raw
        // readback before starting Tesseract; stdin avoids temporary image files.
        drop(job.rgba);
        let output = run_command(
            &self.executable,
            &["stdin", "stdout", "-l", &job.language, "--psm", "6"],
            Some(png),
            SCAN_TIMEOUT,
        )?;
        let text = String::from_utf8(output).wrap_err("Tesseract result is not UTF-8")?;
        Ok(text.trim_end_matches(['\r', '\n', '\u{c}']).to_string())
    }
}

fn parse_languages(output: &str) -> Result<Vec<String>> {
    let mut languages: Vec<_> = output
        .lines()
        .map(str::trim)
        .filter(|tag| {
            !tag.is_empty()
                && *tag != "osd"
                && !tag.starts_with('-')
                && tag
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'/'))
        })
        .map(str::to_owned)
        .collect();
    languages.sort();
    languages.dedup();
    if languages.is_empty() {
        return Err(eyre!("No Tesseract OCR languages found. {INSTALL_HINT}"));
    }
    Ok(languages)
}

fn default_language<'a>(languages: &'a [String], locale: &str) -> &'a str {
    let locale = locale
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .replace('-', "_");
    let base = locale.split('_').next().unwrap_or_default();
    let preferred = match base {
        "en" => "eng",
        "ko" => "kor",
        "ja" => "jpn",
        "zh" if locale.contains("hant")
            || locale.contains("_tw")
            || locale.contains("_hk")
            || locale.contains("_mo") =>
        {
            "chi_tra"
        }
        "zh" => "chi_sim",
        "de" => "deu",
        "fr" => "fra",
        "es" => "spa",
        "it" => "ita",
        "pt" => "por",
        "ru" => "rus",
        "ar" => "ara",
        "hi" => "hin",
        "nl" => "nld",
        "pl" => "pol",
        "tr" => "tur",
        "uk" => "ukr",
        "vi" => "vie",
        "th" => "tha",
        other => other,
    };
    languages
        .iter()
        .find(|tag| *tag == preferred)
        .or_else(|| languages.iter().find(|tag| *tag == "eng"))
        .unwrap_or(&languages[0])
        .as_str()
}

fn language_label(tag: &str) -> String {
    let name = match tag {
        "eng" => "English",
        "kor" => "한국어",
        "jpn" => "日本語",
        "chi_sim" => "简体中文",
        "chi_tra" => "繁體中文",
        "deu" => "Deutsch",
        "fra" => "Français",
        "spa" => "Español",
        "ita" => "Italiano",
        "por" => "Português",
        "rus" => "Русский",
        _ => return tag.to_string(),
    };
    format!("{name} ({tag})")
}

// Reap on every return path, including I/O failures. There is no idle engine
// process: one command completes before another job can reach this worker.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn run_command(executable: &Path, args: &[&str], input: Option<Vec<u8>>, timeout: Duration) -> Result<Vec<u8>> {
    let mut child = ChildGuard(
        Command::new(executable).args(args)
        // Stable machine-readable language-list header, without changing the
        // app's locale or invoking a shell. Language names remain UTF-8.
        .env("LC_ALL", "C")
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().wrap_err_with(|| format!("Could not start Tesseract. {INSTALL_HINT}"))?,
    );
    let stdin = child.0.stdin.take();
    let stdout = child.0.stdout.take().ok_or_else(|| eyre!("Could not open Tesseract stdout"))?;
    let stderr = child.0.stderr.take().ok_or_else(|| eyre!("Could not open Tesseract stderr"))?;
    thread::scope(|scope| {
        // Drain both pipes while writing the image. Waiting before draining (or
        // writing everything first) can deadlock when a pipe buffer fills.
        let writer = scope.spawn(move || -> io::Result<()> {
            if let (Some(mut stdin), Some(input)) = (stdin, input) {
                stdin.write_all(&input)?;
            }
            Ok(())
        });
        let stdout = scope.spawn(move || read_limited(stdout, MAX_STDOUT));
        let stderr = scope.spawn(move || read_limited(stderr, MAX_STDERR));
        let status = wait_with_timeout(&mut child.0, timeout);
        let written = writer.join().map_err(|_| eyre!("Tesseract input worker panicked"))?;
        let (output, overflow) = stdout.join().map_err(|_| eyre!("Tesseract output worker panicked"))??;
        let (error, _) = stderr.join().map_err(|_| eyre!("Tesseract error worker panicked"))??;
        let status = status?;
        if !status.success() {
            return Err(eyre!(
                "Tesseract exited with {status}: {}",
                String::from_utf8_lossy(&error).trim()
            ));
        }
        written.wrap_err("Could not send the image to Tesseract")?;
        if overflow {
            return Err(eyre!("Tesseract OCR output exceeded the 4 MiB limit"));
        }
        Ok(output)
    })
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Result<ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if start.elapsed() < timeout => thread::sleep(Duration::from_millis(20)),
            result => {
                // Close the child's pipe endpoints before the scoped reader and
                // writer joins; returning first would leave those joins blocked.
                let _ = child.kill();
                let _ = child.wait();
                return match result {
                    Err(error) => Err(error).wrap_err("Could not wait for Tesseract"),
                    _ => Err(eyre!("Tesseract timed out after {} seconds", timeout.as_secs_f32())),
                };
            }
        }
    }
}

fn read_limited(mut reader: impl Read, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    let mut overflow = false;
    let mut buffer = [0; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            return Ok((output, overflow));
        }
        let keep = count.min(limit.saturating_sub(output.len()));
        output.extend_from_slice(&buffer[..keep]);
        overflow |= keep < count;
        // Continue draining excess bytes so the child can always exit.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn installed_language_parser_ignores_header_osd_and_duplicates() {
        let output = "List of available languages in \"/usr/share/tessdata\" (5):\nkor\neng\nosd\nscript/Latin\neng\n";
        assert_eq!(parse_languages(output).unwrap(), ["eng", "kor", "script/Latin"]);
        assert!(parse_languages("List of available languages (1):\nosd\n").is_err());
    }

    #[test]
    fn locale_default_always_uses_an_installed_language() {
        let languages: Vec<_> = ["chi_sim", "chi_tra", "eng", "kor"].map(String::from).to_vec();
        assert_eq!(default_language(&languages, "ko_KR.UTF-8"), "kor");
        assert_eq!(default_language(&languages, "zh-Hant-TW"), "chi_tra");
        assert_eq!(default_language(&languages, "zh_CN.UTF-8"), "chi_sim");
        assert_eq!(default_language(&languages, "unsupported"), "eng");
        assert_eq!(default_language(&["kor".into()], "en_US.UTF-8"), "kor");
    }

    #[test]
    fn oversized_pipe_output_is_drained_with_bounded_storage() {
        let (output, overflow) = read_limited(io::Cursor::new(vec![1; 20_000]), 1024).unwrap();
        assert_eq!(output, vec![1; 1024]);
        assert!(overflow);
    }

    #[test]
    fn missing_engine_reports_installation_instructions() {
        let error = run_command(
            Path::new("/no-such-ocr-test-directory/tesseract"),
            &["--list-langs"],
            None,
            LANGUAGE_TIMEOUT,
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("Install Tesseract"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn pipes_are_drained_concurrently_and_errors_take_priority_over_broken_stdin() {
        let output = run_command(
            Path::new("/bin/sh"),
            &["-c", "head -c 131072 /dev/zero >&2; cat"],
            Some(vec![42; 512 * 1024]),
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(output, vec![42; 512 * 1024]);
        let error = run_command(
            Path::new("/bin/sh"),
            &["-c", "printf 'language pack is missing' >&2; exit 4"],
            Some(vec![42; 512 * 1024]),
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert!(format!("{error:#}").contains("language pack is missing"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn timed_out_process_is_killed_and_reaped_before_returning() {
        let start = Instant::now();
        let error =
            run_command(Path::new("/bin/sh"), &["-c", "exec sleep 5"], None, Duration::from_millis(60)).unwrap_err();
        assert!(error.to_string().contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(2));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "Requires a system Tesseract executable and English language data"]
    fn tesseract_recognizes_the_rgba_fixture_and_returns_empty_for_a_blank_image() {
        let mut backend = Backend::new().unwrap();
        let (languages, language, _) = backend.available_languages().unwrap();
        assert!(languages.iter().any(|candidate| candidate.tag == language));
        assert!(backend.languages.iter().any(|tag| tag == "eng"));
        let image = image::load_from_memory(include_bytes!("testdata/english.png"))
            .unwrap()
            .to_rgba8();
        let text = backend
            .recognize(OcrJob {
                width: image.width(),
                height: image.height(),
                rgba: image.into_raw(),
                language: "eng".into(),
            })
            .unwrap();
        assert!(text.contains("MAC OCR 123"), "Unexpected OCR result: {text:?}");
        let text = backend
            .recognize(OcrJob {
                width: 200,
                height: 100,
                rgba: vec![255; 200 * 100 * 4],
                language: "eng".into(),
            })
            .unwrap();
        assert!(text.is_empty(), "A scan must not reuse the previous image's text");
    }
}
