#!/bin/bash

if [ $# -lt 1 ]; then
  echo "Usage: $0 <chemin/vers/fichier1.pdf> [autres_fichiers.pdf...]"
  exit 1
fi

DOSSIER_TEXTE="./corpus_txt"
DOSSIER_RESUMES="./resumes"

mkdir -p "$DOSSIER_TEXTE" "$DOSSIER_RESUMES"

for fichier_pdf in "$@"; do
  if [[ ! -f "$fichier_pdf" ]]; then
    echo "Fichier introuvable: $fichier_pdf" >&2
    continue
  fi
  nom_fichier=$(basename "$fichier_pdf" .pdf)
  fichier_txt="$DOSSIER_TEXTE/$nom_fichier.txt"
  if [ -f "$fichier_txt" ]; then
    echo "[SKIP] $fichier_txt existe déjà."
    continue
  fi
  echo "[CONVERT] $fichier_pdf -> $fichier_txt"

  if ! ./pdftotext.sh "$fichier_pdf" "$fichier_texte"; then
    echo "[ERROR] conversion échouée pour $fichier_pdf" >&2
    continue
  fi
  echo "[OK] $fichier_pdf converti."
done

echo "--- Conversion terminée ---"

cd extractInfo/main || exit 1

MODE="txt"
for arg in "$@"; do
  if [[ "$arg" == "-x" ]]; then MODE="xml"; fi
done

echo "[SUMMARY] génération résumés en mode $MODE"
if ! cargo run --release ../../corpus_txt ../../resumes "$MODE"; then
  echo "[ERROR] échec génération résumés" >&2
  exit 1
fi

echo "--- Résumés générés dans $DOSSIER_RESUMES ---"
exit 0
