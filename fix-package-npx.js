// Script to update package.json
import fs from 'fs';
import path from 'path';

const packageJsonPath = path.resolve('./package.json');

// Read the package.json file
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));

// Update the lint scripts to use npx
packageJson.scripts.lint = 'npx eslint .';
packageJson.scripts['lint:fix'] = 'npx eslint . --fix';

// Write the updated package.json file
fs.writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + '\n');

console.log('Successfully updated package.json');
