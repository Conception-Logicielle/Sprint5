#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <fichier_source.txt> <fichier_sortie.xml>"
  exit 1
fi

if ! command -v xmllint >/dev/null 2>&1; then
  echo "[ERROR] xmllint n'est pas installé. Veuillez installer libxml2." >&2
  exit 1
fi

TXT_FILE="$1"
XML_FILE="$2"

if [ ! -f "$TXT_FILE" ]; then
  echo "[ERROR] : Le fichier source est introuvable : $TXT_FILE" >&2
  exit 1
fi

if [ ! -f "$XML_FILE" ]; then
  echo "[ERROR] : Le fichier XML est introuvable : $XML_FILE" >&2
  exit 1
fi

levenshtein() {
  awk -v s1="$1" -v s2="$2" '
  function min(a, b, c){ m=a; if(b < m) m=b; if(c < m) m=c; return m }
  BEGIN {
    n1 = length(s1); n2 = length(s2);
    for(i = 0; i <= n1; i++) d[i,0] = i;
    for(j = 0; j <= n2; j++) d[0,j] = j;
    for(i = 1; i <= n1; i++){
      for(j = 1; j <= n2; j++){
        cost = (substr(s1, i, 1) == substr(s2, j, 1)) ? 0 : 1;
        d[i,j] = min(d[i-1,j] + 1, d[i,j-1] + 1, d[i-1,j-1] + cost);
      }
    }
    print d[n1, n2]
  }'
}

similarity() {
  local a="$1" b="$2"
  local max=${#a}
  [ "${#b}" -gt "$max" ] && max=${#b}
  if [ "$max" -eq 0 ]; then
    echo 100
    return
  fi
  local dist
  dist=$(levenshtein "$a" "$b")
  echo $(( (max - dist) * 100 / max ))
}

extract_xml() {
  local tag="$1" file="$2"
  xmllint --xpath "string(//$tag)" "$file" 2>/dev/null || echo ""
}

normalize() {
  tr '[:upper:]' '[:lower:]' | tr -d '[:punct:]' | tr -s ' '
}

TEXT_CLEAN=$(<"$TXT_FILE" normalize)

echo "=== Vérification de $XML_FILE vs $TXT_FILE ==="

basename_xml=$(basename "$XML_FILE" .xml)
expected_fname="$basename_xml.pdf"
found_fname=$(extract_xml FileName "$XML_FILE")
expected_fname_clean=$(echo "$expected_fname" | normalize)
found_fname_clean=$(echo "$found_fname" | normalize)
score_fname=$(similarity "$expected_fname_clean" "$found_fname_clean")
printf "FileName : %3d%% (\"%s\"est attendu ,or \"%s\") a été trouvé\n" "$score_fname" "$expected_fname" "$found_fname"

found_title=$(extract_xml Title "$XML_FILE")
found_title_clean=$(echo "$found_title" | normalize)
if echo "$TEXT_CLEAN" | grep -qF "$found_title_clean"; then
  score_title=100
else
  score_title=$(similarity "$found_title_clean" "$TEXT_CLEAN")
fi
printf "Title    : %3d%% (\"%s\")\n" "$score_title" "$found_title"

found_auteur=$(extract_xml Auteur "$XML_FILE")
found_auteur_clean=$(echo "$found_auteur" | normalize)
if echo "$TEXT_CLEAN" | grep -qF "$found_auteur_clean"; then
  score_auteur=100
else
  score_auteur=$(similarity "$found_auteur_clean" "$TEXT_CLEAN")
fi
printf "Auteur   : %3d%% (\"%s\")\n" "$score_auteur" "$found_auteur"

found_abstract=$(extract_xml Abstract "$XML_FILE")
found_abstract_clean=$(echo "$found_abstract" | normalize)
if echo "$TEXT_CLEAN" | grep -qF "$found_abstract_clean"; then
  score_abstract=100
else
  score_abstract=$(similarity "$found_abstract_clean" "$TEXT_CLEAN")
fi
printf "Abstract : %3d%% (\"%.30s...\")\n" "$score_abstract" "$found_abstract"

total=$((score_fname + score_title + score_auteur + score_abstract))
score=$(( total / 4 ))
echo "----------------------------------------"
echo "Score global : $score/100"
