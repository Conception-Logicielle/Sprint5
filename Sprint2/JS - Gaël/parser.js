const fs = require('fs-extra');
const path = require('path');
const pdfParse = require('pdf-parse');

const INPUT_DIR = '../CORPUS_TRAIN';
const OUTPUT_DIR = './corpus_txt';

async function extractMetadata(text) {
    const lines = text.split('\n').map(line => line.trim()).filter(Boolean);

    let title = 'Titre non trouvé';
    let abstract = 'Résumé non trouvé';

    for (let i = 0; i < lines.length; i++) {
        if (lines[i].toLowerCase().includes('abstract')) {
            title = lines[i - 1] || title;
            abstract = lines[i + 1] || abstract;
            break;
        }
    }

    return { title, abstract };
}

async function processPDF(filePath, fileName) {
    const dataBuffer = await fs.readFile(filePath);
    const pdfData = await pdfParse(dataBuffer);
    const { title, abstract } = await extractMetadata(pdfData.text);

    const outputName = fileName.replace(/\s+/g, '_').replace(/\.pdf$/i, '');
    const outputText = `${outputName}\n${title}\n${abstract}`;

    const outputPath = path.join(OUTPUT_DIR, `${outputName}.txt`);
    await fs.writeFile(outputPath, outputText, 'utf8');
    console.log(`✔ Fichier traité : ${outputPath}`);
}

async function main() {
    console.time('⏱ Temps total d\'exécution');

    try {
        if (await fs.pathExists(OUTPUT_DIR)) {
            await fs.remove(OUTPUT_DIR);
        }
        await fs.mkdir(OUTPUT_DIR);

        const files = await fs.readdir(INPUT_DIR);
        const pdfFiles = files.filter(file => file.toLowerCase().endsWith('.pdf'));

        if (pdfFiles.length === 0) {
            console.log('Aucun fichier PDF trouvé dans le dossier input.');
            return;
        }

        for (const file of pdfFiles) {
            const fullPath = path.join(INPUT_DIR, file);
            await processPDF(fullPath, file);
        }

        console.log('✅ Tous les fichiers ont été traités.');
    } catch (err) {
        console.error('❌ Erreur :', err);
    }

    console.timeEnd('⏱ Temps total d\'exécution');
}

main();