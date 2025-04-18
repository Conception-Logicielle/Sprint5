#!/bin/bash

echo "🔄 Conversion des fichiers PDF en texte avec pdftotext..."

DOSSIER_PDF="../CORPUS_TRAIN"
DOSSIER_TEXTE="./corpus_txt"

mkdir -p "$DOSSIER_TEXTE"

# Vérifie si pdftotext est installé
if ! command -v pdftotext &> /dev/null; then
    echo "❌ Erreur : 'pdftotext' est introuvable. Installe-le avec : sudo apt install poppler-utils"
    exit 1
fi

for fichier_pdf in "$DOSSIER_PDF"/*.pdf; do
    nom_fichier=$(basename "$fichier_pdf" .pdf)
    fichier_txt="$DOSSIER_TEXTE/$nom_fichier.txt"
    fichier_temp="./temp_cleaned.txt"

    echo "📄 Conversion de $fichier_pdf en $fichier_txt"
    pdftotext -layout -enc UTF-8 -nopgbrk -eol unix "$fichier_pdf" "$fichier_txt"

    if [ $? -ne 0 ]; then
        echo "❌ Erreur lors de la conversion de $fichier_pdf"
        continue
    fi

    echo "🧠 fichier $fichier_txt convertit"

done

echo "✅ Conversion et mise en page terminées pour tous les fichiers."