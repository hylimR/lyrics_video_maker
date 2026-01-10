import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { importSubtitleToKLyric, serializeKLyric } from '../src/utils/KLyricFormat.js';

// Helper for ESM directory
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function main() {
    const args = process.argv.slice(2);

    if (args.length < 2) {
        console.error('Usage: node ass_to_json.js <input_file> <output_json>');
        process.exit(1);
    }

    const inputFile = args[0];
    const outputFile = args[1];

    if (!fs.existsSync(inputFile)) {
        console.error(`Error: Input file "${inputFile}" does not exist.`);
        process.exit(1);
    }

    try {
        const content = fs.readFileSync(inputFile, 'utf-8');
        const filename = path.basename(inputFile);

        console.log(`Converting ${filename} to KLyric JSON...`);

        // Convert options
        const options = {
            resolution: { width: 1920, height: 1080 },
            preserveRaw: true
        };

        const result = importSubtitleToKLyric(content, filename, options);

        // Serialize to JSON
        const jsonOutput = serializeKLyric(result.klyric, true);

        fs.writeFileSync(outputFile, jsonOutput, 'utf-8');

        console.log(`Successfully converted to ${outputFile}`);

    } catch (error) {
        console.error('Conversion Failed:', error.message);
        console.error(error.stack);
        process.exit(1);
    }
}

main();
