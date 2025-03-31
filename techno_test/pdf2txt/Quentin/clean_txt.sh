#!/bin/bash

FICHIER_TEXTE="$1"
FICHIER_TEMP="./temp_cleaned.txt"

# Mots-clés typiques pour détecter le vrai début
DEBUT_PATTERN="Efficient Estimation|Abstract|Introduction"

# Variables de contrôle
STARTED=0
> "$FICHIER_TEMP"

while IFS= read -r line; do
    if [[ $STARTED -eq 0 && "$line" =~ $DEBUT_PATTERN ]]; then
        STARTED=1
    fi

    if [[ $STARTED -eq 1 ]]; then
        echo "$line" >> "$FICHIER_TEMP"
    fi
done < "$FICHIER_TEXTE"

mv "$FICHIER_TEMP" "$FICHIER_TEXTE"
echo "✅ Nettoyage terminé : $FICHIER_TEXTE"
