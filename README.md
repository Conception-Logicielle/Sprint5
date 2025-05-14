# 📚 Parseur d'Articles Scientifiques en Texte (Sprint 4)

Ce projet a pour objectif d’extraire automatiquement les sections structurées (titre, auteurs, résumé, etc.) d’articles scientifiques en format PDF. Il convertit les PDF en texte brut via `pdftotext`, puis utilise un parseur écrit en **Rust** pour produire des résumés structurés au format **texte** ou **XML**.

---

## 🧰 Prérequis

Avant toute utilisation, assurez-vous que votre environnement dispose des outils suivants :

- Système Linux (ou WSL sous Windows)
- Bash + Zenity (pour une interface graphique de sélection de fichiers)
  ```bash
  sudo apt install zenity
  ```
- poppler-utils (nécessaire pour `pdftotext`)
  ```bash
  sudo apt install poppler-utils
  ```
- Python 
  ```bash
  sudo apt install python3 python3-tk
  ```
- Rust + Cargo (compilateur et gestionnaire de paquets Rust)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

---

## 📁 Structure du projet

```
- CORPUS_TRAIN/            # (Optionnel) Dossier initial de fichiers PDF
- Final_Version/
  ├── main.sh              # Script principal à exécuter
  ├── pdftotext.sh         # Conversion PDF → texte brut via pdftotext
  ├── corpus_txt/          # Textes extraits depuis les PDF
  ├── resumes/             # Résumés générés (texte ou XML)
  └── extractInfo/
       └── main/           # Code Rust (main.rs, Cargo.toml, etc.)
```

> 💡 Les fichiers PDF à traiter peuvent être sélectionnés manuellement depuis n’importe quel emplacement.

---

## 🚀 Utilisation

1. Assurez-vous que toutes les dépendances sont installées, notamment :

   * Python 3
   * `tkinter` (inclus dans la plupart des distributions Python)
   * `Pillow` (pour l'affichage d'image) :

     ```bash
     pip install pillow
     ```

2. Lancer l’interface graphique :

   ```bash
   python3 interface.py
   ```

3. Depuis l’interface :

   * Cliquez sur **📂 Parcourir** pour sélectionner un ou plusieurs fichiers PDF.
   * Choisissez le **mode de sortie** (`txt` ou `xml`).
   * Cliquez sur **⚙️ Convertir & Résumer** pour lancer le processus.
   * Les fichiers `.txt` ou `.xml` seront automatiquement générés dans le dossier `corpus_txt/`.

---

## 🛠️ Modes de sortie disponibles

Vous pouvez choisir entre deux formats de résumé :

- **Texte brut** (`-t`) : génère un fichier `resumes.txt`
- **XML structuré** (`-x`) : génère un fichier `articles.xml`

### Exemples :
```bash
./main.sh -t fichier1.pdf          # Sortie en texte
./main.sh -x fichier1.pdf fichier2.pdf   # Sortie en XML
```

> 📝 Le paramètre `-x` ou `-t` peut être placé à n’importe quelle position dans la commande. Si aucun mode n’est spécifié, la sortie par défaut est en `txt`.

---

## 📌 Fonctionnalités clés

- Interface simple pour sélectionner des PDF (via Zenity)
- Extraction structurée en balises :
  ```xml
  <article>
    <preamble>Nom du fichier</preamble>
    <titre>...</titre>
    <auteur>...</auteur>
    <abstract>...</abstract>
    <introduction>...</introduction>
    <corps>...</corps>
    <conclusion>...</conclusion>
    <discussion>...</discussion>
    <biblio>...</biblio>
  </article>
  ```
- Compatible avec des corpus scientifiques complexes
- Architecture modulaire (Shell + Rust)

---

## ⚠️ Limitations connues

- Certains PDF très mal structurés (ex. : `michev.pdf`) peuvent générer des résultats erronés ou vides
- Le langage Rust, bien que performant, complexifie la maintenance si tous les membres de l’équipe ne le maîtrisent pas

---

## 📤 Résultats

- Les résumés générés se trouvent dans le dossier `resumes/`
- Les fichiers convertis depuis les PDF sont visibles dans `corpus_txt/`

---

## 📎 À venir

- Transcription du parseur Rust dans un langage plus accessible à l’équipe
- Optimisation des performances (temps de traitement divisé par 3 visé)
- Amélioration du taux de précision d’extraction au-delà de 80 %


---
