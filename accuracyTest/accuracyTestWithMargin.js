const fs = require('fs');
const parseString = require('xml2js').parseString;

const GENERATE_XML = "../Final_Version/resume/articles.xml";
const EXPECTED_XML = "expected.xml";
const ALLOWED_TAGS = [
    "articles", "article",
    "preamble", "titre", "auteur", "abstract",
    "introduction", "corps", "conclusion", "discussion", "biblio"
];

/**
 * @description Compare 2 strings ligne-par-ligne laissant les marges extra ou des lignes manquantes
 */
function normalizeLine(line) {
    return line.replace(/-/g, '').trim().toLowerCase();
}

function compareWithMargin(texteGenere, texteAttendu, marge = 2) {
    const lignesGenerees = texteGenere.split(/\r?\n/).map(ligne => ligne.trim()).filter(ligne => ligne.length);
    const lignesAttendues = texteAttendu.split(/\r?\n/).map(ligne => ligne.trim()).filter(ligne => ligne.length);
    const nbLignesGenerees = lignesGenerees.length;
    const nbLignesAttendues = lignesAttendues.length;

    if (nbLignesAttendues === 0) return false;

    for (let positionDebut = 0; positionDebut <= nbLignesGenerees - nbLignesAttendues; positionDebut++) {
        const lignesAvant = positionDebut;
        const lignesApres = nbLignesGenerees - (positionDebut + nbLignesAttendues);

        if (lignesAvant <= marge && lignesApres <= marge) {
            let correspondance = true;
            for (let i = 0; i < nbLignesAttendues; i++) {
                if (lignesGenerees[positionDebut + i] !== lignesAttendues[i]) {
                    correspondance = false;
                    break;
                }
            }
            if (correspondance) return true;
        }
    }
    return false;
}



/**
 * @description Retourne true si le titre (une fois qu'il est mis sur une ligne) est exactement le meme que celui attendu
 */
function verifyTitle(generated, expected) {
    return generated.trim() === expected.trim();
}

/**
 * @description Retourne true si les auteurs sont les meme que ceux attendus.
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * */
function verifyAuthors(generated, expected) {
    return compareWithMargin(generated, expected, 2);
}

/**
 * @description Retourne true si l'abstract est le meme que celui attendu
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyAbstract(generated, expected) {
    return compareWithMargin(generated, expected, 2);
}

/**
 * @description Retourne true si les l'introduction est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyIntroduction(generated, expected) {
    return compareWithMargin(generated, expected, 2);
}

/**
 * @description Retourne true si le body est le meme que celui attendu
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyBody(genere, attendu) {
    return lignesSontSimilaires(genere, attendu);
}

/**
 * @description Retourne true si la conclusion est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * Si il n'y en a pas, il faut que la fonction le prenne en compte et verifie que "Aucune conclusion trouvée." est bien écrit
 */
function verifyConclusion(genere, attendu) {
    return lignesSontSimilaires(genere, attendu, "Aucune conclusion trouvée.");
}

/**
 * @description Retourne true si la discussion est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * Si il n'y en a pas, il faut que la fonction le prenne en compte et verifie que "Aucune discussion trouvée." est bien écrit
 */
function verifyDiscussion(genere, attendu) {
    return lignesSontSimilaires(genere, attendu, "Aucune discussion trouvée.");
}

