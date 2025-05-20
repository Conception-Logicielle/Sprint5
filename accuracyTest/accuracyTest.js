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
 * @description Retourne true si le titre (une fois qu'il est mis sur une ligne) est exactement le meme que celui attendu
 */
function verifiyTitle(generated, expected) {
    return true
}

/**
 * @description Retourne true si les auteurs sont les meme que ceux attendus.
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * */
function verifyAuthors(generated, expected) {
    return true
}

/**
 * @description Retourne true si l'abstract est le meme que celui attendu
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyAbstract(generated, expected) {
    return true
}

/**
 * @description Retourne true si les l'introduction est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyIntroduction(generated, expected) {
    return true
}

/**
 * @description Retourne true si le body est le meme que celui attendu
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyBody(generated, expected) {
    return true
}

/**
 * @description Retourne true si la conclusion est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * Si il n'y en a pas, il faut que la fonction le prenne en compte et verifie que "Aucune conclusion trouvée." est bien écrit
 */
function verifyConclusion(generated, expected) {
    return true
}

/**
 * @description Retourne true si la discussion est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 * Si il n'y en a pas, il faut que la fonction le prenne en compte et verifie que "Aucune discussion trouvée." est bien écrit
 */
function verifyDiscussion(generated, expected) {
    return true
}

/**
 * @description Retourne true si la bibliographie est la meme que celle attendue
 * WARN : si il y a (max) deux ligne en plus que celles attendus, ou deux lignes oubliées, que ce soit au début ou a la fin, le test doit réussir (marge d'erreur)
 */
function verifyBibliography(generated, expected) {
    return true
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

            const count = Math.min(generatedArticles.length, expectedArticles.length);

            for (let i = 0; i < count; i++) {
                const gen = generatedArticles[i];
                const exp = expectedArticles[i];

                function check(section, fn) {
                    total_sections_trouvees++;

                    const g = gen[section];
                    const e = exp[section];

                    if (!g || !e) {
                        console.warn(`Article ${i + 1} : section "${section}" absente ` +
                            `${!g ? 'dans le fichier généré' : ''}` +
                            `${!g && !e ? ' et ' : ''}` +
                            `${!e ? 'dans le fichier attendu' : ''}.`);
                        return;
                    }

                    if (fn(g[0], e[0])) {
                        summary[section]++;
                    }
                }

                check("titre", verifiyTitle);
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

computeAccuracy();