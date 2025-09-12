// Script to update package.json
import fs from 'node:fs'
import path from 'node:path'

const packageJsonPath = path.resolve('./package.json')

// Read the package.json file
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

// Update the lint scripts to use NODE_OPTIONS
packageJson.scripts.lint = 'cross-env NODE_OPTIONS=--no-warnings eslint .'
packageJson.scripts['lint:fix'] = 'cross-env NODE_OPTIONS=--no-warnings eslint . --fix'

// Write the updated package.json file
fs.writeFileSync(
  packageJsonPath,
  `${JSON.stringify(packageJson, null, 2)}\n`,
)

console.log('Successfully updated package.json')
