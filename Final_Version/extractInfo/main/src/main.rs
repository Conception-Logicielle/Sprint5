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
struct RegexSet {
    metadata: Regex,
    likely_author: Regex,
    name_pair: Regex,
    multiple_names: Regex,
    bad_header: Regex,
}

impl RegexSet {
    fn new() -> Self {
        Self {
            metadata: Regex::new(r"(?i)(conference|volume|doi|issn|copyright|journal|published|©)").unwrap(),
            likely_author: Regex::new(r"(?ix)^((?:[A-Z][a-z]+(?:\s+[A-Z][a-z]+)+)(?:[\d†*]*)\s*(?:,\s*|\s+and\s+|\s+et\s+)?)+$").unwrap(),
            name_pair: Regex::new(r"^[A-Z][a-z]+(\s+[A-Z][a-z]{2,})([\d†*∗°]*)$").unwrap(),
            multiple_names: Regex::new(r"(?i)([A-Z][a-z]+(?:\s+[A-Z]\.?)?\s+[A-Z][a-z]+(?:\d*)\s*(,|and|et)\s*){1,}").unwrap(),
            bad_header: Regex::new(r"(?i)(journal|volume|submitted|published|copyright|doi|issn|arxiv|^[0-9]{2}/[0-9]{2})").unwrap(),
        }
    }
}

/// Détecte le titre de l'article dans les premières lignes du fichier.
/// Critères utilisés pour détecter un titre :
///
/// 1. Doit apparaître dans les premières lignes.
/// 2. Ignore les lignes détectées comme métadonnées (`regex.metadata`).
/// 3. Ligne valide si au moins 2 mots et majorité de mots avec majuscule.
/// 4. Continue tant qu’aucun motif d’auteur n’est détecté.
/// 5. S’arrête sur ligne vide après le début ou motif d’auteur.
fn extract_title(lines: &[String], regex: &RegexSet) -> Option<(String, usize)> {
    let mut i = 0;

    // Ignorer les en-têtes non pertinents
    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || regex.bad_header.is_match(line) || line.contains('@') {
            i += 1;
            continue;
        }

        // ignorer ligne si elle ressemble à un nom de journal
        if line.to_lowercase().contains("journal of")
            || line.to_lowercase().contains("volume")
            || line.to_lowercase().contains("submitted")
            || line.chars().all(|c| c.is_numeric() || c == '/' || c == ';' || c.is_whitespace()) {
            i += 1;
            continue;
        }

        break; // on a trouvé une première ligne candidate
    }

    if i >= lines.len() {
        return None;
    }

    let mut title = lines[i].trim().to_string();
    let mut end_index = i + 1;
    println!("Line {}: {}", i, title);
    if i + 1 < lines.len() {
        let next = lines[i + 1].trim();
        println!("Next line: {}", next);
        let is_author_like = next.contains(',') ||
            next.contains(" and ") ||
            next.to_lowercase().contains("university") ||
            next.contains('@');
        if !is_author_like {
            println!("c'est un titre");
            title.push(' ');
            title.push_str(next);
            end_index = i + 2;
        } else {
            println!("{} c'est pas un titre", next);
        }
    }

    Some((title.trim().to_string(), end_index))
}

/// Nettoie une chaîne de caractères contenant des noms ou affiliations.
fn clean_string(input: &str) -> String {
    let to_remove = ["1", "2", "†", "*", "∗", "\\", "  "];
    let mut cleaned = input.to_string();
    for item in to_remove {
        cleaned = cleaned.replace(item, " ");
    }
    cleaned.trim().to_string()
}

