use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use rayon::prelude::*;

struct ArticleData {
    filename: String,
    title: String,
    authors: String,
    abstract_text: String,
    bibliography: String,
}

fn extract_article_fields(path: &Path) -> io::Result<ArticleData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let filename = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .replace(' ', "_");

    let mut title = String::with_capacity(256);
    let mut authors = String::with_capacity(256);
    let mut abstract_text = String::with_capacity(1024);
    let mut bibliography = String::with_capacity(1024);

    let mut section = 0; // 0=title, 1=authors, 2=abstract, 3=main, 4=biblio

    for line_result in reader.lines() {
        let Ok(mut line) = line_result else { continue };
        line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let lower = line.to_ascii_lowercase();

        if lower.contains("abstract") {
            section = 2;
            continue;
        }
        if lower.starts_with("1 ") || lower.starts_with("1.") || lower.starts_with("introduction") {
            section = 3;
            continue;
        }
        if lower.contains("references") || lower.contains("bibliography") {
            section = 4;
            continue;
        }

        match section {
            0 => {
                if line.contains('@')
                    || lower.contains("university")
                    || lower.contains("institute")
                    || lower.contains("school")
                {
                    section = 1;
                    authors.push_str(line.trim());
                    authors.push(' ');
                } else {
                    title.push_str(line.trim());
                    title.push(' ');
                }
            }
            1 => {
                authors.push_str(line.trim());
                authors.push(' ');
            }
            2 => {
                abstract_text.push_str(line.trim());
                abstract_text.push(' ');
            }
            4 => {
                bibliography.push_str(line.trim());
                bibliography.push(' ');
            }
            _ => {}
        }
    }

    Ok(ArticleData {
        filename,
        title: title.trim().to_owned(),
        authors: authors.trim().to_owned(),
        abstract_text: abstract_text.trim().to_owned(),
        bibliography: bibliography.trim().to_owned(),
    })
}

fn write_combined_xml(path: &Path, articles: &[ArticleData]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "<articles>")?;

    for article in articles {
        writeln!(file, "  <article>")?;
        writeln!(file, "    <preamble>{}</preamble>", article.filename)?;
        writeln!(file, "    <titre>{}</titre>", article.title)?;
        writeln!(file, "    <auteur>{}</auteur>", article.authors)?;
        writeln!(file, "    <abstract>{}</abstract>", article.abstract_text)?;
        writeln!(file, "    <biblio>{}</biblio>", article.bibliography)?;
        writeln!(file, "  </article>")?;
    }

    writeln!(file, "</articles>")?;
    Ok(())
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_folder> <output_folder> <mode: txt|xml>", args[0]);
        std::process::exit(1);
    }

    let input_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);
    let mode = &args[3];

    fs::create_dir_all(output_folder)?;

    let start_all = Instant::now();

    let entries: Vec<PathBuf> = fs::read_dir(input_folder)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("txt"))
        .collect();

    if mode == "txt" {
        let resumes: Vec<String> = entries
            .par_iter()
            .filter_map(|path| {
                let start = Instant::now();
                extract_article_fields(path).ok().map(|data| {
                    let duration = start.elapsed();
                    format!(
                        "==============================\n\
Fichier        : {}\n\
Titre          : {}\n\
Auteurs        : {}\n\
Résumé         : {}\n\
Références     : {}\n\
Longueur texte : {} caractères\n\
Temps analyse  : {} ms\n",
                        data.filename,
                        data.title,
                        data.authors,
                        data.abstract_text,
                        data.bibliography,
                        data.abstract_text.len(),
                        duration.as_millis()
                    )
                })
            })
            .collect();

        let mut file = File::create(output_folder.join("resumes.txt"))?;
        for resume in resumes {
            file.write_all(resume.as_bytes())?;
        }
        writeln!(file, "==============================")?;
        writeln!(file, "Traitement terminé en {} ms", start_all.elapsed().as_millis())?;
    } else {
        let articles: Vec<_> = entries
            .par_iter()
            .filter_map(|path| extract_article_fields(path).ok())
            .collect();
        write_combined_xml(&output_folder.join("articles.xml"), &articles)?;
    }

    println!(
        "Extraction réussie en mode {}. Temps total : {} ms",
        mode,
        start_all.elapsed().as_millis()
    );

    Ok(())
}
