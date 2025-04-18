## 📚 Parseur d'Articles Scientifiques en Texte

Ce projet convertit automatiquement des articles scientifiques au format PDF en texte brut et
en fait des résumés.

---

### 🧰 Prérequis

il est necessaire d’avoir :

- Un environnement **Linux/WSL** avec `bash`  
- **Poppler-utils** (pour utiliser `pdftotext`)  
  ```bash
  sudo apt update
  sudo apt install poppler-utils
  ```
- **Un environnement Rust pour effectuer la génération de résumé**  

---

### 📁 Structure

```
- CORPUS_TRAIN   # Dossier contenant les fichiers PDF à traiter
- Final_Version
  ├── main.sh               # Script principal de conversion + mise en forme
  └── corpus_txt/             # Sortie texte générée automatiquement
```

> 📌 Les fichiers PDF doivent être placés dans `../CORPUS_TRAIN`  
> Le script générera un `.txt` par PDF dans `./corpus_txt`

---

### 🚀 Lancer le script

```bash
chmod +x main.sh
./main.sh
```

### AddON
Une interface a été ajoutée par notre membre Gautier Jourdon dans le dossier interface. Cette dernière permet de selectionner
un fichier PDF et de le convertir en texte brut peut importe le dossier dans lequel il se trouve.

Warn : Il faut lancer le programme python "Interface" sur linux