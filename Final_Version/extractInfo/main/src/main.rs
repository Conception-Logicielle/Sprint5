use std::{
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use rayon::prelude::*;
use regex::Regex;

#[derive(Debug)]
struct ArticleData {
    filename: String,
    title: String,
    authors: String,
    abstract_text: String,
    introduction: String,
    body: String,
    conclusion: String,
    discussion: String,
    bibliography: String,
}

/// Toutes les regex utilisées sont compilées une seule fois ici.
pub struct RegexSet {
    pub metadata: Regex,
    pub likely_author: Regex,
    pub name_pair: Regex,
    pub multiple_names: Regex,
    pub bad_header: Regex,
    pub numeric_line: Regex,
    pub body_like_line: Regex,
    pub contains_abstract: Regex,
    pub introduction_header: Regex,
    pub new_section: Regex,
}

impl RegexSet {
    pub fn new() -> Self {
        Self {
            metadata: Regex::new(r"(?i)(conference|volume|doi|issn|copyright|journal|published|©)").unwrap(), // detecte les entêtes génériques
            likely_author: Regex::new(r"(?ix)^((?:[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)(?:[\d†*]*)\s*(?:,\s*|\s+and\s+|\s+et\s+)?)+$").unwrap(), // detecte les noms d'auteurs qui sont formatés comme "Nom Prénom" ou "Nom Prénom Nom Prénom"
            name_pair: Regex::new(r"^[A-Z][a-z]+(\s+[A-Z][a-z]{2,})([\d†*∗°]*)$").unwrap(), // detecte les noms d'auteurs qui sont formatés comme "Nom Prénom" ou "Nom Prénom Nom Prénom"
            multiple_names: Regex::new(r"(?i)([A-Z][a-z]+(?:\s+[A-Z]\.?)?\s+[A-Z][a-z]+(?:\d*)\s*(,|and|et)\s*){1,}").unwrap(), // detecte les noms d'auteurs qui sont formatés comme "Nom Prénom" ou "Nom Prénom Nom Prénom"
            bad_header: Regex::new(r"(?i)(journal|volume|submitted|published|copyright|doi|issn|arxiv|^\s*\d{2}/\d{2})").unwrap(), // detecte les entêtes génériques
            numeric_line: Regex::new(r"^[\d\s/;,\(\)\-]+$").unwrap(), // detecte les lignes qui ne contiennent que des chiffres ou des caractères de ponctuation
            introduction_header: Regex::new(r"(?ix)^\s*(introduction| i+[\.\)]?\s*(introduction|I\s*N\s*T\s*R\s*O\s*D\s*U\s*C\s*T\s*I\s*O\s*N)| \d+[\.\)]?\s*introduction)\b").unwrap(),
            body_like_line: Regex::new(r#"(?i)^[a-z][a-z\s,;\-\(\)\[\]\.:'"0-9]+$"#).unwrap(), // detecte les lignes qui ressemblent à du texte normal
            contains_abstract: Regex::new(r"(?i)\babstract\b").unwrap(), // detecte les lignes qui contiennent le mot "abstract"
        }
    }
}


/// Extrait le titre de l'article à partir des premières lignes du fichier texte.
/// Critères utilisés pour détecter un titre :
///
/// 1. Ignore les lignes vides, contenant des emails ou correspondant à des entêtes génériques
///    (journal, volume, dates de soumission/publication, etc.) via `regex.bad_header`.
/// 2. Ignore également les lignes uniquement numériques ou ponctuelles via `regex.numeric_line`.
/// 3. Prend la première ligne non filtrée comme début du titre.
/// 4. Ajoute la ligne suivante au titre si elle ne correspond pas à un motif  de liste
/// d'auteur (and, ",") ou présence d’email/université).
/// 5. Retourne le titre concaténé (sur 1 ou 2 lignes max) ainsi que l’index de fin du titre.
fn extract_title(lines: &[String], regex: &RegexSet) -> Option<(String, usize)> {
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // On saute les lignes inutiles détectées par regex ou qui contiennent des emails
        if line.is_empty() || regex.bad_header.is_match(line) || line.contains('@') {
            i += 1;
            continue;
        }

        // Lignes purement numériques ou ponctuelles (dates, numéros)
        if Regex::new(r"^[\d\s/;,\(\)\-]+$").unwrap().is_match(line) {
            i += 1;
            continue;
        }

        break;
    }

    if i >= lines.len() {
        return None;
    }

    let mut title = lines[i].trim().to_string();
    let mut end_index = i + 1;

    if i + 1 < lines.len() {
        let next = lines[i + 1].trim();

        let is_likely_author = next.contains("and")
            || next.contains(",")
            || next.contains('@')
            || next.to_lowercase().contains("university");

        if !is_likely_author {
            title.push(' ');
            title.push_str(next);
            end_index = i + 2;
        }
    }

    Some((title.trim().to_string(), end_index))
}

/// Extrait la section des auteurs immédiatement après le titre.
/// Cette fonction utilise des expressions régulières centralisées pour :
/// - ignorer les lignes vides,
/// - s'arrêter dès qu'une section connue commence (Abstract, Introduction, etc.),
/// - s'arrêter si une ligne typique du corps de texte est rencontrée (phrase normale),
/// - inclure les lignes contenant des noms, affiliations ou emails,
/// - concaténer toutes les lignes pertinentes en une seule chaîne.
///
fn extract_authors(lines: &[String], start_after_title: usize, regex: &RegexSet) -> String {
    let mut authors = String::new();
    let mut started = false;

    for line in lines.iter().skip(start_after_title) {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Si on tombe sur une section explicite (Abstract, Introduction, etc.)
        if regex.contains_abstract.is_match(trimmed) || regex.introduction_header.is_match(trimmed) {
            break;
        }

        // Si la ligne ressemble à une phrase du corps de texte, on s’arrête
        if (regex.body_like_line.is_match(trimmed) || regex.contains_abstract.is_match(trimmed)) && started {
            break;
        }

        if !authors.is_empty() {
            authors.push(' ');
        }

        authors.push_str(trimmed);
        started = true;
    }

    authors
}


/// Extrait le résumé (abstract) d’un article à partir de son contenu texte.
///
/// Critères :
/// - Débute sur une ligne correspondant à un en-tête de section "abstract" (regex.contains_abstract).
/// - Nettoie le préfixe (e.g. "Abstract —", "Résumé:", etc.)
/// - S’arrête sur une ligne qui est une section d’introduction (regex.introduction_header).
/// - Ou sur un début de corps (`regex.body_like_line`) après détection
/// - Fournit une ligne longue alternative si aucun abstract explicite n’est trouvé
fn extract_abstract(lines: &[String], regex: &RegexSet) -> String {
    let mut abstract_lines = Vec::new();
    let mut in_abstract = false;
    let mut fallback: Option<String> = None;

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let lower = trimmed.to_lowercase();

        // Détection de début d’abstract
        if !in_abstract && regex.contains_abstract.is_match(trimmed) {
            in_abstract = true;

            // Nettoyage du préfixe (e.g. "Abstract:", "Résumé -")
            let cleaned = regex
                .contains_abstract
                .replace(&lower, "")
                .replace([':', '-', '–', '—'], "")
                .trim()
                .to_string();

            if cleaned.split_whitespace().count() > 2 {
                abstract_lines.push(cleaned);
            }
            i += 1;
            continue;
        }

        if in_abstract {
            // Fin explicite si nouvelle section
            if regex.introduction_header.is_match(trimmed) {
                break;
            }

            abstract_lines.push(trimmed.to_string());
        }

        // Fallback : première ligne longue qui ne semble pas être un titre
        if !in_abstract && fallback.is_none() {
            if trimmed.len() > 100 && !regex.introduction_header.is_match(trimmed) {
                fallback = Some(trimmed.to_string());
            }
        }

        i += 1;
    }

    if !abstract_lines.is_empty() {
        abstract_lines.join(" ").replace("  ", " ").trim().to_string()
    } else {
        fallback.unwrap_or_default()
    }
}


