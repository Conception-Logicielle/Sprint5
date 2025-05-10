## 📚 Parseur d'Articles Scientifiques en Texte

Ce projet convertit automatiquement des articles scientifiques au format PDF en texte brut, puis génère des résumés (au format texte ou XML) via un parseur écrit en Rust.

---

### 🧰 Prérequis

- Un environnement **Linux/WSL** avec `bash`
- `zenity` (pour l’interface graphique simple)
  ```bash
  sudo apt install zenity
  ```

* `poppler-utils` (pour utiliser `pdftotext`)

  ```bash
  sudo apt install poppler-utils
  ```
* Un environnement **Rust** avec `cargo` installé :

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

---

### 📁 Structure du projet

```
- CORPUS_TRAIN/            # Dossier contenant les fichiers PDF à traiter
- Final_Version/
  ├── main.sh              # Script principal avec interface Zenity
  ├── pdftotext.sh         # Script de conversion PDF → texte via pdftotext
  ├── corpus_txt/          # Dossier généré contenant les fichiers .txt extraits
  ├── resumes/             # Dossier de sortie contenant les résumés générés
  └── extractInfo/
       └── main/           # Contient le code Rust (main.rs + Cargo.toml)
```

> 📌 Les fichiers PDF peuvent être sélectionnés via l’interface, peu importe leur emplacement.

---

### 🚀 Lancer le script principal

```bash
chmod +x main.sh
./main.sh
```
---

### 🔧 Modes de sortie

Le script Rust permet deux types de sortie :

* `txt` : un fichier `resumes.txt` contenant les titres, auteurs, résumés et références formatés.
* `xml` : un fichier `articles.xml` contenant les mêmes données sous forme de balises XML.

Choisir le modde en ligne de commande dans `main.sh` avec le paramètre `-x` :

```bash
./main.sh -x     # génère un fichier XML
./main.sh -t     # génère un fichier texte
```