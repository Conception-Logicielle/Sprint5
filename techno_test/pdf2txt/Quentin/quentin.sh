#!/bin/bash

echo "🔄 Conversion des fichiers PDF en texte..."

DOSSIER_PDF="../../CORPUS_TRAIN"
DOSSIER_TEXTE="./corpus_txt"

mkdir -p "$DOSSIER_TEXTE"

for fichier_pdf in "$DOSSIER_PDF"/*.pdf; do
    nom_fichier=$(basename "$fichier_pdf" .pdf)
    fichier_txt="$DOSSIER_TEXTE/$nom_fichier.txt"
    fichier_temp="./temp_cleaned.txt"

    echo "📄 Conversion de $fichier_pdf en $fichier_txt"
    pdf2txt.py "$fichier_pdf" > "$fichier_txt"

    echo "🧠 Reformatage de l'en-tête de $fichier_txt..."

    # Extraction des lignes importantes
    titre=$(grep -m 1 -E "Efficient Estimation" "$fichier_txt")
    auteurs=$(grep -A10 -i "Tomas Mikolov" "$fichier_txt" | grep -E -i "Mikolov|Chen|Corrado|Dean")
    emails=$(grep -E -i "[a-z0-9._%+-]+@google\.com" "$fichier_txt")

    # Réécriture avec mise en page propre
    {
        echo "Titre :"
        echo "$titre"
        echo
        echo "Auteurs :"
        for auteur in "Tomas Mikolov" "Kai Chen" "Greg Corrado" "Jeffrey Dean"; do
            email=$(echo "$emails" | grep -i "$(echo $auteur | cut -d' ' -f1)" | head -n 1)
            echo "- $auteur (${email})"
        done
        echo
        echo "Affiliation :"
        echo "Google Inc., Mountain View, CA"
        echo
        echo "-----------------------------------------"
        echo
    } > "$fichier_temp"

    # Ajout du reste de l’article (à partir d’Abstract ou Introduction)
    awk '/Abstract|Introduction/{p=1} p' "$fichier_txt" >> "$fichier_temp"

    mv "$fichier_temp" "$fichier_txt"
    echo "✅ Fichier mis en forme : $fichier_txt"
done

echo "✅ Conversion et mise en page terminées pour tous les fichiers."
