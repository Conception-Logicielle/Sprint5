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

## 📊 Tests d'Accuracy (Dossier `AccuracyTest`)

Dans le dossier `AccuracyTest`, vous trouverez des scripts pour comparer la qualité des résumés XML générés par rapport aux fichiers de référence.

### Deux types de comparaison sont proposés :

1. **Comparaison avec marge (ligne par ligne)**

    * Cette méthode compare les textes section par section en tenant compte d’une marge de décalage de lignes (±2 lignes).
    * Elle est cependant **trop stricte** et ne supporte pas bien les différences d’indentation, casse, ou coupures de mots.
    * Par conséquent, elle produit généralement un **taux de réussite faible**.

2. **Comparaison avec normalisation complète**

    * Cette méthode normalise les textes avant comparaison en supprimant retours à la ligne, différences de casse, espaces multiples, et tirets.
    * Elle réalise une comparaison plus souple basée sur l’inclusion textuelle.
    * Ce test donne un **taux de réussite plus élevé et plus représentatif** de la qualité réelle.

---

### Lancer les tests

Les scripts sont écrits en Node.js et s’exécutent via la commande :

```bash
node accuracyTest.js
```

Assurez-vous que les fichiers XML `articles.xml` (généré) et `expected.xml` (référence) sont bien présents dans les chemins configurés.

---

### Remarques

* La méthode avec marge est utile pour des cas très stricts, mais souvent trop sévère.
* La méthode avec normalisation est recommandée pour évaluer les résultats dans un cadre réel, avec des variations courantes dans la mise en forme.
* Les deux méthodes sont complémentaires et peuvent être utilisées selon vos besoins.