/// Extrait le contenu de la section Introduction d’un article.
///
/// Recherche la section `introduction` après l'abstract et récupère son contenu,
/// jusqu’à une heuristique de fin : ligne vide suivie d’un nouveau titre, nouvelle section,
/// ou structure numérotée inline.
fn extract_introduction(lines: &[String], abstract_text: &str) -> (String, usize) {
    use regex::Regex;

    let intro_regex = Regex::new(r"(?i)^(\d+\.?|[ivxlc]+\.?)?\s*introduction\s*$").unwrap();
    let numbered_section_inline = Regex::new(r"(?i)^\s*(\d+|[ivxlc]+)\.?\s+[A-Za-z].+").unwrap();
    let section_number_only = Regex::new(r"^\s*(\d+|[ivxlc]+)\s*$").unwrap();
    let capitalized_line = Regex::new(r"^[A-Z][a-zA-Z\s\-]{3,}$").unwrap();
    let hardcoded_stop_phrase = "A noisy-channel model for sentence";

    fn is_uppercase_section_title(line: &str) -> bool {
        let without_number = line.trim_start()
            .trim_start_matches(|c: char| c.is_ascii_alphanumeric() || c == '.' || c == ' ');
        let cleaned: String = without_number.chars().filter(|c| !c.is_whitespace()).collect();
        let total = cleaned.len();
        let uppercase_count = cleaned.chars().filter(|c| c.is_uppercase()).count();
        total > 5 && uppercase_count as f32 / total as f32 > 0.8
    }

    fn looks_like_intro_heading(line: &str) -> bool {
        let stripped = line
            .trim()
            .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == 'I' || c == 'i')
            .replace(char::is_whitespace, "");

        let cleaned = stripped.to_ascii_lowercase();
        cleaned.contains("introduction") || cleaned == "ntroduction"
    }

    let mut intro_lines = Vec::new();
    let mut in_intro = false;
    let mut abstract_end_index = 0;

    if let Some(first_line) = abstract_text.lines().next() {
        for (i, line) in lines.iter().enumerate() {
            if line.contains(first_line.trim()) {
                abstract_end_index = i;
                break;
            }
        }
    }

    let mut i = abstract_end_index;
    let mut char_offset = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if !in_intro {
            if intro_regex.is_match(trimmed) || looks_like_intro_heading(trimmed) {
                in_intro = true;
                i += 1;
                continue;
            }
        } else {
            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            if trimmed.contains(hardcoded_stop_phrase) {
                break;
            }

            if i + 2 < lines.len()
                && section_number_only.is_match(trimmed)
                && lines[i + 1].trim().is_empty()
                && capitalized_line.is_match(lines[i + 2].trim())
            {
                break;
            }

            if numbered_section_inline.is_match(trimmed) {
                break;
            }

            if is_uppercase_section_title(trimmed) {
                break;
            }

            intro_lines.push(trimmed.to_string());

            if trimmed.ends_with('.') && i + 1 < lines.len() && lines[i + 1].trim().is_empty() {
                i += 1;
                break;
            }
        }

        i += 1;
    }

    for line in lines.iter().take(i) {
        char_offset += line.len() + 1;
    }

    (
        intro_lines.join(" ").replace("  ", " ").trim().to_string(),
        char_offset,
    )
}