/**
 * @description Retourne true si la bibliographie est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyBibliography(genere, attendu) {
    return lignesSontSimilaires(genere, attendu);
}

function wrapCDataInTags(xml, tags) {
    tags.forEach(tag => {
        const regex = new RegExp(`<${tag}>([\\s\\S]*?)<\\/${tag}>`, 'gi');
        xml = xml.replace(regex, (match, content) => {
            if (content.includes('<![CDATA[')) return match;
            return `<${tag}><![CDATA[${content}]]></${tag}>`;
        });
    });
    return xml;
}

function parseXML(filepath, callback) {
    fs.readFile(filepath, (err, data) => {
        if (err) return callback(err);

        let raw = data.toString()
            .replace(/&(?!(?:amp|lt|gt|quot|apos|#\d+);)/g, "&amp;")
            .replace(/<([\/]?)([a-zA-Z0-9]+)[^>]*>/g, (match, slash, tag) =>
                ALLOWED_TAGS.includes(tag.toLowerCase()) ? match : ''
            );

        raw = wrapCDataInTags(raw, [
            "titre", "auteur", "abstract", "introduction",
            "corps", "conclusion", "discussion", "biblio"
        ]);

        parseString(raw, { explicitArray: true }, (err, result) => {
            if (err) return callback(err);

            const root = result.articles || result.ARTICLES || result.Articles;
            if (!root || !root.article) return callback(new Error("Structure XML vide"));

            callback(null, root.article);
        });
    });
}

function computeAccuracy() {
    const summary = {
        titre: 0, auteur: 0, abstract: 0, introduction: 0,
        corps: 0, conclusion: 0, discussion: 0, biblio: 0
    };

    let total_sections_trouvees = 0;

    parseXML(GENERATE_XML, (err, generatedArticles) => {
        if (err) return console.error("Erreur lecture XML généré :", err);

        parseXML(EXPECTED_XML, (err, expectedArticles) => {
            if (err) return console.error("Erreur lecture XML attendu :", err);

            // Créer une map des articles attendus par préambule
            const expectedMap = {};
            for (const art of expectedArticles) {
                const preamble = art.preamble?.[0]?.trim();
                if (preamble) {
                    if (!expectedMap[preamble]) expectedMap[preamble] = [];
                    expectedMap[preamble].push(art);
                }
            }

            for (const gen of generatedArticles) {
                const preamble = gen.preamble?.[0]?.trim();
                if (!preamble || !expectedMap[preamble] || expectedMap[preamble].length === 0) {
                    continue; // Aucun article attendu avec ce même préambule
                }

                // Prendre et retirer le premier article attendu correspondant
                const exp = expectedMap[preamble].shift();

                function check(section, fn) {
                    total_sections_trouvees++;

                    const g = gen[section];
                    const e = exp[section];

                    if (!g || !e) {
                        console.warn(`Article "${preamble}" : section "${section}" absente ` +
                            `${!g ? 'dans le fichier généré' : ''}` +
                            `${!g && !e ? ' et ' : ''}` +
                            `${!e ? 'dans le fichier attendu' : ''}.`);
                        return;
                    }

                    if (fn(g[0], e[0])) {
                        summary[section]++;
                    }
                }

                check("titre", verifyTitle);
                check("auteur", verifyAuthors);
                check("abstract", verifyAbstract);
                check("introduction", verifyIntroduction);
                check("corps", verifyBody);
                check("conclusion", verifyConclusion);
                check("discussion", verifyDiscussion);
                check("biblio", verifyBibliography);
            }

            const total_correct = Object.values(summary).reduce((acc, val) => acc + val, 0);
            const accuracy = total_sections_trouvees > 0 ? (total_correct / total_sections_trouvees) : 0;

            console.log("\nRésumé des vérifications :");
            console.log(summary);
            console.log(`\nSections correctes : ${total_correct}`);
            console.log(`Sections trouvées   : ${total_sections_trouvees}`);
            console.log(`Précision            : ${(accuracy * 100).toFixed(2)} %`);
        });
    });
}

/**
 * Compare deux textes ligne par ligne avec une tolérance de 2 lignes en plus ou en moins
 * (au début ou à la fin). Si le texte attendu est vide et qu'un messageManquant est fourni,
 * vérifie que le texte généré contient exactement ce message. Retourne true si les textes
 * sont considérés comme similaires selon ces critères.
 * @param {string} genere - Le texte généré à comparer
 * @param {string} attendu - Le texte attendu
 * @param {string|null} messageManquant - Message à attendre si le texte attendu est vide
 * @returns {boolean}
 */
function lignesSontSimilaires(genere, attendu, messageManquant = null) {
    // Découper les textes en lignes, retirer les espaces inutiles
    const lignesGen = genere.trim().split(/\r?\n/).map(ligne => ligne.trim()).filter(ligne => ligne.length > 0);
    const lignesAtt = attendu.trim().split(/\r?\n/).map(ligne => ligne.trim()).filter(ligne => ligne.length > 0);

    // Cas où aucune ligne attendue, on vérifie le message spécial
    if (lignesAtt.length === 0 && messageManquant) {
        return lignesGen.length === 1 && lignesGen[0] === messageManquant;
    }

    // Si la différence de nombre de lignes est trop grande, ce n'est pas valide
    if (Math.abs(lignesGen.length - lignesAtt.length) > 2) return false;

    // On teste tous les décalages possibles (marge d'erreur de 2 lignes)
    for (let decalage = -2; decalage <= 2; decalage++) {
        let debutGen = Math.max(0, decalage);
        let debutAtt = Math.max(0, -decalage);
        let longueur = Math.min(lignesGen.length - debutGen, lignesAtt.length - debutAtt);
        if (longueur < lignesAtt.length - 2) continue;
        let ok = true;
        for (let i = 0; i < longueur; i++) {
            if (lignesGen[debutGen + i] !== lignesAtt[debutAtt + i]) {
                ok = false;
                break;
            }
        }
        if (ok) return true;
    }
    return false;
}

computeAccuracy();