/// Extraite les auteurs de l'article.
/// Critères utilisés pour détecter les auteurs :
///
/// 1. Commence après la fin du titre (`start_after_title`).
/// 2. Ignore les lignes vides.
/// 3. S'arrête si la ligne contient des mots-clés d'affiliation ou un email.
/// 4. Cherche des segments contenant au moins deux mots et une majuscule initiale.
/// 5. Accepte les lignes si au moins un nom probable est détecté.
/// 6. Termine dès qu’aucun nom n’est trouvé après le début.
fn extract_authors(lines: &[String], start_after_title: usize) -> String {
    let mut authors = String::new();
    let mut started = false;

    for line in lines.iter().skip(start_after_title) {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();

        // Arrêt dès qu’on tombe sur le texte courant ou début de section
        if lower.contains("abstract")
            || lower.contains("introduction")
            || lower.starts_with("1 ") {
            break;
        }

        // Heuristique : si la ligne commence par une minuscule et ne contient ni @ ni majuscules significatives, c’est du texte
        let is_likely_text = {
            let starts_with_lower = trimmed
                .chars()
                .next()
                .map(|c| c.is_lowercase())
                .unwrap_or(false);
            let has_email = trimmed.contains('@');
            let has_many_uppers = trimmed
                .split_whitespace()
                .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
                .count() >= 2;
            starts_with_lower && !has_email && !has_many_uppers
        };

        if is_likely_text && started {
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

/// Extraite le résumé de l'article.
/// Critères utilisés pour détecter un abstract :
///
/// 1. Commence à la première ligne qui contient un mot-clé comme "abstract", "résumé", etc.
/// 2. Accepte les formes avec séparateur (`:` ou `—`) ou seul sur la ligne.
/// 3. Ignore les mots-clés initiaux dans la ligne pour ne garder que le contenu réel.
/// 4. Continue tant qu’aucun mot-clé de début de section n’est détecté (ex. "introduction").
/// 5. Arrête si une structure "1" + vide + "Introduction" est rencontrée.
/// 6. Si aucun abstract explicite n’est trouvé, utilise un paragraphe de fallback assez long.
fn extract_abstract(lines: &[String]) -> String {
    const START_KEYWORDS: &[&str] = &["abstract", "résumé", "summary", "executive summary"];
    const STOP_KEYWORDS: &[&str] = &[
        "introduction", "keywords", "index terms", "1.", "i.", "1 introduction", "1. introduction"
    ];

    let mut abstract_lines = Vec::new();
    let mut in_abstract = false;
    let mut fallback: Option<String> = None;

    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();
        let lower = trimmed.to_lowercase();

        // Début explicite
        if !in_abstract && START_KEYWORDS.iter().any(|kw| {
            lower.starts_with(&format!("{kw}—")) ||
                lower.starts_with(&format!("{kw}:")) ||
                lower == *kw
        }) {
            in_abstract = true;

            let mut cleaned = trimmed.to_string();
            for kw in START_KEYWORDS {
                for sep in ["—", ":", "-"] {
                    let prefix = format!("{kw}{sep}");
                    if cleaned.to_lowercase().starts_with(&prefix) {
                        cleaned = cleaned[prefix.len()..].trim_start().to_string();
                        break;
                    }
                }
            }

            if cleaned.split_whitespace().count() > 2 {
                abstract_lines.push(cleaned);
            }
            i += 1;
            continue;
        }

        if in_abstract {
            // Fin si structure : ligne avec "1", puis vide, puis "Introduction"
            if i + 2 < lines.len()
                && lines[i].trim() == "1"
                && lines[i + 1].trim().is_empty()
                && lines[i + 2].trim().to_lowercase().starts_with("introduction")
            {
                break;
            }

            // Fin si ligne de section
            if STOP_KEYWORDS.iter().any(|kw| lower.starts_with(kw)) {
                break;
            }

            abstract_lines.push(trimmed.to_string());
        }

        if !in_abstract && fallback.is_none() {
            if trimmed.len() > 100 && !lower.contains("introduction") {
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

/// Extrait l'introduction à partir des lignes, après le résumé
/// Critères utilisés pour détecter une introduction :
///
/// 1. Commence juste après la fin de l’abstract détecté.
/// 2. Débute à la ligne contenant le mot "Introduction" (avec ou sans numérotation).
/// 3. Reconnaît aussi les variantes typographiques ou fautes légères (ex. "1. Introduction", "I. INTRODUCTION").
/// 4. Termine si une section numérotée est détectée (ligne comme "2.", "II.", etc.).
/// 5. S’arrête si une ligne tout en majuscules ressemble à un titre de section.
/// 6. S’arrête si une phrase clé précise est rencontrée (ex. `hardcoded_stop_phrase`).
/// 7. Ignore les lignes vides pendant la lecture de l’introduction.
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
        let authors = extract_authors(&lines, title_end_index);
        let abstract_text = extract_abstract(&lines);
        let (introduction, intro_char_end) = extract_introduction(&lines, &abstract_text);
        let (body, body_char_end) = extract_body(&lines, intro_char_end);
        //println!("{}: {}", filename, body_char_end);
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
            abstract_text: String::new(),
            introduction: String::new(),
            body: String::new(),
            conclusion : String::new(),
            discussion : String::new(),
            bibliography : String::new(),
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