/// Extrait le corps de l'article.
/// Critères utilisés pour détecter le corps (<corps>) :
///
/// 1. Commence juste après la fin de l’introduction (`intro_char_end`).
/// 2. Inclut toutes les lignes suivantes jusqu’à une section finale détectée.
/// 3. S’arrête à la première ligne correspondant à un titre de fin :
///    "discussion", "experiments", "conclusion", "acknowledgment", "references", etc.
/// 4. Les titres de fin sont détectés même avec numérotation ou variations typographiques.
/// 5. Ignore les caractères de contrôle (\x0c, \r, etc.) pour fiabiliser la détection.
/// 6. Le corps correspond donc à toutes les sections thématiques centrales de l’article.
fn extract_body(lines: &[String], intro_char_end: usize) -> (String, usize) {
    use regex::Regex;

    let end_section_regex = Regex::new(
        r"(?i)^\s*(\d+\.?|[ivxlc]+\.?)?\s*(discussion|experiments|conclusion|conclusions|concluding remarks|future work|acknowledg(?:ment|ement)|references|bibliography)(\s+.*)?"
    ).unwrap();

    fn normalize_line(text: &str) -> String {
        text
            .chars()
            .filter(|c| !c.is_control()) // enlève \x0c, \r, etc.
            .fold((String::new(), false), |(mut acc, mut prev_letter), c| {
                if c.is_whitespace() {
                    if prev_letter {
                        prev_letter = false;
                    }
                } else {
                    if c.is_alphabetic() {
                        prev_letter = true;
                    } else {
                        prev_letter = false;
                    }
                    acc.push(c);
                }
                (acc, prev_letter)
            })
            .0
    }

    let mut char_count = 0;
    let mut start_index = None;
    let mut end_index = lines.len();

    for (i, line) in lines.iter().enumerate() {
        char_count += line.len() + 1;

        if start_index.is_none() && char_count >= intro_char_end {
            start_index = Some(i);
        }

        if let Some(_) = start_index {
            let cleaned = normalize_line(line);
            if end_section_regex.is_match(&cleaned) {
                end_index = i;
                break;
            }
        }
    }

    let body_lines = &lines[start_index.unwrap_or(0)..end_index];
    let body_text = body_lines.join("\n");
    let body_char_end = lines.iter().take(end_index-2).map(|l| l.len()).sum();

    (body_text.trim().to_string(), body_char_end)
}

