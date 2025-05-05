#!/bin/bash

echo "🔄 Conversion des fichiers PDF en texte avec pdftotext.sh..."

DOSSIER_PDF="../CORPUS_TRAIN"
DOSSIER_TEXTE="./corpus_txt"

mkdir -p "$DOSSIER_TEXTE"

if [ ! -x ./pdftotext.sh ]; then
    echo "❌ Le script pdftotext.sh est introuvable ou non exécutable."
    exit 1
fi



# Appel du script pdftotext.sh pour chaque fichier PDF
for fichier_pdf in "$DOSSIER_PDF"/*.pdf; do
    echo "📄 Conversion de $fichier_pdf"
    ./pdftotext.sh "$fichier_pdf" "$DOSSIER_TEXTE"

    if [ $? -ne 0 ]; then
        echo "❌ Erreur lors de la conversion de $fichier_pdf"
        continue
    fi

    echo "🧠 fichier $fichier_pdf convertit"
done

echo "✅ Conversion et mise en page terminées pour tous les fichiers."

echo "🔄 Génération des fichier de résumés..."
DOSSIER_RESUMES="./resumes"

cd extractInfo/main

cargo run --release ../../corpus_txt ../../resume
