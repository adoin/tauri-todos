// Script to update package.json
import fs from 'node:fs'
import path from 'node:path'

const packageJsonPath = path.resolve('./package.json')

// Read the package.json file
const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'))

// Update the lint scripts
packageJson.scripts.lint = 'node --no-warnings=ExperimentalWarning eslint .'
packageJson.scripts['lint:fix'] = 'node --no-warnings=ExperimentalWarning eslint . --fix'

// Write the updated package.json file
fs.writeFileSync(packageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`)

console.log('Successfully updated package.json')