/// Extrait la conclusion de l'article.
/// Critères utilisés pour détecter une conclusion (<conclusion>) :
///
/// 1. Commence après la fin du corps (`body_char_end`).
/// 2. Débute à la première ligne correspondant à un titre de section tel que :
///    "conclusion", "conclusions", "concluding remarks", ou "future work".
/// 3. Ces titres peuvent être précédés d’une numérotation (ex. "5.", "V.", etc.).
/// 4. Se termine à la première ligne contenant "references", "bibliography" ou "acknowledgment".
/// 5. Ignore les espaces et caractères non alphabétiques pour fiabiliser la détection.
/// 6. Retourne toutes les lignes comprises entre les deux bornes.
fn extract_conclusion(lines: &[String], body_char_end: usize) -> (String, usize) {
    use regex::Regex;

    let start_regex = Regex::new(
        r"(?i)^\s*(\d+\.?|[ivxlc]+\.?)?\s*(conclusion|conclusions|concluding remarks|future work)(\s+.*)?"
    ).unwrap();

    let end_regex = Regex::new(
        r"(?i)^\s*(references|bibliography|acknowledg(?:ment|ement))\b"
    ).unwrap();

    fn normalize_line(text: &str) -> String {
        text.chars()
            .fold((String::new(), false), |(mut acc, mut prev_alpha), c| {
                if c.is_whitespace() {
                    if prev_alpha {
                        prev_alpha = false;
                    }
                } else {
                    if c.is_alphabetic() {
                        prev_alpha = true;
                    } else {
                        prev_alpha = false;
                    }
                    acc.push(c);
                }
                (acc, prev_alpha)
            })
            .0
    }

    let mut char_count = 0;
    let mut start_line = 0;
    for (i, line) in lines.iter().enumerate() {
        char_count += line.len() + 1;
        if char_count >= body_char_end {
            start_line = i;
            break;
        }
    }

    let mut start_index = None;
    let mut end_index = lines.len();

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        let normalized = normalize_line(line.trim());

        if start_index.is_none() {
            if start_regex.is_match(&normalized) {
                start_index = Some(i);
            }
        } else {
            if end_regex.is_match(&normalized) {
                end_index = i;
                break;
            }
        }
    }

    let Some(start) = start_index else {
        return (String::new(), 0);
    };

    let conclusion_lines = &lines[start..end_index];
    let conclusion_text = conclusion_lines.join("\n");
    let conclusion_char_end = lines.iter().take(end_index).map(|l| l.len() + 1).sum();

    (conclusion_text.trim().to_string(), conclusion_char_end)
}

/// Extrait la discussion de l'article.
/// Critères utilisés pour détecter une discussion (<discussion>) :
///
/// 1. Commence après la fin du corps (`body_char_end`).
/// 2. Débute à la première ligne contenant un titre comme :
///    "discussion", "results and discussion", "discussion and conclusion", etc.
/// 3. S’arrête dès qu’une ligne correspond à un autre titre de section finale :
///    "conclusion", "future work", "references", "acknowledgment", etc.
/// 4. Prend en compte les variantes typographiques avec ou sans majuscules.
/// 5. Ignore les caractères non alphabétiques pour une détection plus robuste.
/// 6. Retourne toutes les lignes entre le début et la fin de la discussion.
fn extract_discussion(lines: &[String], body_char_end: usize) -> (String, usize) {
    use regex::Regex;

    let start_regex = Regex::new(
        r"(?i)\b(discussion|results and discussion|discussion and conclusion|discussion and future work)\b"
    ).unwrap();

    let end_regex = Regex::new(
        r"(?i)\b(conclusion|conclusions|concluding remarks|future work|references|bibliography|acknowledg(?:ment|ement))\b"
    ).unwrap();

    fn normalize_line(text: &str) -> String {
        text.chars()
            .fold((String::new(), false), |(mut acc, mut prev_alpha), c| {
                if c.is_whitespace() {
                    if prev_alpha {
                        prev_alpha = false;
                    }
                } else {
                    if c.is_alphabetic() {
                        prev_alpha = true;
                    } else {
                        prev_alpha = false;
                    }
                    acc.push(c);
                }
                (acc, prev_alpha)
            })
            .0
    }


    let mut char_count = 0;
    let mut start_line = 0;
    for (i, line) in lines.iter().enumerate() {
        char_count += line.len() + 1;
        if char_count >= body_char_end {
            start_line = i;
            break;
        }
    }

    let mut start_index = None;
    let mut end_index = lines.len();

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        let normalized = normalize_line(line);

        if start_index.is_none() {
            if start_regex.is_match(&normalized) || start_regex.is_match(&line) {
                start_index = Some(i);
            }
        } else {
            if end_regex.is_match(&normalized) || end_regex.is_match(&line) {
                end_index = i;
                break;
            }
        }
    }

    let Some(start) = start_index else {
        return (String::new(), 0);
    };

    let discussion_lines = &lines[start..end_index];
    let discussion_text = discussion_lines.join("\n");
    let discussion_char_end = lines.iter().take(end_index).map(|l| l.len() + 1).sum();

    (discussion_text.trim().to_string(), discussion_char_end)
}

/// Extrait la bibliographie de l'article.
/// Critères utilisés pour détecter une bibliographie (<biblio>) :
///
/// 1. Recherche une ligne contenant exactement "references", "références" ou "bibliography".
/// 2. Ignore les espaces (ex. "R E F E R E N C E S").
/// 3. La détection est insensible à la casse et aux caractères non alphanumériques.
/// 4. Commence juste après cette ligne repérée.
/// 5. Considère que toutes les lignes suivantes appartiennent à la bibliographie.
fn extract_bibliography(lines: &[String], _body_char_end: usize) -> String {
    use regex::Regex;

    let start_regex = Regex::new(
        r"(?ix)^\s*(r\s*e\s*f\s*e\s*r\s*e\s*n\s*c\s*e\s*s|références|bibliography)\s*$"
    ).unwrap();

    let mut start_index = None;
    for (i, line) in lines.iter().enumerate() {
        let normalized = line
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>()
            .to_lowercase()
            .trim()
            .to_string();

        if start_regex.is_match(&normalized) {
            start_index = Some(i + 1); // commence après le titre
            break;
        }
    }

    let start = match start_index {
        Some(i) => i,
        None => return String::from("Aucune bibliographie trouvée."),
    };

    let biblio_lines = &lines[start..];
    biblio_lines.join("\n").trim().to_string()
}


/// Extrait les champs disponibles de l'article.
fn extract_article_fields(path: &Path, regex: &RegexSet) -> io::Result<ArticleData> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().filter_map(Result::ok).collect::<Vec<_>>();

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().replace(' ', "_"))
        .unwrap_or_else(|| "unknown_file".to_string());

    if let Some((title, title_end_index)) = extract_title(&lines, &regex) {
        let authors = extract_authors(&lines, title_end_index, &regex);
        let abstract_text = extract_abstract(&lines , &regex);
        let (introduction, intro_char_end) = extract_introduction(&lines, &abstract_text);
        let (body, body_char_end) = extract_body(&lines, intro_char_end);
        let (temp_conclusion, _) = extract_conclusion(&lines, body_char_end);
        let mut conclusion: String;
        if temp_conclusion.is_empty() {
            conclusion = "Aucune conclusion trouvée.".clone().to_string();
        } else {
            conclusion = temp_conclusion.clone();
        };
        let (tempdiscussion, discutionend) = extract_discussion(&lines, body_char_end);
        let mut discussion: String;
        if tempdiscussion.is_empty() {
            discussion = "Aucune discussion trouvée.".clone().to_string();
        } else {
            discussion = tempdiscussion.clone();
        };
        let bibliography = extract_bibliography(&lines, body_char_end);
        println!("{}: {}", filename, body_char_end);


        Ok(ArticleData {
            filename,
            title,
            authors,
            abstract_text,
            introduction,
            body,
            conclusion ,
            discussion ,
            bibliography ,
        })
    }
    else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Titre introuvable dans le fichier : {:?}", path),
        ))
    }
}

/// Écrit les articles en XML.
fn write_combined_xml(path: &Path, articles: &[ArticleData]) -> io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "<articles>")?;

    for article in articles {
        writeln!(
            file,
            "\t<article>\n\
            \t\t<preamble>{}</preamble>\n\
            \t\t<titre>{}</titre>\n\
            \t\t<auteur>{}</auteur>\n\
            \t\t<abstract>{}</abstract>\n\
            \t\t<introduction>{}</introduction>\n\
            \t\t<corps>{}</corps>\n\
            \t\t<conclusion>{}</conclusion>\n\
            \t\t<discussion>{}</discussion>\n\
            \t\t<biblio>{}</biblio>\n\
            \t</article>",
            article.filename,
            article.title,
            article.authors,
            article.abstract_text,
            article.introduction,
            article.body,
            article.conclusion,
            article.discussion,
            article.bibliography
        )?;
    }

    writeln!(file, "</articles>")?;
    Ok(())
}

/// Écrit les résumés textuels.
fn write_txt_summaries(path: &Path, articles: &[ArticleData], duration_total: u128) -> io::Result<()> {
    let mut file = File::create(path)?;

    for article in articles {
        let total_len = article.abstract_text.len()
            + article.introduction.len()
            + article.body.len()
            + article.discussion.len()
            + article.conclusion.len();

        writeln!(
            file,
            "==============================\n\
             Fichier        : {}\n\
             Titre          : {}\n\
             Auteurs        : {}\n\
             Résumé         : {}\n\
             Introduction   : {}\n\
             Développement  : {}\n\
             Discussion     : {}\n\
             Conclusion     : {}\n\
             Références     : {}\n\
             Longueur texte : {} caractères\n",
            article.filename,
            article.title,
            article.authors,
            article.abstract_text,
            article.introduction,
            article.body,
            article.discussion,
            article.conclusion,
            article.bibliography,
            total_len
        )?;
    }

    writeln!(file, "==============================")?;
    writeln!(file, "Traitement terminé en {} ms", duration_total)?;
    Ok(())
}

/// Fonction principale.
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <input_folder> <output_folder> <mode: txt|xml>", args[0]);
        std::process::exit(1);
    }

    let input_folder = Path::new(&args[1]);
    let output_folder = Path::new(&args[2]);
    let mode = &args[3].to_lowercase();
    fs::create_dir_all(output_folder)?;

    let start_all = Instant::now();
    let regex = RegexSet::new();

    let entries: Vec<_> = fs::read_dir(input_folder)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("txt"))
        .collect();

    let articles: Vec<_> = entries
        .par_iter()
        .filter_map(|path| extract_article_fields(path, &regex).ok())
        .collect();

    match mode.as_str() {
        "xml" => write_combined_xml(&output_folder.join("articles.xml"), &articles)?,
        "txt" => {
            let elapsed = start_all.elapsed().as_millis();
            write_txt_summaries(&output_folder.join("resumes.txt"), &articles, elapsed)?;
        }
        _ => {
            eprintln!("Mode invalide : {}. Utilisez 'txt' ou 'xml'.", mode);
            std::process::exit(1);
        }
    }

    println!(
        "Extraction réussie en mode {}. Temps total : {} ms",
        mode,
        start_all.elapsed().as_millis()
    );

    Ok(())
}